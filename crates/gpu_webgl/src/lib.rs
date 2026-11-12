#![allow(non_snake_case)]

#[cfg(target_os = "macos")]
mod cgl;

#[cfg(target_os = "macos")]
pub use cgl::CglWebGlContext;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShaderCompileResult {
    pub shader: u32,
    pub compiled: bool,
    pub info_log: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgramLinkResult {
    pub program: u32,
    pub linked: bool,
    pub info_log: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActiveInfo {
    pub name: String,
    pub size: i32,
    pub kind: u32,
}

/// Narrow command-buffer-client boundary corresponding to
/// `WebGraphicsContext3DProvider::ContextGL() -> gpu::gles2::GLES2Interface`.
/// Implementations execute commands on a real graphics context; WebGL object
/// validation and error semantics remain in the `webgl` package.
pub trait WebGlGpuContext {
    /// Only a provider with an independently presented low-latency surface
    /// admits Canvas's effective desynchronized mode. CGL readback does not.
    fn SupportsLowLatencyPresentation(&self) -> bool {
        false
    }
    fn Reshape(&mut self, width: u32, height: u32) -> Result<(), String>;
    fn CompileShader(&mut self, shader_type: u32, source: &str) -> ShaderCompileResult;
    fn LinkProgram(&mut self, vertex_shader: u32, fragment_shader: u32) -> ProgramLinkResult;
    fn DeleteShader(&mut self, shader: u32);
    fn ProgramParameter(&mut self, program: u32, parameter: u32) -> i32;
    fn ActiveUniform(&mut self, program: u32, index: u32) -> Option<ActiveInfo>;
    fn ActiveAttrib(&mut self, program: u32, index: u32) -> Option<ActiveInfo>;
    fn UniformLocation(&mut self, program: u32, name: &str) -> i32;
    fn UniformMatrix2fv(&mut self, location: i32, transpose: bool, values: &[f32]);
    fn UniformMatrix4fv(&mut self, location: i32, transpose: bool, values: &[f32]);
    fn UniformMatrix3fv(&mut self, location: i32, transpose: bool, values: &[f32]);
    fn Uniform1fv(&mut self, location: i32, values: &[f32]);
    fn Uniform2fv(&mut self, location: i32, values: &[f32]);
    fn Uniform3fv(&mut self, location: i32, values: &[f32]);
    fn Uniform4fv(&mut self, location: i32, values: &[f32]);
    fn Uniform2f(&mut self, location: i32, x: f32, y: f32);
    fn Uniform3f(&mut self, location: i32, x: f32, y: f32, z: f32);
    fn Uniform4f(&mut self, location: i32, x: f32, y: f32, z: f32, w: f32);
    fn Uniform1f(&mut self, location: i32, value: f32);
    fn Uniform1iv(&mut self, location: i32, values: &[i32]);
    fn Uniform2iv(&mut self, location: i32, values: &[i32]);
    fn Uniform3iv(&mut self, location: i32, values: &[i32]);
    fn Uniform4iv(&mut self, location: i32, values: &[i32]);
    fn Uniform1i(&mut self, location: i32, value: i32);
    fn Uniform2i(&mut self, location: i32, x: i32, y: i32);
    fn Uniform3i(&mut self, location: i32, x: i32, y: i32, z: i32);
    fn Uniform4i(&mut self, location: i32, x: i32, y: i32, z: i32, w: i32);
    fn Uniform1uiv(&mut self, location: i32, values: &[u32]);
    fn Uniform2uiv(&mut self, location: i32, values: &[u32]);
    fn Uniform3uiv(&mut self, location: i32, values: &[u32]);
    fn Uniform4uiv(&mut self, location: i32, values: &[u32]);
    fn Uniform1ui(&mut self, location: i32, value: u32);
    fn Uniform2ui(&mut self, location: i32, x: u32, y: u32);
    fn Uniform3ui(&mut self, location: i32, x: u32, y: u32, z: u32);
    fn Uniform4ui(&mut self, location: i32, x: u32, y: u32, z: u32, w: u32);
    fn CreateBuffer(&mut self) -> u32;
    fn BindBuffer(&mut self, target: u32, buffer: u32);
    fn BufferData(&mut self, target: u32, bytes: &[u8], usage: u32);
    fn CreateTexture(&mut self) -> u32;
    fn BindTexture(&mut self, target: u32, texture: u32);
    fn TexParameteri(&mut self, target: u32, name: u32, value: i32);
    #[allow(clippy::too_many_arguments)]
    fn TexImage2D(
        &mut self,
        target: u32,
        level: i32,
        internal_format: i32,
        width: i32,
        height: i32,
        border: i32,
        format: u32,
        kind: u32,
        pixels: Option<&[u8]>,
    );
    fn UseProgram(&mut self, program: u32);
    fn AttribLocation(&mut self, program: u32, name: &str) -> i32;
    fn EnableVertexAttribArray(&mut self, index: u32);
    fn VertexAttribPointerF32(&mut self, index: u32, size: i32, stride: i32, offset: usize);
    fn VertexAttrib2fv(&mut self, index: u32, values: &[f32]);
    fn CreateVertexArray(&mut self) -> u32;
    fn BindVertexArray(&mut self, vertex_array: u32);
    fn Viewport(&mut self, x: i32, y: i32, width: i32, height: i32);
    fn ClearColor(&mut self, red: f32, green: f32, blue: f32, alpha: f32);
    fn ClearDepth(&mut self, depth: f64);
    fn ClearStencil(&mut self, stencil: i32);
    fn Enable(&mut self, capability: u32);
    fn Disable(&mut self, capability: u32);
    fn DepthFunc(&mut self, function: u32);
    fn FrontFace(&mut self, mode: u32);
    fn CullFace(&mut self, mode: u32);
    fn Scissor(&mut self, x: i32, y: i32, width: i32, height: i32);
    fn DepthMask(&mut self, enabled: bool);
    fn ColorMask(&mut self, red: bool, green: bool, blue: bool, alpha: bool);
    fn BlendEquation(&mut self, mode: u32);
    fn BlendFuncSeparate(
        &mut self,
        source_rgb: u32,
        destination_rgb: u32,
        source_alpha: u32,
        destination_alpha: u32,
    );
    fn BindDefaultFramebuffer(&mut self);
    fn DrawDefaultBuffer(&mut self);
    fn Clear(&mut self, mask: u32);
    fn DrawArrays(&mut self, mode: u32, first: i32, count: i32);
    fn DrawElements(&mut self, mode: u32, count: i32, kind: u32, offset: usize);
    fn IntegerParameter(&mut self, parameter: u32) -> i32;
    fn IntegerParameters(&mut self, parameter: u32, count: usize) -> Vec<i32>;
    fn FloatParameters(&mut self, parameter: u32, count: usize) -> Vec<f32>;
    fn Readback(&mut self) -> Result<canvas::CanvasPixelSnapshot, String>;
}
