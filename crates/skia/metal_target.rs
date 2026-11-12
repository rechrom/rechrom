//! Metal render-target boundary for the translated GPU renderer.
//!
//! The ownership and submission sequence follows Skia's Metal backend:
//! `GrMtlDirectContext.mm` creates a Metal-backed context, while
//! `SkSurface_GaneshMtl.mm` wraps a renderable Metal texture. The translated
//! Ganesh/Graphite device is not complete yet. This module owns the real Metal
//! texture, render pass and synchronous readback boundary plus the first
//! translated draw subsets: compiled pipelines for SrcOver solid rectangles
//! and repeat-linear textured quads.

use core::{ffi::c_void, ptr::NonNull};

use objc2::{rc::Retained, runtime::ProtocolObject};
use objc2_foundation::NSString;
use objc2_metal::{
    MTLBlendFactor, MTLBlendOperation, MTLBlitCommandEncoder, MTLClearColor, MTLCommandBuffer,
    MTLCommandBufferStatus, MTLCommandEncoder, MTLCommandQueue, MTLCreateSystemDefaultDevice,
    MTLDevice, MTLLibrary, MTLLoadAction, MTLOrigin, MTLPixelFormat, MTLPrimitiveType, MTLRegion,
    MTLRenderCommandEncoder, MTLRenderPassColorAttachmentDescriptor, MTLRenderPassDescriptor,
    MTLRenderPipelineDescriptor, MTLRenderPipelineState, MTLSamplerAddressMode,
    MTLSamplerDescriptor, MTLSamplerMinMagFilter, MTLSamplerMipFilter, MTLSamplerState, MTLSize,
    MTLStorageMode, MTLStoreAction, MTLTexture, MTLTextureDescriptor, MTLTextureType,
    MTLTextureUsage, MTLViewport,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetalTargetError {
    InvalidDimensions,
    UnsupportedPixelFormat,
    BufferTooSmall,
    DeviceUnavailable,
    QueueCreationFailed,
    TextureCreationFailed,
    ShaderCompilationFailed,
    ShaderFunctionUnavailable,
    PipelineCreationFailed,
    CommandBufferCreationFailed,
    EncoderCreationFailed,
    SamplerCreationFailed,
    CommandFailed,
    InvalidRect,
    InvalidTexture,
    UnsupportedSampleCount,
}

/// BGRA8 render target backed by a real Metal texture.
///
/// Like `GrMtlAttachment`, the renderable texture uses private GPU storage. A
/// separate shared texture is only a synchronized readback bridge for the
/// current host-pixel renderer. A future IOSurface/CAMetalLayer owner can
/// remove that bridge without changing the command submission API.
pub struct MetalTarget {
    width: usize,
    height: usize,
    device: Retained<ProtocolObject<dyn MTLDevice>>,
    queue: Retained<ProtocolObject<dyn MTLCommandQueue>>,
    /// Single-sample texture exposed for sampling and host readback. Ganesh's
    /// WrapBackendTexture path resolves its transient MSAA attachment here.
    texture: Retained<ProtocolObject<dyn MTLTexture>>,
    multisample_texture: Option<Retained<ProtocolObject<dyn MTLTexture>>>,
    readback_texture: Retained<ProtocolObject<dyn MTLTexture>>,
    sample_count: usize,
    solid_pipeline: Retained<ProtocolObject<dyn MTLRenderPipelineState>>,
    texture_pipeline: Retained<ProtocolObject<dyn MTLRenderPipelineState>>,
    repeat_linear_sampler: Retained<ProtocolObject<dyn MTLSamplerState>>,
}

// Source: src/gpu/ganesh/mtl/GrMtlPipelineStateBuilder.mm. Ganesh emits MSL
// from SkSL and compiles one MTLLibrary before creating an MTL pipeline state.
// This first translated draw primitive keeps that ownership/submission order,
// while the complete SkSL/geometry-processor translation remains pending.
const PRIMITIVE_MSL: &str = r#"
#include <metal_stdlib>
using namespace metal;

struct SolidVertexOut {
    float4 position [[position]];
};

vertex SolidVertexOut rechrom_solid_vertex(
        uint vertex_id [[vertex_id]],
        constant float2* positions [[buffer(0)]]) {
    SolidVertexOut out;
    out.position = float4(positions[vertex_id], 0.0, 1.0);
    return out;
}

fragment float4 rechrom_solid_fragment(constant float4& color [[buffer(0)]]) {
    return color;
}

struct TextureVertexOut {
    float4 position [[position]];
    float3 texture_coordinate;
};

vertex TextureVertexOut rechrom_texture_vertex(
        uint vertex_id [[vertex_id]],
        constant float2* positions [[buffer(0)]],
        constant float4* texture_coordinates [[buffer(1)]]) {
    TextureVertexOut out;
    out.position = float4(positions[vertex_id], 0.0, 1.0);
    out.texture_coordinate = texture_coordinates[vertex_id].xyz;
    return out;
}

fragment float4 rechrom_texture_fragment(
        TextureVertexOut in [[stage_in]],
        texture2d<float> image [[texture(0)]],
        sampler image_sampler [[sampler(0)]],
        constant float& opacity [[buffer(0)]]) {
    float2 coordinate = in.texture_coordinate.xy / in.texture_coordinate.z;
    return image.sample(image_sampler, coordinate) * opacity;
}
"#;

impl MetalTarget {
    pub fn new(width: usize, height: usize) -> Result<Self, MetalTargetError> {
        Self::new_with_sample_count(width, height, 1)
    }

    /// Create the Metal attachment topology selected by Chromium's
    /// `SkiaOutputDeviceOffscreen::BeginPaint`: a single-sample backend
    /// texture plus, when requested, a transient multisample render target
    /// which resolves into it.
    ///
    /// Source: components/viz/service/display_embedder/
    /// skia_output_device_offscreen.cc:184-195 and
    /// third_party/skia/src/gpu/ganesh/mtl/GrMtlAttachment.mm:114-139.
    pub fn new_with_sample_count(
        width: usize,
        height: usize,
        sample_count: usize,
    ) -> Result<Self, MetalTargetError> {
        if width == 0
            || height == 0
            || width > isize::MAX as usize / 4
            || height > isize::MAX as usize / (width * 4)
        {
            return Err(MetalTargetError::InvalidDimensions);
        }

        let device = unsafe { Retained::retain(MTLCreateSystemDefaultDevice()) }
            .ok_or(MetalTargetError::DeviceUnavailable)?;
        if sample_count == 0 || !device.supportsTextureSampleCount(sample_count) {
            return Err(MetalTargetError::UnsupportedSampleCount);
        }
        let queue = device
            .newCommandQueue()
            .ok_or(MetalTargetError::QueueCreationFailed)?;
        let descriptor = unsafe {
            MTLTextureDescriptor::texture2DDescriptorWithPixelFormat_width_height_mipmapped(
                MTLPixelFormat::BGRA8Unorm,
                width,
                height,
                false,
            )
        };
        descriptor.setStorageMode(MTLStorageMode::Private);
        descriptor.setUsage(MTLTextureUsage::RenderTarget | MTLTextureUsage::ShaderRead);
        let texture = device
            .newTextureWithDescriptor(&descriptor)
            .ok_or(MetalTargetError::TextureCreationFailed)?;
        let multisample_texture = if sample_count > 1 {
            let multisample_descriptor = unsafe {
                MTLTextureDescriptor::texture2DDescriptorWithPixelFormat_width_height_mipmapped(
                    MTLPixelFormat::BGRA8Unorm,
                    width,
                    height,
                    false,
                )
            };
            multisample_descriptor.setTextureType(MTLTextureType::MTLTextureType2DMultisample);
            unsafe { multisample_descriptor.setSampleCount(sample_count) };
            multisample_descriptor.setStorageMode(MTLStorageMode::Private);
            multisample_descriptor.setUsage(MTLTextureUsage::RenderTarget);
            Some(
                device
                    .newTextureWithDescriptor(&multisample_descriptor)
                    .ok_or(MetalTargetError::TextureCreationFailed)?,
            )
        } else {
            None
        };
        descriptor.setStorageMode(MTLStorageMode::Shared);
        descriptor.setUsage(MTLTextureUsage::Unknown);
        let readback_texture = device
            .newTextureWithDescriptor(&descriptor)
            .ok_or(MetalTargetError::TextureCreationFailed)?;

        let source = NSString::from_str(PRIMITIVE_MSL);
        let library = device
            .newLibraryWithSource_options_error(&source, None)
            .map_err(|_| MetalTargetError::ShaderCompilationFailed)?;
        let vertex = library
            .newFunctionWithName(&NSString::from_str("rechrom_solid_vertex"))
            .ok_or(MetalTargetError::ShaderFunctionUnavailable)?;
        let fragment = library
            .newFunctionWithName(&NSString::from_str("rechrom_solid_fragment"))
            .ok_or(MetalTargetError::ShaderFunctionUnavailable)?;
        let pipeline = MTLRenderPipelineDescriptor::new();
        pipeline.setVertexFunction(Some(&vertex));
        pipeline.setFragmentFunction(Some(&fragment));
        pipeline.setRasterSampleCount(sample_count);
        let color_attachment = unsafe { pipeline.colorAttachments().objectAtIndexedSubscript(0) };
        color_attachment.setPixelFormat(MTLPixelFormat::BGRA8Unorm);
        // Skia's SrcOver xfer processor writes premultiplied source color.
        // Source: src/gpu/ganesh/mtl/GrMtlPipelineStateBuilder.mm:332-360.
        color_attachment.setBlendingEnabled(true);
        color_attachment.setSourceRGBBlendFactor(MTLBlendFactor::One);
        color_attachment.setDestinationRGBBlendFactor(MTLBlendFactor::OneMinusSourceAlpha);
        color_attachment.setRgbBlendOperation(MTLBlendOperation::Add);
        color_attachment.setSourceAlphaBlendFactor(MTLBlendFactor::One);
        color_attachment.setDestinationAlphaBlendFactor(MTLBlendFactor::OneMinusSourceAlpha);
        color_attachment.setAlphaBlendOperation(MTLBlendOperation::Add);
        let solid_pipeline = device
            .newRenderPipelineStateWithDescriptor_error(&pipeline)
            .map_err(|_| MetalTargetError::PipelineCreationFailed)?;

        let texture_vertex = library
            .newFunctionWithName(&NSString::from_str("rechrom_texture_vertex"))
            .ok_or(MetalTargetError::ShaderFunctionUnavailable)?;
        let texture_fragment = library
            .newFunctionWithName(&NSString::from_str("rechrom_texture_fragment"))
            .ok_or(MetalTargetError::ShaderFunctionUnavailable)?;
        let texture_pipeline_descriptor = MTLRenderPipelineDescriptor::new();
        texture_pipeline_descriptor.setVertexFunction(Some(&texture_vertex));
        texture_pipeline_descriptor.setFragmentFunction(Some(&texture_fragment));
        texture_pipeline_descriptor.setRasterSampleCount(sample_count);
        let texture_color_attachment = unsafe {
            texture_pipeline_descriptor
                .colorAttachments()
                .objectAtIndexedSubscript(0)
        };
        texture_color_attachment.setPixelFormat(MTLPixelFormat::BGRA8Unorm);
        texture_color_attachment.setBlendingEnabled(true);
        texture_color_attachment.setSourceRGBBlendFactor(MTLBlendFactor::One);
        texture_color_attachment.setDestinationRGBBlendFactor(MTLBlendFactor::OneMinusSourceAlpha);
        texture_color_attachment.setRgbBlendOperation(MTLBlendOperation::Add);
        texture_color_attachment.setSourceAlphaBlendFactor(MTLBlendFactor::One);
        texture_color_attachment
            .setDestinationAlphaBlendFactor(MTLBlendFactor::OneMinusSourceAlpha);
        texture_color_attachment.setAlphaBlendOperation(MTLBlendOperation::Add);
        let texture_pipeline = device
            .newRenderPipelineStateWithDescriptor_error(&texture_pipeline_descriptor)
            .map_err(|_| MetalTargetError::PipelineCreationFailed)?;

        // Source: GrMtlSampler.mm:45-82. This is the exact sampler state used
        // for a repeat/repeat, linear, non-mipmapped normalized texture draw.
        let sampler_descriptor = MTLSamplerDescriptor::new();
        sampler_descriptor.setRAddressMode(MTLSamplerAddressMode::ClampToEdge);
        sampler_descriptor.setSAddressMode(MTLSamplerAddressMode::Repeat);
        sampler_descriptor.setTAddressMode(MTLSamplerAddressMode::Repeat);
        sampler_descriptor.setMagFilter(MTLSamplerMinMagFilter::Linear);
        sampler_descriptor.setMinFilter(MTLSamplerMinMagFilter::Linear);
        sampler_descriptor.setMipFilter(MTLSamplerMipFilter::NotMipmapped);
        sampler_descriptor.setLodMinClamp(0.0);
        sampler_descriptor.setLodMaxClamp(f32::MAX);
        sampler_descriptor.setMaxAnisotropy(1);
        sampler_descriptor.setNormalizedCoordinates(true);
        let repeat_linear_sampler = device
            .newSamplerStateWithDescriptor(&sampler_descriptor)
            .ok_or(MetalTargetError::SamplerCreationFailed)?;

        Ok(Self {
            width,
            height,
            device,
            queue,
            texture,
            multisample_texture,
            readback_texture,
            sample_count,
            solid_pipeline,
            texture_pipeline,
            repeat_linear_sampler,
        })
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn sample_count(&self) -> usize {
        self.sample_count
    }

    fn configure_color_attachment(&self, attachment: &MTLRenderPassColorAttachmentDescriptor) {
        if let Some(multisample_texture) = &self.multisample_texture {
            attachment.setTexture(Some(multisample_texture));
            attachment.setResolveTexture(Some(&self.texture));
            // The current translated subset submits multiple passes to the
            // same target. Preserve MSAA samples between them while resolving
            // every completed pass for readback/sampling.
            attachment.setStoreAction(MTLStoreAction::StoreAndMultisampleResolve);
        } else {
            attachment.setTexture(Some(&self.texture));
            attachment.setStoreAction(MTLStoreAction::Store);
        }
    }

    /// Submit a genuine Metal render pass whose load action clears the target.
    /// Completion is waited here because the next translated boundary is a
    /// host readback; later GPU composition will carry the command buffer/fence
    /// instead of blocking.
    pub fn clear(&self, color: [f64; 4]) -> Result<(), MetalTargetError> {
        // Objective-C `+new` returns a retained descriptor of the declared
        // class; objc2 marks generic NSObject construction unsafe because it
        // cannot prove framework initialization contracts.
        let pass = unsafe { MTLRenderPassDescriptor::new() };
        let attachment = unsafe { pass.colorAttachments().objectAtIndexedSubscript(0) };
        self.configure_color_attachment(&attachment);
        attachment.setLoadAction(MTLLoadAction::Clear);
        attachment.setClearColor(MTLClearColor {
            red: color[0],
            green: color[1],
            blue: color[2],
            alpha: color[3],
        });

        let command = self
            .queue
            .commandBuffer()
            .ok_or(MetalTargetError::CommandBufferCreationFailed)?;
        let encoder = command
            .renderCommandEncoderWithDescriptor(&pass)
            .ok_or(MetalTargetError::EncoderCreationFailed)?;
        encoder.endEncoding();
        command.commit();
        unsafe { command.waitUntilCompleted() };
        if command.status() != MTLCommandBufferStatus::Completed {
            return Err(MetalTargetError::CommandFailed);
        }
        Ok(())
    }

    /// Draw one premultiplied SrcOver rectangle through a real Metal render
    /// pipeline. This is the first executable subset of Ganesh's
    /// `GrMtlOpsRenderPass::onBindPipeline` + `onDraw` sequence: bind pipeline,
    /// set viewport and buffers, draw a triangle strip, submit and wait.
    ///
    /// Coordinates are top-left device pixels `[left, top, right, bottom]`.
    /// The full SkCanvas/Ganesh playback capability remains false until paths,
    /// images, clips, transforms and effects use the same GPU device.
    pub fn draw_solid_rect(&self, rect: [f32; 4], color: [f32; 4]) -> Result<(), MetalTargetError> {
        if !rect
            .iter()
            .chain(color.iter())
            .all(|value| value.is_finite())
        {
            return Err(MetalTargetError::InvalidRect);
        }
        let left = rect[0].clamp(0.0, self.width as f32);
        let top = rect[1].clamp(0.0, self.height as f32);
        let right = rect[2].clamp(0.0, self.width as f32);
        let bottom = rect[3].clamp(0.0, self.height as f32);
        if right <= left || bottom <= top {
            return Ok(());
        }
        let x = |value: f32| value * (2.0 / self.width as f32) - 1.0;
        let y = |value: f32| 1.0 - value * (2.0 / self.height as f32);
        let vertices = [
            [x(left), y(top)],
            [x(left), y(bottom)],
            [x(right), y(top)],
            [x(right), y(bottom)],
        ];
        let alpha = color[3].clamp(0.0, 1.0);
        let premultiplied = [
            color[0].clamp(0.0, 1.0) * alpha,
            color[1].clamp(0.0, 1.0) * alpha,
            color[2].clamp(0.0, 1.0) * alpha,
            alpha,
        ];

        let pass = unsafe { MTLRenderPassDescriptor::new() };
        let attachment = unsafe { pass.colorAttachments().objectAtIndexedSubscript(0) };
        self.configure_color_attachment(&attachment);
        attachment.setLoadAction(MTLLoadAction::Load);
        let command = self
            .queue
            .commandBuffer()
            .ok_or(MetalTargetError::CommandBufferCreationFailed)?;
        let encoder = command
            .renderCommandEncoderWithDescriptor(&pass)
            .ok_or(MetalTargetError::EncoderCreationFailed)?;
        // Source: GrMtlOpsRenderPass::initRenderState and onBindPipeline.
        encoder.setViewport(MTLViewport {
            originX: 0.0,
            originY: 0.0,
            width: self.width as f64,
            height: self.height as f64,
            znear: 0.0,
            zfar: 1.0,
        });
        encoder.setRenderPipelineState(&self.solid_pipeline);
        unsafe {
            encoder.setVertexBytes_length_atIndex(
                NonNull::from(&vertices).cast(),
                core::mem::size_of_val(&vertices),
                0,
            );
            encoder.setFragmentBytes_length_atIndex(
                NonNull::from(&premultiplied).cast(),
                core::mem::size_of_val(&premultiplied),
                0,
            );
            encoder.drawPrimitives_vertexStart_vertexCount(
                MTLPrimitiveType::TriangleStrip,
                0,
                vertices.len(),
            );
        }
        encoder.endEncoding();
        command.commit();
        unsafe { command.waitUntilCompleted() };
        if command.status() != MTLCommandBufferStatus::Completed {
            return Err(MetalTargetError::CommandFailed);
        }
        Ok(())
    }

    /// Draw a premultiplied RGBA8 texture through Skia's repeat/repeat,
    /// linear, non-mipmapped sampler state and SrcOver blending.
    ///
    /// `local_matrix` is Skia's row-major 3x3 matrix from top-left
    /// destination-device coordinates to normalized homogeneous texture
    /// coordinates. Values outside `[0, 1]` repeat in both axes. The matrix is
    /// evaluated at the clipped quad corners, preserving pattern phase when a
    /// draw crosses the target.
    pub fn draw_repeat_linear_textured_quad(
        &self,
        rect: [f32; 4],
        source_width: usize,
        source_height: usize,
        source_rgba: &[u8],
        source_row_bytes: usize,
        local_matrix: [f32; 9],
        opacity: f32,
    ) -> Result<(), MetalTargetError> {
        if !rect.iter().all(|value| value.is_finite()) {
            return Err(MetalTargetError::InvalidRect);
        }
        if source_width == 0
            || source_height == 0
            || source_width > isize::MAX as usize / 4
            || !local_matrix.iter().all(|value| value.is_finite())
            || !opacity.is_finite()
        {
            return Err(MetalTargetError::InvalidTexture);
        }
        let minimum_row = source_width
            .checked_mul(4)
            .ok_or(MetalTargetError::InvalidTexture)?;
        let required = source_row_bytes
            .checked_mul(source_height)
            .ok_or(MetalTargetError::InvalidTexture)?;
        if source_row_bytes < minimum_row || source_rgba.len() < required {
            return Err(MetalTargetError::InvalidTexture);
        }

        let left = rect[0].clamp(0.0, self.width as f32);
        let top = rect[1].clamp(0.0, self.height as f32);
        let right = rect[2].clamp(0.0, self.width as f32);
        let bottom = rect[3].clamp(0.0, self.height as f32);
        if right <= left || bottom <= top {
            return Ok(());
        }

        let texture_descriptor = unsafe {
            MTLTextureDescriptor::texture2DDescriptorWithPixelFormat_width_height_mipmapped(
                MTLPixelFormat::RGBA8Unorm,
                source_width,
                source_height,
                false,
            )
        };
        texture_descriptor.setStorageMode(MTLStorageMode::Shared);
        texture_descriptor.setUsage(MTLTextureUsage::ShaderRead);
        let source_texture = self
            .device
            .newTextureWithDescriptor(&texture_descriptor)
            .ok_or(MetalTargetError::TextureCreationFailed)?;
        let source_pointer = NonNull::new(source_rgba.as_ptr().cast_mut().cast::<c_void>())
            .ok_or(MetalTargetError::InvalidTexture)?;
        unsafe {
            source_texture.replaceRegion_mipmapLevel_withBytes_bytesPerRow(
                MTLRegion {
                    origin: MTLOrigin { x: 0, y: 0, z: 0 },
                    size: MTLSize {
                        width: source_width,
                        height: source_height,
                        depth: 1,
                    },
                },
                0,
                source_pointer,
                source_row_bytes,
            );
        }

        let x = |value: f32| value * (2.0 / self.width as f32) - 1.0;
        let y = |value: f32| 1.0 - value * (2.0 / self.height as f32);
        let positions = [
            [x(left), y(top)],
            [x(left), y(bottom)],
            [x(right), y(top)],
            [x(right), y(bottom)],
        ];
        let texture_coordinate = |device_x: f32, device_y: f32| {
            [
                local_matrix[0] * device_x + local_matrix[1] * device_y + local_matrix[2],
                local_matrix[3] * device_x + local_matrix[4] * device_y + local_matrix[5],
                local_matrix[6] * device_x + local_matrix[7] * device_y + local_matrix[8],
                0.0,
            ]
        };
        let texture_coordinates = [
            texture_coordinate(left, top),
            texture_coordinate(left, bottom),
            texture_coordinate(right, top),
            texture_coordinate(right, bottom),
        ];
        let opacity = opacity.clamp(0.0, 1.0);

        let pass = unsafe { MTLRenderPassDescriptor::new() };
        let attachment = unsafe { pass.colorAttachments().objectAtIndexedSubscript(0) };
        self.configure_color_attachment(&attachment);
        attachment.setLoadAction(MTLLoadAction::Load);
        let command = self
            .queue
            .commandBuffer()
            .ok_or(MetalTargetError::CommandBufferCreationFailed)?;
        let encoder = command
            .renderCommandEncoderWithDescriptor(&pass)
            .ok_or(MetalTargetError::EncoderCreationFailed)?;
        encoder.setViewport(MTLViewport {
            originX: 0.0,
            originY: 0.0,
            width: self.width as f64,
            height: self.height as f64,
            znear: 0.0,
            zfar: 1.0,
        });
        encoder.setRenderPipelineState(&self.texture_pipeline);
        unsafe {
            encoder.setVertexBytes_length_atIndex(
                NonNull::from(&positions).cast(),
                core::mem::size_of_val(&positions),
                0,
            );
            encoder.setVertexBytes_length_atIndex(
                NonNull::from(&texture_coordinates).cast(),
                core::mem::size_of_val(&texture_coordinates),
                1,
            );
            // Source: GrMtlPipelineState.mm:130-135.
            encoder.setFragmentTexture_atIndex(Some(&source_texture), 0);
            encoder.setFragmentSamplerState_atIndex(Some(&self.repeat_linear_sampler), 0);
            encoder.setFragmentBytes_length_atIndex(
                NonNull::from(&opacity).cast(),
                core::mem::size_of_val(&opacity),
                0,
            );
            encoder.drawPrimitives_vertexStart_vertexCount(
                MTLPrimitiveType::TriangleStrip,
                0,
                positions.len(),
            );
        }
        encoder.endEncoding();
        command.commit();
        unsafe { command.waitUntilCompleted() };
        if command.status() != MTLCommandBufferStatus::Completed {
            return Err(MetalTargetError::CommandFailed);
        }
        Ok(())
    }

    /// Copy the completed BGRA8 texture into caller-owned host memory.
    pub fn read_pixels(&self, pixels: &mut [u8], row_bytes: usize) -> Result<(), MetalTargetError> {
        let minimum_row = self.width * 4;
        let required = row_bytes
            .checked_mul(self.height)
            .ok_or(MetalTargetError::BufferTooSmall)?;
        if row_bytes < minimum_row || pixels.len() < required {
            return Err(MetalTargetError::BufferTooSmall);
        }
        let destination = NonNull::new(pixels.as_mut_ptr().cast::<c_void>())
            .ok_or(MetalTargetError::BufferTooSmall)?;
        let origin = MTLOrigin { x: 0, y: 0, z: 0 };
        let size = MTLSize {
            width: self.width,
            height: self.height,
            depth: 1,
        };
        let command = self
            .queue
            .commandBuffer()
            .ok_or(MetalTargetError::CommandBufferCreationFailed)?;
        let encoder = command
            .blitCommandEncoder()
            .ok_or(MetalTargetError::EncoderCreationFailed)?;
        unsafe {
            encoder.copyFromTexture_sourceSlice_sourceLevel_sourceOrigin_sourceSize_toTexture_destinationSlice_destinationLevel_destinationOrigin(
                &self.texture,
                0,
                0,
                origin,
                size,
                &self.readback_texture,
                0,
                0,
                origin,
            );
        }
        encoder.endEncoding();
        command.commit();
        unsafe { command.waitUntilCompleted() };
        if command.status() != MTLCommandBufferStatus::Completed {
            return Err(MetalTargetError::CommandFailed);
        }
        unsafe {
            self.readback_texture
                .getBytes_bytesPerRow_fromRegion_mipmapLevel(
                    destination,
                    row_bytes,
                    MTLRegion { origin, size },
                    0,
                )
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metal_render_pass_clears_real_bgra_texture() {
        let target = MetalTarget::new(4, 3).expect("Metal device and texture");
        target
            .clear([1.0, 0.0, 1.0, 1.0])
            .expect("Metal clear command");
        let mut pixels = vec![0; 4 * 3 * 4];
        target.read_pixels(&mut pixels, 4 * 4).expect("readback");
        assert!(pixels
            .chunks_exact(4)
            .all(|pixel| pixel == [255, 0, 255, 255]));
    }

    #[test]
    fn rejects_invalid_layouts() {
        assert!(matches!(
            MetalTarget::new(0, 1),
            Err(MetalTargetError::InvalidDimensions)
        ));
        let target = MetalTarget::new(2, 2).expect("Metal target");
        assert_eq!(
            target.read_pixels(&mut [0; 15], 8),
            Err(MetalTargetError::BufferTooSmall)
        );
        assert!(matches!(
            MetalTarget::new_with_sample_count(2, 2, 0),
            Err(MetalTargetError::UnsupportedSampleCount)
        ));
    }

    #[test]
    fn four_sample_target_resolves_partial_device_coverage() {
        let target = MetalTarget::new_with_sample_count(4, 2, 4).expect("4x MSAA Metal target");
        assert_eq!(target.sample_count(), 4);
        target.clear([0.0, 0.0, 0.0, 1.0]).expect("black clear");
        target
            .draw_solid_rect([0.5, 0.0, 3.5, 2.0], [1.0, 1.0, 1.0, 1.0])
            .expect("half-pixel white rect");
        let mut pixels = vec![0; 4 * 2 * 4];
        target
            .read_pixels(&mut pixels, 4 * 4)
            .expect("MSAA resolve readback");
        let pixel = |x: usize, y: usize| &pixels[(y * 4 + x) * 4..(y * 4 + x + 1) * 4];
        let left = pixel(0, 0);
        assert_eq!(left[0], left[1]);
        assert_eq!(left[1], left[2]);
        assert!((32..=223).contains(&left[0]));
        assert_eq!(left[3], 255);
        assert_eq!(pixel(1, 0), [255, 255, 255, 255]);
        assert_eq!(pixel(2, 1), [255, 255, 255, 255]);
        let right = pixel(3, 1);
        assert_eq!(right[0], right[1]);
        assert_eq!(right[1], right[2]);
        assert!((32..=223).contains(&right[0]));
        assert_eq!(right[3], 255);
    }

    #[test]
    fn metal_pipeline_draws_and_blends_solid_rect() {
        let target = MetalTarget::new(4, 4).expect("Metal target and pipeline");
        target.clear([0.0, 0.0, 1.0, 1.0]).expect("blue clear");
        target
            .draw_solid_rect([1.0, 1.0, 3.0, 3.0], [1.0, 0.0, 0.0, 0.5])
            .expect("SrcOver red rect");
        let mut pixels = vec![0; 4 * 4 * 4];
        target.read_pixels(&mut pixels, 4 * 4).expect("readback");
        let pixel = |x: usize, y: usize| &pixels[(y * 4 + x) * 4..(y * 4 + x + 1) * 4];
        assert_eq!(pixel(0, 0), [255, 0, 0, 255]);
        // BGRA bytes for half-red SrcOver opaque blue. Metal's UNorm blend
        // may round the half channel to either adjacent integer.
        let center = pixel(1, 1);
        assert_eq!(center[1], 0);
        assert_eq!(center[3], 255);
        assert!((127..=128).contains(&center[0]));
        assert!((127..=128).contains(&center[2]));
        assert_eq!(pixel(3, 3), [255, 0, 0, 255]);
    }

    #[test]
    fn metal_pipeline_repeats_linear_texture_with_local_matrix() {
        let target = MetalTarget::new(4, 1).expect("Metal target and texture pipeline");
        target.clear([0.0, 0.0, 0.0, 1.0]).expect("black clear");
        let red_green_rgba = [255, 0, 0, 255, 0, 255, 0, 255];
        target
            .draw_repeat_linear_textured_quad(
                [0.0, 0.0, 4.0, 1.0],
                2,
                1,
                &red_green_rgba,
                8,
                [0.5, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 1.0],
                1.0,
            )
            .expect("repeat-linear texture draw");
        let mut pixels = vec![0; 4 * 4];
        target.read_pixels(&mut pixels, 4 * 4).expect("readback");
        assert_eq!(&pixels[0..4], &[0, 0, 255, 255]);
        assert_eq!(&pixels[4..8], &[0, 255, 0, 255]);
        assert_eq!(&pixels[8..12], &[0, 0, 255, 255]);
        assert_eq!(&pixels[12..16], &[0, 255, 0, 255]);

        // A constant coordinate halfway between the two texel centers must
        // produce the linear-filtered red/green midpoint, not either texel.
        target
            .draw_repeat_linear_textured_quad(
                [0.0, 0.0, 1.0, 1.0],
                2,
                1,
                &red_green_rgba,
                8,
                [0.0, 0.0, 0.5, 0.0, 0.0, 0.5, 0.0, 0.0, 1.0],
                1.0,
            )
            .expect("linear midpoint texture draw");
        target.read_pixels(&mut pixels, 4 * 4).expect("readback");
        assert_eq!(pixels[0], 0);
        assert!((127..=128).contains(&pixels[1]));
        assert!((127..=128).contains(&pixels[2]));
        assert_eq!(pixels[3], 255);
    }
}
