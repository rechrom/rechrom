#![allow(non_snake_case)]

use canvas::{
    CanvasContextCreationAttributesCore, CanvasContextFactory, CanvasContextType,
    CanvasPixelSnapshot, CanvasRenderingContext,
};
use gpu_webgl::WebGlGpuContext;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub const VERTEX_SHADER: u32 = 0x8B31;
pub const FRAGMENT_SHADER: u32 = 0x8B30;
pub const COMPILE_STATUS: u32 = 0x8B81;
pub const LINK_STATUS: u32 = 0x8B82;
pub const ACTIVE_UNIFORMS: u32 = 0x8B86;
pub const ACTIVE_ATTRIBUTES: u32 = 0x8B89;
pub const ARRAY_BUFFER: u32 = 0x8892;
pub const ELEMENT_ARRAY_BUFFER: u32 = 0x8893;
pub const STREAM_DRAW: u32 = 0x88E0;
pub const STATIC_DRAW: u32 = 0x88E4;
pub const DYNAMIC_DRAW: u32 = 0x88E8;
pub const FLOAT: u32 = 0x1406;
pub const COLOR_BUFFER_BIT: u32 = 0x0000_4000;
pub const TRIANGLES: u32 = 0x0004;
pub const UNSIGNED_SHORT: u32 = 0x1403;
pub const NO_ERROR: u32 = 0;
pub const INVALID_ENUM: u32 = 0x0500;
pub const INVALID_VALUE: u32 = 0x0501;
pub const INVALID_OPERATION: u32 = 0x0502;
pub const VENDOR: u32 = 0x1F00;
pub const RENDERER: u32 = 0x1F01;
pub const VERSION: u32 = 0x1F02;
pub const SHADING_LANGUAGE_VERSION: u32 = 0x8B8C;
pub const VIEWPORT: u32 = 0x0BA2;
pub const SCISSOR_BOX: u32 = 0x0C10;
pub const ALIASED_POINT_SIZE_RANGE: u32 = 0x846D;
pub const ALIASED_LINE_WIDTH_RANGE: u32 = 0x846E;

struct Shader {
    shader_type: u32,
    source: String,
    service_id: u32,
    compiled: bool,
    info_log: String,
}

#[derive(Default)]
struct Program {
    vertex: Option<u32>,
    fragment: Option<u32>,
    service_id: u32,
    linked: bool,
    info_log: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WebGLActiveInfo {
    pub name: String,
    pub size: i32,
    pub kind: u32,
}

struct State {
    gpu: Box<dyn WebGlGpuContext>,
    shaders: HashMap<u32, Shader>,
    programs: HashMap<u32, Program>,
    buffers: HashMap<u32, u32>,
    textures: HashMap<u32, u32>,
    vertex_arrays: HashMap<u32, u32>,
    uniform_locations: HashMap<u32, (u32, i32)>,
    next_object: u32,
    bound_buffers: HashMap<u32, Option<u32>>,
    error: u32,
    lost: bool,
    content_revision: u64,
    context_type: CanvasContextType,
    creation_attributes: CanvasContextCreationAttributesCore,
}

impl State {
    fn Object(&mut self) -> u32 {
        self.next_object = self.next_object.wrapping_add(1).max(1);
        self.next_object
    }
    fn SetError(&mut self, error: u32) {
        if self.error == NO_ERROR {
            self.error = error;
        }
    }
}

/// `WebGLRenderingContextBase` state owner. The package validates WebGL
/// objects/enums and forwards only accepted commands to the GPU service.
#[derive(Clone)]
pub struct WebGLRenderingContextState(Rc<RefCell<State>>);

impl WebGLRenderingContextState {
    pub fn FromGpu(gpu: Box<dyn WebGlGpuContext>) -> Self {
        Self::FromGpuForType(gpu, CanvasContextType::WebGL1)
    }

    pub fn FromGpuForType(gpu: Box<dyn WebGlGpuContext>, context_type: CanvasContextType) -> Self {
        Self(Rc::new(RefCell::new(State {
            gpu,
            shaders: HashMap::new(),
            programs: HashMap::new(),
            buffers: HashMap::new(),
            textures: HashMap::new(),
            vertex_arrays: HashMap::new(),
            uniform_locations: HashMap::new(),
            next_object: 0,
            bound_buffers: HashMap::new(),
            error: NO_ERROR,
            lost: false,
            content_revision: 1,
            context_type,
            creation_attributes: Default::default(),
        })))
    }

    #[cfg(target_os = "macos")]
    pub fn Create(width: u32, height: u32) -> Result<Self, String> {
        Self::CreateForType(width, height, CanvasContextType::WebGL1)
    }

    #[cfg(target_os = "macos")]
    pub fn CreateForType(
        width: u32,
        height: u32,
        context_type: CanvasContextType,
    ) -> Result<Self, String> {
        let gpu: Box<dyn WebGlGpuContext> = match context_type {
            CanvasContextType::WebGL1 => Box::new(gpu_webgl::CglWebGlContext::new(width, height)?),
            CanvasContextType::WebGL2 => {
                Box::new(gpu_webgl::CglWebGlContext::new_webgl2(width, height)?)
            }
            _ => return Err("not a WebGL context type".into()),
        };
        Ok(Self::FromGpuForType(gpu, context_type))
    }

    pub fn ContextType(&self) -> CanvasContextType {
        self.0.borrow().context_type
    }
    pub fn CreationAttributes(&self) -> CanvasContextCreationAttributesCore {
        self.0.borrow().creation_attributes
    }

    pub fn CreateShader(&self, shader_type: u32) -> Option<u32> {
        if !matches!(shader_type, VERTEX_SHADER | FRAGMENT_SHADER) {
            self.0.borrow_mut().SetError(INVALID_ENUM);
            return None;
        }
        let mut state = self.0.borrow_mut();
        let id = state.Object();
        state.shaders.insert(
            id,
            Shader {
                shader_type,
                source: String::new(),
                service_id: 0,
                compiled: false,
                info_log: String::new(),
            },
        );
        Some(id)
    }

    pub fn ShaderSource(&self, shader: u32, source: String) {
        let mut state = self.0.borrow_mut();
        let Some(shader) = state.shaders.get_mut(&shader) else {
            state.SetError(INVALID_VALUE);
            return;
        };
        shader.source = source;
    }

    pub fn CompileShader(&self, shader: u32) {
        let mut state = self.0.borrow_mut();
        let Some((shader_type, source)) = state
            .shaders
            .get(&shader)
            .map(|s| (s.shader_type, s.source.clone()))
        else {
            state.SetError(INVALID_VALUE);
            return;
        };
        let result = state.gpu.CompileShader(shader_type, &source);
        let shader = state.shaders.get_mut(&shader).unwrap();
        shader.service_id = result.shader;
        shader.compiled = result.compiled;
        shader.info_log = result.info_log;
    }

    pub fn ShaderParameter(&self, shader: u32, parameter: u32) -> bool {
        if parameter != COMPILE_STATUS {
            self.0.borrow_mut().SetError(INVALID_ENUM);
            return false;
        }
        self.0
            .borrow()
            .shaders
            .get(&shader)
            .is_some_and(|s| s.compiled)
    }

    pub fn ShaderInfoLog(&self, shader: u32) -> String {
        self.0
            .borrow()
            .shaders
            .get(&shader)
            .map(|s| s.info_log.clone())
            .unwrap_or_default()
    }

    pub fn CreateProgram(&self) -> u32 {
        let mut state = self.0.borrow_mut();
        let id = state.Object();
        state.programs.insert(id, Program::default());
        id
    }

    pub fn AttachShader(&self, program: u32, shader: u32) {
        let mut state = self.0.borrow_mut();
        let Some(shader_type) = state.shaders.get(&shader).map(|shader| shader.shader_type) else {
            state.SetError(INVALID_VALUE);
            return;
        };
        let Some(program) = state.programs.get_mut(&program) else {
            state.SetError(INVALID_VALUE);
            return;
        };
        match shader_type {
            VERTEX_SHADER => program.vertex = Some(shader),
            FRAGMENT_SHADER => program.fragment = Some(shader),
            _ => unreachable!(),
        }
    }

    pub fn LinkProgram(&self, program: u32) {
        let mut state = self.0.borrow_mut();
        let Some((vertex, fragment)) = state
            .programs
            .get(&program)
            .and_then(|p| Some((p.vertex?, p.fragment?)))
        else {
            state.SetError(INVALID_OPERATION);
            return;
        };
        let Some(vertex) = state
            .shaders
            .get(&vertex)
            .filter(|s| s.compiled)
            .map(|s| s.service_id)
        else {
            state.SetError(INVALID_OPERATION);
            return;
        };
        let Some(fragment) = state
            .shaders
            .get(&fragment)
            .filter(|s| s.compiled)
            .map(|s| s.service_id)
        else {
            state.SetError(INVALID_OPERATION);
            return;
        };
        let result = state.gpu.LinkProgram(vertex, fragment);
        let program = state.programs.get_mut(&program).unwrap();
        program.service_id = result.program;
        program.linked = result.linked;
        program.info_log = result.info_log;
    }

    pub fn ProgramParameter(&self, program: u32, parameter: u32) -> bool {
        if parameter != LINK_STATUS {
            self.0.borrow_mut().SetError(INVALID_ENUM);
            return false;
        }
        self.0
            .borrow()
            .programs
            .get(&program)
            .is_some_and(|p| p.linked)
    }

    pub fn ProgramParameterInteger(&self, program: u32, parameter: u32) -> i32 {
        let mut state = self.0.borrow_mut();
        let Some(service_id) = state
            .programs
            .get(&program)
            .filter(|program| program.linked)
            .map(|program| program.service_id)
        else {
            state.SetError(INVALID_VALUE);
            return 0;
        };
        state.gpu.ProgramParameter(service_id, parameter)
    }

    pub fn ProgramInfoLog(&self, program: u32) -> String {
        self.0
            .borrow()
            .programs
            .get(&program)
            .map(|p| p.info_log.clone())
            .unwrap_or_default()
    }

    pub fn DeleteShader(&self, shader: u32) {
        let mut state = self.0.borrow_mut();
        let Some(shader) = state.shaders.remove(&shader) else {
            return;
        };
        if shader.service_id != 0 {
            state.gpu.DeleteShader(shader.service_id);
        }
    }

    pub fn ActiveUniform(&self, program: u32, index: u32) -> Option<WebGLActiveInfo> {
        let mut state = self.0.borrow_mut();
        let service_id = state
            .programs
            .get(&program)
            .filter(|program| program.linked)
            .map(|program| program.service_id)?;
        state
            .gpu
            .ActiveUniform(service_id, index)
            .map(|info| WebGLActiveInfo {
                name: info.name,
                size: info.size,
                kind: info.kind,
            })
    }

    pub fn ActiveAttrib(&self, program: u32, index: u32) -> Option<WebGLActiveInfo> {
        let mut state = self.0.borrow_mut();
        let service_id = state
            .programs
            .get(&program)
            .filter(|program| program.linked)
            .map(|program| program.service_id)?;
        state
            .gpu
            .ActiveAttrib(service_id, index)
            .map(|info| WebGLActiveInfo {
                name: info.name,
                size: info.size,
                kind: info.kind,
            })
    }

    pub fn GetUniformLocation(&self, program: u32, name: &str) -> Option<u32> {
        let mut state = self.0.borrow_mut();
        let service_id = state
            .programs
            .get(&program)
            .filter(|program| program.linked)
            .map(|program| program.service_id)?;
        let location = state.gpu.UniformLocation(service_id, name);
        if location < 0 {
            return None;
        }
        let id = state.Object();
        state.uniform_locations.insert(id, (program, location));
        Some(id)
    }

    fn UniformLocation(&self, location: u32) -> Option<i32> {
        let mut state = self.0.borrow_mut();
        let Some((_, service_location)) = state.uniform_locations.get(&location).copied() else {
            state.SetError(INVALID_VALUE);
            return None;
        };
        Some(service_location)
    }

    pub fn UniformMatrix2fv(&self, location: u32, transpose: bool, values: &[f32]) {
        if transpose || values.len() % 4 != 0 {
            self.0.borrow_mut().SetError(INVALID_VALUE);
            return;
        }
        let Some(location) = self.UniformLocation(location) else {
            return;
        };
        self.0
            .borrow_mut()
            .gpu
            .UniformMatrix2fv(location, false, values);
    }

    pub fn UniformMatrix4fv(&self, location: u32, transpose: bool, values: &[f32]) {
        if transpose || values.len() % 16 != 0 {
            self.0.borrow_mut().SetError(INVALID_VALUE);
            return;
        }
        let Some(location) = self.UniformLocation(location) else {
            return;
        };
        self.0
            .borrow_mut()
            .gpu
            .UniformMatrix4fv(location, false, values);
    }

    pub fn UniformMatrix3fv(&self, location: u32, transpose: bool, values: &[f32]) {
        if transpose || values.len() % 9 != 0 {
            self.0.borrow_mut().SetError(INVALID_VALUE);
            return;
        }
        let Some(location) = self.UniformLocation(location) else {
            return;
        };
        self.0
            .borrow_mut()
            .gpu
            .UniformMatrix3fv(location, false, values);
    }

    fn UniformFloatVector(&self, location: u32, components: usize, values: &[f32]) {
        if values.len() % components != 0 {
            self.0.borrow_mut().SetError(INVALID_VALUE);
            return;
        }
        let Some(location) = self.UniformLocation(location) else {
            return;
        };
        let mut state = self.0.borrow_mut();
        match components {
            1 => state.gpu.Uniform1fv(location, values),
            2 => state.gpu.Uniform2fv(location, values),
            3 => state.gpu.Uniform3fv(location, values),
            4 => state.gpu.Uniform4fv(location, values),
            _ => unreachable!(),
        }
    }

    pub fn Uniform1fv(&self, location: u32, values: &[f32]) {
        self.UniformFloatVector(location, 1, values);
    }

    pub fn Uniform2fv(&self, location: u32, values: &[f32]) {
        self.UniformFloatVector(location, 2, values);
    }

    pub fn Uniform3fv(&self, location: u32, values: &[f32]) {
        self.UniformFloatVector(location, 3, values);
    }

    pub fn Uniform4fv(&self, location: u32, values: &[f32]) {
        self.UniformFloatVector(location, 4, values);
    }

    pub fn Uniform2f(&self, location: u32, x: f32, y: f32) {
        let Some(location) = self.UniformLocation(location) else {
            return;
        };
        self.0.borrow_mut().gpu.Uniform2f(location, x, y);
    }

    pub fn Uniform3f(&self, location: u32, x: f32, y: f32, z: f32) {
        let Some(location) = self.UniformLocation(location) else {
            return;
        };
        self.0.borrow_mut().gpu.Uniform3f(location, x, y, z);
    }

    pub fn Uniform4f(&self, location: u32, x: f32, y: f32, z: f32, w: f32) {
        let Some(location) = self.UniformLocation(location) else {
            return;
        };
        self.0.borrow_mut().gpu.Uniform4f(location, x, y, z, w);
    }

    pub fn Uniform1f(&self, location: u32, value: f32) {
        let Some(location) = self.UniformLocation(location) else {
            return;
        };
        self.0.borrow_mut().gpu.Uniform1f(location, value);
    }

    pub fn Uniform1i(&self, location: u32, value: i32) {
        let Some(location) = self.UniformLocation(location) else {
            return;
        };
        self.0.borrow_mut().gpu.Uniform1i(location, value);
    }

    fn UniformIntegerVector(&self, location: u32, components: usize, values: &[i32]) {
        if values.len() % components != 0 {
            self.0.borrow_mut().SetError(INVALID_VALUE);
            return;
        }
        let Some(location) = self.UniformLocation(location) else {
            return;
        };
        let mut state = self.0.borrow_mut();
        match components {
            1 => state.gpu.Uniform1iv(location, values),
            2 => state.gpu.Uniform2iv(location, values),
            3 => state.gpu.Uniform3iv(location, values),
            4 => state.gpu.Uniform4iv(location, values),
            _ => unreachable!(),
        }
    }

    pub fn Uniform1iv(&self, location: u32, values: &[i32]) {
        self.UniformIntegerVector(location, 1, values);
    }

    pub fn Uniform2iv(&self, location: u32, values: &[i32]) {
        self.UniformIntegerVector(location, 2, values);
    }

    pub fn Uniform3iv(&self, location: u32, values: &[i32]) {
        self.UniformIntegerVector(location, 3, values);
    }

    pub fn Uniform4iv(&self, location: u32, values: &[i32]) {
        self.UniformIntegerVector(location, 4, values);
    }

    pub fn Uniform2i(&self, location: u32, x: i32, y: i32) {
        let Some(location) = self.UniformLocation(location) else {
            return;
        };
        self.0.borrow_mut().gpu.Uniform2i(location, x, y);
    }

    pub fn Uniform3i(&self, location: u32, x: i32, y: i32, z: i32) {
        let Some(location) = self.UniformLocation(location) else {
            return;
        };
        self.0.borrow_mut().gpu.Uniform3i(location, x, y, z);
    }

    pub fn Uniform4i(&self, location: u32, x: i32, y: i32, z: i32, w: i32) {
        let Some(location) = self.UniformLocation(location) else {
            return;
        };
        self.0.borrow_mut().gpu.Uniform4i(location, x, y, z, w);
    }

    fn UniformUnsignedIntegerVector(&self, location: u32, components: usize, values: &[u32]) {
        if self.ContextType() != CanvasContextType::WebGL2 {
            self.0.borrow_mut().SetError(INVALID_OPERATION);
            return;
        }
        if values.len() % components != 0 {
            self.0.borrow_mut().SetError(INVALID_VALUE);
            return;
        }
        let Some(location) = self.UniformLocation(location) else {
            return;
        };
        let mut state = self.0.borrow_mut();
        match components {
            1 => state.gpu.Uniform1uiv(location, values),
            2 => state.gpu.Uniform2uiv(location, values),
            3 => state.gpu.Uniform3uiv(location, values),
            4 => state.gpu.Uniform4uiv(location, values),
            _ => unreachable!(),
        }
    }

    pub fn Uniform1uiv(&self, location: u32, values: &[u32]) {
        self.UniformUnsignedIntegerVector(location, 1, values);
    }

    pub fn Uniform2uiv(&self, location: u32, values: &[u32]) {
        self.UniformUnsignedIntegerVector(location, 2, values);
    }

    pub fn Uniform3uiv(&self, location: u32, values: &[u32]) {
        self.UniformUnsignedIntegerVector(location, 3, values);
    }

    pub fn Uniform4uiv(&self, location: u32, values: &[u32]) {
        self.UniformUnsignedIntegerVector(location, 4, values);
    }

    pub fn Uniform1ui(&self, location: u32, value: u32) {
        let Some(location) = self.UniformLocation(location) else {
            return;
        };
        self.0.borrow_mut().gpu.Uniform1ui(location, value);
    }

    pub fn Uniform2ui(&self, location: u32, x: u32, y: u32) {
        let Some(location) = self.UniformLocation(location) else {
            return;
        };
        self.0.borrow_mut().gpu.Uniform2ui(location, x, y);
    }

    pub fn Uniform3ui(&self, location: u32, x: u32, y: u32, z: u32) {
        let Some(location) = self.UniformLocation(location) else {
            return;
        };
        self.0.borrow_mut().gpu.Uniform3ui(location, x, y, z);
    }

    pub fn Uniform4ui(&self, location: u32, x: u32, y: u32, z: u32, w: u32) {
        let Some(location) = self.UniformLocation(location) else {
            return;
        };
        self.0.borrow_mut().gpu.Uniform4ui(location, x, y, z, w);
    }

    pub fn UseProgram(&self, program: u32) {
        let mut state = self.0.borrow_mut();
        let Some(service_id) = state
            .programs
            .get(&program)
            .filter(|p| p.linked)
            .map(|p| p.service_id)
        else {
            state.SetError(INVALID_OPERATION);
            return;
        };
        state.gpu.UseProgram(service_id);
    }

    pub fn GetAttribLocation(&self, program: u32, name: &str) -> i32 {
        let mut state = self.0.borrow_mut();
        let Some(service_id) = state
            .programs
            .get(&program)
            .filter(|p| p.linked)
            .map(|p| p.service_id)
        else {
            state.SetError(INVALID_OPERATION);
            return -1;
        };
        state.gpu.AttribLocation(service_id, name)
    }

    pub fn CreateBuffer(&self) -> u32 {
        let mut state = self.0.borrow_mut();
        let id = state.Object();
        let service_id = state.gpu.CreateBuffer();
        state.buffers.insert(id, service_id);
        id
    }

    pub fn BindBuffer(&self, target: u32, buffer: Option<u32>) {
        let mut state = self.0.borrow_mut();
        if !matches!(target, ARRAY_BUFFER | ELEMENT_ARRAY_BUFFER) {
            state.SetError(INVALID_ENUM);
            return;
        }
        if buffer.is_some_and(|id| !state.buffers.contains_key(&id)) {
            state.SetError(INVALID_VALUE);
            return;
        }
        let service_id = buffer
            .and_then(|id| state.buffers.get(&id).copied())
            .unwrap_or(0);
        state.gpu.BindBuffer(target, service_id);
        state.bound_buffers.insert(target, buffer);
    }

    pub fn BufferData(&self, target: u32, bytes: &[u8], usage: u32) {
        let mut state = self.0.borrow_mut();
        if !matches!(target, ARRAY_BUFFER | ELEMENT_ARRAY_BUFFER) {
            state.SetError(INVALID_ENUM);
            return;
        }
        if !matches!(usage, STREAM_DRAW | STATIC_DRAW | DYNAMIC_DRAW) {
            state.SetError(INVALID_ENUM);
            return;
        }
        if state
            .bound_buffers
            .get(&target)
            .copied()
            .flatten()
            .is_none()
        {
            state.SetError(INVALID_OPERATION);
            return;
        }
        state.gpu.BufferData(target, bytes, usage);
    }

    pub fn CreateTexture(&self) -> u32 {
        let mut state = self.0.borrow_mut();
        let id = state.Object();
        let service_id = state.gpu.CreateTexture();
        state.textures.insert(id, service_id);
        id
    }

    pub fn BindTexture(&self, target: u32, texture: Option<u32>) {
        let mut state = self.0.borrow_mut();
        let Some(service_id) = texture
            .map(|id| state.textures.get(&id).copied())
            .unwrap_or(Some(0))
        else {
            state.SetError(INVALID_VALUE);
            return;
        };
        state.gpu.BindTexture(target, service_id);
    }

    pub fn TexParameteri(&self, target: u32, name: u32, value: i32) {
        self.0.borrow_mut().gpu.TexParameteri(target, name, value);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn TexImage2D(
        &self,
        target: u32,
        level: i32,
        internal_format: i32,
        width: i32,
        height: i32,
        border: i32,
        format: u32,
        kind: u32,
        pixels: Option<&[u8]>,
    ) {
        if width < 0 || height < 0 || border != 0 {
            self.0.borrow_mut().SetError(INVALID_VALUE);
            return;
        }
        self.0.borrow_mut().gpu.TexImage2D(
            target,
            level,
            internal_format,
            width,
            height,
            border,
            format,
            kind,
            pixels,
        );
    }

    pub fn EnableVertexAttribArray(&self, index: u32) {
        self.0.borrow_mut().gpu.EnableVertexAttribArray(index);
    }
    pub fn VertexAttribPointer(
        &self,
        index: u32,
        size: i32,
        ty: u32,
        normalized: bool,
        stride: i32,
        offset: usize,
    ) {
        let mut state = self.0.borrow_mut();
        if ty != FLOAT || normalized || !(1..=4).contains(&size) || stride < 0 {
            state.SetError(INVALID_VALUE);
            return;
        }
        state
            .gpu
            .VertexAttribPointerF32(index, size, stride, offset);
    }
    pub fn VertexAttrib2fv(&self, index: u32, values: &[f32]) {
        if values.len() < 2 {
            self.0.borrow_mut().SetError(INVALID_VALUE);
            return;
        }
        self.0.borrow_mut().gpu.VertexAttrib2fv(index, values);
    }
    pub fn CreateVertexArray(&self) -> u32 {
        let mut state = self.0.borrow_mut();
        let id = state.Object();
        let service_id = state.gpu.CreateVertexArray();
        state.vertex_arrays.insert(id, service_id);
        id
    }
    pub fn BindVertexArray(&self, vertex_array: Option<u32>) {
        let mut state = self.0.borrow_mut();
        let Some(service_id) = vertex_array
            .map(|id| state.vertex_arrays.get(&id).copied())
            .unwrap_or(Some(0))
        else {
            state.SetError(INVALID_VALUE);
            return;
        };
        state.gpu.BindVertexArray(service_id);
    }
    pub fn Viewport(&self, x: i32, y: i32, width: i32, height: i32) {
        self.0.borrow_mut().gpu.Viewport(x, y, width, height);
    }
    pub fn ClearColor(&self, r: f32, g: f32, b: f32, a: f32) {
        self.0.borrow_mut().gpu.ClearColor(r, g, b, a);
    }
    pub fn ClearDepth(&self, depth: f64) {
        self.0.borrow_mut().gpu.ClearDepth(depth.clamp(0.0, 1.0));
    }
    pub fn ClearStencil(&self, stencil: i32) {
        self.0.borrow_mut().gpu.ClearStencil(stencil);
    }
    pub fn Enable(&self, capability: u32) {
        self.0.borrow_mut().gpu.Enable(capability);
    }
    pub fn Disable(&self, capability: u32) {
        self.0.borrow_mut().gpu.Disable(capability);
    }
    pub fn DepthFunc(&self, function: u32) {
        self.0.borrow_mut().gpu.DepthFunc(function);
    }
    pub fn FrontFace(&self, mode: u32) {
        self.0.borrow_mut().gpu.FrontFace(mode);
    }
    pub fn CullFace(&self, mode: u32) {
        self.0.borrow_mut().gpu.CullFace(mode);
    }
    pub fn Scissor(&self, x: i32, y: i32, width: i32, height: i32) {
        self.0.borrow_mut().gpu.Scissor(x, y, width, height);
    }
    pub fn DepthMask(&self, enabled: bool) {
        self.0.borrow_mut().gpu.DepthMask(enabled);
    }
    pub fn ColorMask(&self, red: bool, green: bool, blue: bool, alpha: bool) {
        self.0.borrow_mut().gpu.ColorMask(red, green, blue, alpha);
    }
    pub fn BlendEquation(&self, mode: u32) {
        self.0.borrow_mut().gpu.BlendEquation(mode);
    }
    pub fn BlendFuncSeparate(
        &self,
        source_rgb: u32,
        destination_rgb: u32,
        source_alpha: u32,
        destination_alpha: u32,
    ) {
        self.0.borrow_mut().gpu.BlendFuncSeparate(
            source_rgb,
            destination_rgb,
            source_alpha,
            destination_alpha,
        );
    }
    pub fn BindDefaultFramebuffer(&self) {
        self.0.borrow_mut().gpu.BindDefaultFramebuffer();
    }
    pub fn DrawDefaultBuffer(&self) {
        self.0.borrow_mut().gpu.DrawDefaultBuffer();
    }
    pub fn Clear(&self, mask: u32) {
        let mut state = self.0.borrow_mut();
        state.gpu.Clear(mask);
        state.content_revision = state.content_revision.wrapping_add(1).max(1);
    }
    pub fn DrawArrays(&self, mode: u32, first: i32, count: i32) {
        if mode != TRIANGLES {
            self.0.borrow_mut().SetError(INVALID_ENUM);
            return;
        }
        if first < 0 || count < 0 {
            self.0.borrow_mut().SetError(INVALID_VALUE);
            return;
        }
        let mut state = self.0.borrow_mut();
        state.gpu.DrawArrays(mode, first, count);
        state.content_revision = state.content_revision.wrapping_add(1).max(1);
    }
    pub fn DrawElements(&self, mode: u32, count: i32, kind: u32, offset: usize) {
        if mode != TRIANGLES {
            self.0.borrow_mut().SetError(INVALID_ENUM);
            return;
        }
        if count < 0 || !matches!(kind, UNSIGNED_SHORT) {
            self.0.borrow_mut().SetError(INVALID_VALUE);
            return;
        }
        let mut state = self.0.borrow_mut();
        state.gpu.DrawElements(mode, count, kind, offset);
        state.content_revision = state.content_revision.wrapping_add(1).max(1);
    }
    pub fn IntegerParameter(&self, parameter: u32) -> i32 {
        self.0.borrow_mut().gpu.IntegerParameter(parameter)
    }
    pub fn IntegerParameters(&self, parameter: u32, count: usize) -> Vec<i32> {
        self.0.borrow_mut().gpu.IntegerParameters(parameter, count)
    }
    pub fn FloatParameters(&self, parameter: u32, count: usize) -> Vec<f32> {
        self.0.borrow_mut().gpu.FloatParameters(parameter, count)
    }
    pub fn GetError(&self) -> u32 {
        let mut state = self.0.borrow_mut();
        let error = state.error;
        state.error = NO_ERROR;
        error
    }
    pub fn Snapshot(&self) -> Option<CanvasPixelSnapshot> {
        self.0.borrow_mut().gpu.Readback().ok()
    }
}

struct CanvasWebGLContext {
    state: WebGLRenderingContextState,
}
impl CanvasRenderingContext for CanvasWebGLContext {
    fn GetContextType(&self) -> CanvasContextType {
        self.state.ContextType()
    }
    fn CreationAttributes(&self) -> CanvasContextCreationAttributesCore {
        self.state.CreationAttributes()
    }
    fn Reshape(&mut self, width: u32, height: u32) {
        let mut state = self.state.0.borrow_mut();
        if state.gpu.Reshape(width, height).is_err() {
            state.lost = true;
        } else {
            state.content_revision = state.content_revision.wrapping_add(1).max(1);
        }
    }
    fn IsContextLost(&self) -> bool {
        self.state.0.borrow().lost
    }
    fn ContentRevision(&self) -> u64 {
        self.state.0.borrow().content_revision
    }
    fn PaintRenderingResultsToCanvas(&mut self) -> Option<CanvasPixelSnapshot> {
        self.state.Snapshot()
    }
}

#[derive(Default)]
pub struct WebGLContextFactory {
    created: Option<WebGLRenderingContextState>,
}
impl WebGLContextFactory {
    pub fn TakeCreated(&mut self) -> Option<WebGLRenderingContextState> {
        self.created.take()
    }
}
impl CanvasContextFactory for WebGLContextFactory {
    fn Create(
        &mut self,
        context_type: CanvasContextType,
        mut attributes: CanvasContextCreationAttributesCore,
        width: u32,
        height: u32,
    ) -> Option<Box<dyn CanvasRenderingContext>> {
        if !matches!(
            context_type,
            CanvasContextType::WebGL1 | CanvasContextType::WebGL2
        ) {
            return None;
        }
        #[cfg(target_os = "macos")]
        {
            let state =
                WebGLRenderingContextState::CreateForType(width, height, context_type).ok()?;
            // CGL currently publishes readback snapshots rather than an
            // independently presented low-latency surface. Context admission
            // therefore disables the effective mode (core attributes h:34-39).
            attributes.desynchronized &= state.0.borrow().gpu.SupportsLowLatencyPresentation();
            state.0.borrow_mut().creation_attributes = attributes;
            self.created = Some(state.clone());
            Some(Box::new(CanvasWebGLContext { state }))
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = (width, height, attributes);
            None
        }
    }
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;

    #[test]
    fn real_cgl_triangle_reaches_canvas_snapshot() {
        let context = WebGLRenderingContextState::Create(32, 32).unwrap();
        let vertex = context.CreateShader(VERTEX_SHADER).unwrap();
        context.ShaderSource(
            vertex,
            "attribute vec2 position; void main(){gl_Position=vec4(position,0.0,1.0);}".into(),
        );
        context.CompileShader(vertex);
        assert!(
            context.ShaderParameter(vertex, COMPILE_STATUS),
            "{}",
            context.ShaderInfoLog(vertex)
        );
        let fragment = context.CreateShader(FRAGMENT_SHADER).unwrap();
        context.ShaderSource(
            fragment,
            "void main(){gl_FragColor=vec4(1.0,0.0,0.0,1.0);}".into(),
        );
        context.CompileShader(fragment);
        assert!(
            context.ShaderParameter(fragment, COMPILE_STATUS),
            "{}",
            context.ShaderInfoLog(fragment)
        );
        let program = context.CreateProgram();
        context.AttachShader(program, vertex);
        context.AttachShader(program, fragment);
        context.LinkProgram(program);
        assert!(
            context.ProgramParameter(program, LINK_STATUS),
            "{}",
            context.ProgramInfoLog(program)
        );
        context.UseProgram(program);
        let buffer = context.CreateBuffer();
        context.BindBuffer(ARRAY_BUFFER, Some(buffer));
        let values = [-1.0_f32, -1.0, 1.0, -1.0, 0.0, 1.0];
        let bytes = values
            .iter()
            .flat_map(|value| value.to_ne_bytes())
            .collect::<Vec<_>>();
        context.BufferData(ARRAY_BUFFER, &bytes, STATIC_DRAW);
        let position = context.GetAttribLocation(program, "position");
        assert!(position >= 0);
        context.EnableVertexAttribArray(position as u32);
        context.VertexAttribPointer(position as u32, 2, FLOAT, false, 0, 0);
        context.Viewport(0, 0, 32, 32);
        context.ClearColor(0.0, 0.0, 0.0, 0.0);
        context.Clear(COLOR_BUFFER_BIT);
        context.DrawArrays(TRIANGLES, 0, 3);
        let snapshot = context.Snapshot().unwrap();
        let center = (16 * snapshot.row_bytes) + 16 * 4;
        assert!(snapshot.pixels[center] > 240 && snapshot.pixels[center + 3] > 240);
    }
}
