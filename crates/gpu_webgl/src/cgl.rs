use crate::{ActiveInfo, ProgramLinkResult, ShaderCompileResult, WebGlGpuContext};
use canvas::CanvasPixelSnapshot;
use core::ffi::{c_char, c_int, c_uint, c_void};
use std::ffi::{CStr, CString};
use std::ptr;

type CGLContextObj = *mut c_void;
type CGLPixelFormatObj = *mut c_void;
type GLenum = c_uint;
type GLuint = c_uint;
type GLint = c_int;
type GLsizei = c_int;
type GLsizeiptr = isize;
type GLboolean = u8;
type GLfloat = f32;

const GL_TEXTURE_2D: GLenum = 0x0DE1;
const GL_RGBA: GLenum = 0x1908;
const GL_UNSIGNED_BYTE: GLenum = 0x1401;
const GL_TEXTURE_MIN_FILTER: GLenum = 0x2801;
const GL_TEXTURE_MAG_FILTER: GLenum = 0x2800;
const GL_NEAREST: GLint = 0x2600;
const GL_FRAMEBUFFER: GLenum = 0x8D40;
const GL_COLOR_ATTACHMENT0: GLenum = 0x8CE0;
const GL_FRAMEBUFFER_COMPLETE: GLenum = 0x8CD5;
const GL_RENDERBUFFER: GLenum = 0x8D41;
const GL_DEPTH24_STENCIL8: GLenum = 0x88F0;
const GL_DEPTH_STENCIL_ATTACHMENT: GLenum = 0x821A;
const GL_BACK: GLenum = 0x0405;
const GL_COMPILE_STATUS: GLenum = 0x8B81;
const GL_LINK_STATUS: GLenum = 0x8B82;
const GL_INFO_LOG_LENGTH: GLenum = 0x8B84;
const GL_ACTIVE_UNIFORMS: GLenum = 0x8B86;
const GL_ACTIVE_UNIFORM_MAX_LENGTH: GLenum = 0x8B87;
const GL_ACTIVE_ATTRIBUTES: GLenum = 0x8B89;
const GL_ACTIVE_ATTRIBUTE_MAX_LENGTH: GLenum = 0x8B8A;
const GL_ARRAY_BUFFER: GLenum = 0x8892;
const GL_FLOAT: GLenum = 0x1406;
const K_CGL_PFA_OPENGL_PROFILE: c_int = 99;
const K_CGL_OGLP_VERSION_4_1_CORE: c_int = 0x4100;

#[link(name = "OpenGL", kind = "framework")]
unsafe extern "C" {
    fn CGLChoosePixelFormat(
        attributes: *const c_int,
        pixel_format: *mut CGLPixelFormatObj,
        count: *mut c_int,
    ) -> c_int;
    fn CGLCreateContext(
        pixel_format: CGLPixelFormatObj,
        share: CGLContextObj,
        context: *mut CGLContextObj,
    ) -> c_int;
    fn CGLDestroyContext(context: CGLContextObj) -> c_int;
    fn CGLReleasePixelFormat(pixel_format: CGLPixelFormatObj) -> c_int;
    fn CGLSetCurrentContext(context: CGLContextObj) -> c_int;

    fn glGenTextures(count: GLsizei, textures: *mut GLuint);
    fn glDeleteTextures(count: GLsizei, textures: *const GLuint);
    fn glBindTexture(target: GLenum, texture: GLuint);
    fn glTexParameteri(target: GLenum, name: GLenum, value: GLint);
    fn glTexImage2D(
        target: GLenum,
        level: GLint,
        internal_format: GLint,
        width: GLsizei,
        height: GLsizei,
        border: GLint,
        format: GLenum,
        ty: GLenum,
        pixels: *const c_void,
    );
    fn glGenFramebuffers(count: GLsizei, framebuffers: *mut GLuint);
    fn glDeleteFramebuffers(count: GLsizei, framebuffers: *const GLuint);
    fn glBindFramebuffer(target: GLenum, framebuffer: GLuint);
    fn glFramebufferTexture2D(
        target: GLenum,
        attachment: GLenum,
        texture_target: GLenum,
        texture: GLuint,
        level: GLint,
    );
    fn glCheckFramebufferStatus(target: GLenum) -> GLenum;
    fn glGenRenderbuffers(count: GLsizei, renderbuffers: *mut GLuint);
    fn glDeleteRenderbuffers(count: GLsizei, renderbuffers: *const GLuint);
    fn glBindRenderbuffer(target: GLenum, renderbuffer: GLuint);
    fn glRenderbufferStorage(
        target: GLenum,
        internal_format: GLenum,
        width: GLsizei,
        height: GLsizei,
    );
    fn glFramebufferRenderbuffer(
        target: GLenum,
        attachment: GLenum,
        renderbuffer_target: GLenum,
        renderbuffer: GLuint,
    );
    fn glCreateShader(shader_type: GLenum) -> GLuint;
    fn glShaderSource(
        shader: GLuint,
        count: GLsizei,
        strings: *const *const c_char,
        lengths: *const GLint,
    );
    fn glCompileShader(shader: GLuint);
    fn glGetShaderiv(shader: GLuint, name: GLenum, value: *mut GLint);
    fn glGetShaderInfoLog(
        shader: GLuint,
        buffer_size: GLsizei,
        length: *mut GLsizei,
        log: *mut c_char,
    );
    fn glDeleteShader(shader: GLuint);
    fn glCreateProgram() -> GLuint;
    fn glAttachShader(program: GLuint, shader: GLuint);
    fn glLinkProgram(program: GLuint);
    fn glGetProgramiv(program: GLuint, name: GLenum, value: *mut GLint);
    fn glGetProgramInfoLog(
        program: GLuint,
        buffer_size: GLsizei,
        length: *mut GLsizei,
        log: *mut c_char,
    );
    fn glGetActiveUniform(
        program: GLuint,
        index: GLuint,
        buffer_size: GLsizei,
        length: *mut GLsizei,
        size: *mut GLint,
        kind: *mut GLenum,
        name: *mut c_char,
    );
    fn glGetActiveAttrib(
        program: GLuint,
        index: GLuint,
        buffer_size: GLsizei,
        length: *mut GLsizei,
        size: *mut GLint,
        kind: *mut GLenum,
        name: *mut c_char,
    );
    fn glGetUniformLocation(program: GLuint, name: *const c_char) -> GLint;
    fn glUniformMatrix2fv(
        location: GLint,
        count: GLsizei,
        transpose: GLboolean,
        values: *const GLfloat,
    );
    fn glUniformMatrix4fv(
        location: GLint,
        count: GLsizei,
        transpose: GLboolean,
        values: *const GLfloat,
    );
    fn glUniformMatrix3fv(
        location: GLint,
        count: GLsizei,
        transpose: GLboolean,
        values: *const GLfloat,
    );
    fn glUniform1fv(location: GLint, count: GLsizei, values: *const GLfloat);
    fn glUniform2fv(location: GLint, count: GLsizei, values: *const GLfloat);
    fn glUniform3fv(location: GLint, count: GLsizei, values: *const GLfloat);
    fn glUniform4fv(location: GLint, count: GLsizei, values: *const GLfloat);
    fn glUniform2f(location: GLint, x: GLfloat, y: GLfloat);
    fn glUniform3f(location: GLint, x: GLfloat, y: GLfloat, z: GLfloat);
    fn glUniform4f(location: GLint, x: GLfloat, y: GLfloat, z: GLfloat, w: GLfloat);
    fn glUniform1f(location: GLint, value: GLfloat);
    fn glUniform1iv(location: GLint, count: GLsizei, values: *const GLint);
    fn glUniform2iv(location: GLint, count: GLsizei, values: *const GLint);
    fn glUniform3iv(location: GLint, count: GLsizei, values: *const GLint);
    fn glUniform4iv(location: GLint, count: GLsizei, values: *const GLint);
    fn glUniform1i(location: GLint, value: GLint);
    fn glUniform2i(location: GLint, x: GLint, y: GLint);
    fn glUniform3i(location: GLint, x: GLint, y: GLint, z: GLint);
    fn glUniform4i(location: GLint, x: GLint, y: GLint, z: GLint, w: GLint);
    fn glUniform1uiv(location: GLint, count: GLsizei, values: *const GLuint);
    fn glUniform2uiv(location: GLint, count: GLsizei, values: *const GLuint);
    fn glUniform3uiv(location: GLint, count: GLsizei, values: *const GLuint);
    fn glUniform4uiv(location: GLint, count: GLsizei, values: *const GLuint);
    fn glUniform1ui(location: GLint, value: GLuint);
    fn glUniform2ui(location: GLint, x: GLuint, y: GLuint);
    fn glUniform3ui(location: GLint, x: GLuint, y: GLuint, z: GLuint);
    fn glUniform4ui(location: GLint, x: GLuint, y: GLuint, z: GLuint, w: GLuint);
    fn glUseProgram(program: GLuint);
    fn glGetAttribLocation(program: GLuint, name: *const c_char) -> GLint;
    fn glGenBuffers(count: GLsizei, buffers: *mut GLuint);
    fn glBindBuffer(target: GLenum, buffer: GLuint);
    fn glBufferData(target: GLenum, size: GLsizeiptr, data: *const c_void, usage: GLenum);
    fn glEnableVertexAttribArray(index: GLuint);
    fn glVertexAttribPointer(
        index: GLuint,
        size: GLint,
        ty: GLenum,
        normalized: u8,
        stride: GLsizei,
        pointer: *const c_void,
    );
    fn glVertexAttrib2fv(index: GLuint, values: *const GLfloat);
    fn glGenVertexArrays(count: GLsizei, arrays: *mut GLuint);
    fn glBindVertexArray(array: GLuint);
    fn glViewport(x: GLint, y: GLint, width: GLsizei, height: GLsizei);
    fn glClearColor(red: f32, green: f32, blue: f32, alpha: f32);
    fn glClearDepth(depth: f64);
    fn glClearStencil(stencil: GLint);
    fn glEnable(capability: GLenum);
    fn glDisable(capability: GLenum);
    fn glDepthFunc(function: GLenum);
    fn glFrontFace(mode: GLenum);
    fn glCullFace(mode: GLenum);
    fn glScissor(x: GLint, y: GLint, width: GLsizei, height: GLsizei);
    fn glDepthMask(enabled: GLboolean);
    fn glColorMask(red: GLboolean, green: GLboolean, blue: GLboolean, alpha: GLboolean);
    fn glBlendEquation(mode: GLenum);
    fn glBlendFuncSeparate(
        source_rgb: GLenum,
        destination_rgb: GLenum,
        source_alpha: GLenum,
        destination_alpha: GLenum,
    );
    fn glDrawBuffer(buffer: GLenum);
    fn glClear(mask: GLenum);
    fn glDrawArrays(mode: GLenum, first: GLint, count: GLsizei);
    fn glDrawElements(mode: GLenum, count: GLsizei, kind: GLenum, indices: *const c_void);
    fn glGetIntegerv(parameter: GLenum, value: *mut GLint);
    fn glGetFloatv(parameter: GLenum, value: *mut GLfloat);
    fn glFinish();
    fn glReadPixels(
        x: GLint,
        y: GLint,
        width: GLsizei,
        height: GLsizei,
        format: GLenum,
        ty: GLenum,
        pixels: *mut c_void,
    );
}

pub struct CglWebGlContext {
    context: CGLContextObj,
    framebuffer: GLuint,
    color_texture: GLuint,
    depth_stencil_renderbuffer: GLuint,
    width: u32,
    height: u32,
    webgl2: bool,
}

impl CglWebGlContext {
    pub fn new(width: u32, height: u32) -> Result<Self, String> {
        Self::Create(width, height, false)
    }

    pub fn new_webgl2(width: u32, height: u32) -> Result<Self, String> {
        Self::Create(width, height, true)
    }

    fn Create(width: u32, height: u32, webgl2: bool) -> Result<Self, String> {
        unsafe {
            // Source: ui/gl/gl_context_cgl.cc. An offscreen CGL context needs
            // no platform drawable; WebGL's DrawingBuffer supplies an FBO.
            let attributes = if webgl2 {
                vec![K_CGL_PFA_OPENGL_PROFILE, K_CGL_OGLP_VERSION_4_1_CORE, 0]
            } else {
                vec![0_i32]
            };
            let mut format = ptr::null_mut();
            let mut count = 0;
            let error = CGLChoosePixelFormat(attributes.as_ptr(), &mut format, &mut count);
            if error != 0 || format.is_null() || count == 0 {
                return Err(format!("CGLChoosePixelFormat failed: {error}"));
            }
            let mut context = ptr::null_mut();
            let error = CGLCreateContext(format, ptr::null_mut(), &mut context);
            CGLReleasePixelFormat(format);
            if error != 0 || context.is_null() {
                return Err(format!("CGLCreateContext failed: {error}"));
            }
            let mut result = Self {
                context,
                framebuffer: 0,
                color_texture: 0,
                depth_stencil_renderbuffer: 0,
                width: 0,
                height: 0,
                webgl2,
            };
            result.Reshape(width, height)?;
            Ok(result)
        }
    }

    fn MakeCurrent(&self) {
        unsafe {
            CGLSetCurrentContext(self.context);
        }
    }

    unsafe fn ShaderLog(shader: GLuint) -> String {
        let mut size = 0;
        unsafe { glGetShaderiv(shader, GL_INFO_LOG_LENGTH, &mut size) };
        if size <= 1 {
            return String::new();
        }
        let mut bytes = vec![0_u8; size as usize];
        unsafe { glGetShaderInfoLog(shader, size, ptr::null_mut(), bytes.as_mut_ptr().cast()) };
        unsafe { CStr::from_ptr(bytes.as_ptr().cast()) }
            .to_string_lossy()
            .into_owned()
    }

    unsafe fn ProgramLog(program: GLuint) -> String {
        let mut size = 0;
        unsafe { glGetProgramiv(program, GL_INFO_LOG_LENGTH, &mut size) };
        if size <= 1 {
            return String::new();
        }
        let mut bytes = vec![0_u8; size as usize];
        unsafe { glGetProgramInfoLog(program, size, ptr::null_mut(), bytes.as_mut_ptr().cast()) };
        unsafe { CStr::from_ptr(bytes.as_ptr().cast()) }
            .to_string_lossy()
            .into_owned()
    }

    fn ShaderSource(&self, source: &str) -> String {
        if !self.webgl2 || !source.starts_with("#version 300 es") {
            return source.into();
        }
        // ANGLE performs the ES-to-host translation in Chromium. The macOS
        // backend has a native 4.1 core context, so preserve the already
        // WebGL-validated shader and translate only GLSL ES surface syntax.
        let mut translated = String::from("#version 410 core\n");
        for line in source.lines().skip(1) {
            let line = if line.trim_start().starts_with("precision ") {
                line.split_once(';').map_or("", |(_, remainder)| remainder)
            } else {
                line
            };
            translated.push_str(
                &line
                    .replace("highp ", "")
                    .replace("mediump ", "")
                    .replace("lowp ", ""),
            );
            translated.push('\n');
        }
        translated
    }

    unsafe fn ActiveInfo(
        program: GLuint,
        index: GLuint,
        count_parameter: GLenum,
        max_length_parameter: GLenum,
        query: unsafe extern "C" fn(
            GLuint,
            GLuint,
            GLsizei,
            *mut GLsizei,
            *mut GLint,
            *mut GLenum,
            *mut c_char,
        ),
    ) -> Option<ActiveInfo> {
        let mut count = 0;
        unsafe { glGetProgramiv(program, count_parameter, &mut count) };
        if index >= count.max(0) as u32 {
            return None;
        }
        let mut max_length = 0;
        unsafe { glGetProgramiv(program, max_length_parameter, &mut max_length) };
        let mut bytes = vec![0_u8; max_length.max(1) as usize];
        let mut length = 0;
        let mut size = 0;
        let mut kind = 0;
        unsafe {
            query(
                program,
                index,
                max_length.max(1),
                &mut length,
                &mut size,
                &mut kind,
                bytes.as_mut_ptr().cast(),
            )
        };
        bytes.truncate(length.max(0) as usize);
        Some(ActiveInfo {
            name: String::from_utf8_lossy(&bytes).into_owned(),
            size,
            kind,
        })
    }
}

impl WebGlGpuContext for CglWebGlContext {
    fn Reshape(&mut self, width: u32, height: u32) -> Result<(), String> {
        self.MakeCurrent();
        unsafe {
            if self.framebuffer != 0 {
                glDeleteFramebuffers(1, &self.framebuffer);
            }
            if self.color_texture != 0 {
                glDeleteTextures(1, &self.color_texture);
            }
            if self.depth_stencil_renderbuffer != 0 {
                glDeleteRenderbuffers(1, &self.depth_stencil_renderbuffer);
            }
            glGenTextures(1, &mut self.color_texture);
            glBindTexture(GL_TEXTURE_2D, self.color_texture);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_NEAREST);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_NEAREST);
            glTexImage2D(
                GL_TEXTURE_2D,
                0,
                GL_RGBA as GLint,
                width as GLsizei,
                height as GLsizei,
                0,
                GL_RGBA,
                GL_UNSIGNED_BYTE,
                ptr::null(),
            );
            glGenFramebuffers(1, &mut self.framebuffer);
            glBindFramebuffer(GL_FRAMEBUFFER, self.framebuffer);
            glFramebufferTexture2D(
                GL_FRAMEBUFFER,
                GL_COLOR_ATTACHMENT0,
                GL_TEXTURE_2D,
                self.color_texture,
                0,
            );
            glGenRenderbuffers(1, &mut self.depth_stencil_renderbuffer);
            glBindRenderbuffer(GL_RENDERBUFFER, self.depth_stencil_renderbuffer);
            glRenderbufferStorage(
                GL_RENDERBUFFER,
                GL_DEPTH24_STENCIL8,
                width as GLsizei,
                height as GLsizei,
            );
            glFramebufferRenderbuffer(
                GL_FRAMEBUFFER,
                GL_DEPTH_STENCIL_ATTACHMENT,
                GL_RENDERBUFFER,
                self.depth_stencil_renderbuffer,
            );
            let status = glCheckFramebufferStatus(GL_FRAMEBUFFER);
            if status != GL_FRAMEBUFFER_COMPLETE {
                return Err(format!("DrawingBuffer FBO incomplete: 0x{status:x}"));
            }
            glViewport(0, 0, width as GLsizei, height as GLsizei);
            glScissor(0, 0, width as GLsizei, height as GLsizei);
        }
        self.width = width;
        self.height = height;
        Ok(())
    }

    fn CompileShader(&mut self, shader_type: u32, source: &str) -> ShaderCompileResult {
        self.MakeCurrent();
        let source = CString::new(self.ShaderSource(source)).unwrap_or_default();
        unsafe {
            let shader = glCreateShader(shader_type);
            let ptr = source.as_ptr();
            glShaderSource(shader, 1, &ptr, ptr::null());
            glCompileShader(shader);
            let mut compiled = 0;
            glGetShaderiv(shader, GL_COMPILE_STATUS, &mut compiled);
            ShaderCompileResult {
                shader,
                compiled: compiled != 0,
                info_log: Self::ShaderLog(shader),
            }
        }
    }

    fn LinkProgram(&mut self, vertex_shader: u32, fragment_shader: u32) -> ProgramLinkResult {
        self.MakeCurrent();
        unsafe {
            let program = glCreateProgram();
            glAttachShader(program, vertex_shader);
            glAttachShader(program, fragment_shader);
            glLinkProgram(program);
            let mut linked = 0;
            glGetProgramiv(program, GL_LINK_STATUS, &mut linked);
            ProgramLinkResult {
                program,
                linked: linked != 0,
                info_log: Self::ProgramLog(program),
            }
        }
    }

    fn DeleteShader(&mut self, shader: u32) {
        self.MakeCurrent();
        unsafe { glDeleteShader(shader) }
    }

    fn ProgramParameter(&mut self, program: u32, parameter: u32) -> i32 {
        self.MakeCurrent();
        let mut value = 0;
        unsafe { glGetProgramiv(program, parameter, &mut value) };
        value
    }

    fn ActiveUniform(&mut self, program: u32, index: u32) -> Option<ActiveInfo> {
        self.MakeCurrent();
        unsafe {
            Self::ActiveInfo(
                program,
                index,
                GL_ACTIVE_UNIFORMS,
                GL_ACTIVE_UNIFORM_MAX_LENGTH,
                glGetActiveUniform,
            )
        }
    }

    fn ActiveAttrib(&mut self, program: u32, index: u32) -> Option<ActiveInfo> {
        self.MakeCurrent();
        unsafe {
            Self::ActiveInfo(
                program,
                index,
                GL_ACTIVE_ATTRIBUTES,
                GL_ACTIVE_ATTRIBUTE_MAX_LENGTH,
                glGetActiveAttrib,
            )
        }
    }

    fn UniformLocation(&mut self, program: u32, name: &str) -> i32 {
        self.MakeCurrent();
        CString::new(name).map_or(-1, |name| unsafe {
            glGetUniformLocation(program, name.as_ptr())
        })
    }

    fn UniformMatrix2fv(&mut self, location: i32, transpose: bool, values: &[f32]) {
        self.MakeCurrent();
        unsafe {
            glUniformMatrix2fv(
                location,
                (values.len() / 4) as GLsizei,
                transpose.into(),
                values.as_ptr(),
            )
        }
    }

    fn UniformMatrix4fv(&mut self, location: i32, transpose: bool, values: &[f32]) {
        self.MakeCurrent();
        unsafe {
            glUniformMatrix4fv(
                location,
                (values.len() / 16) as GLsizei,
                transpose.into(),
                values.as_ptr(),
            )
        }
    }

    fn UniformMatrix3fv(&mut self, location: i32, transpose: bool, values: &[f32]) {
        self.MakeCurrent();
        unsafe {
            glUniformMatrix3fv(
                location,
                (values.len() / 9) as GLsizei,
                transpose.into(),
                values.as_ptr(),
            )
        }
    }

    fn Uniform1fv(&mut self, location: i32, values: &[f32]) {
        self.MakeCurrent();
        unsafe { glUniform1fv(location, values.len() as GLsizei, values.as_ptr()) }
    }

    fn Uniform2fv(&mut self, location: i32, values: &[f32]) {
        self.MakeCurrent();
        unsafe { glUniform2fv(location, (values.len() / 2) as GLsizei, values.as_ptr()) }
    }

    fn Uniform3fv(&mut self, location: i32, values: &[f32]) {
        self.MakeCurrent();
        unsafe { glUniform3fv(location, (values.len() / 3) as GLsizei, values.as_ptr()) }
    }

    fn Uniform4fv(&mut self, location: i32, values: &[f32]) {
        self.MakeCurrent();
        unsafe { glUniform4fv(location, (values.len() / 4) as GLsizei, values.as_ptr()) }
    }

    fn Uniform2f(&mut self, location: i32, x: f32, y: f32) {
        self.MakeCurrent();
        unsafe { glUniform2f(location, x, y) }
    }

    fn Uniform3f(&mut self, location: i32, x: f32, y: f32, z: f32) {
        self.MakeCurrent();
        unsafe { glUniform3f(location, x, y, z) }
    }

    fn Uniform4f(&mut self, location: i32, x: f32, y: f32, z: f32, w: f32) {
        self.MakeCurrent();
        unsafe { glUniform4f(location, x, y, z, w) }
    }

    fn Uniform1f(&mut self, location: i32, value: f32) {
        self.MakeCurrent();
        unsafe { glUniform1f(location, value) }
    }

    fn Uniform1iv(&mut self, location: i32, values: &[i32]) {
        self.MakeCurrent();
        unsafe { glUniform1iv(location, values.len() as GLsizei, values.as_ptr()) }
    }

    fn Uniform2iv(&mut self, location: i32, values: &[i32]) {
        self.MakeCurrent();
        unsafe { glUniform2iv(location, (values.len() / 2) as GLsizei, values.as_ptr()) }
    }

    fn Uniform3iv(&mut self, location: i32, values: &[i32]) {
        self.MakeCurrent();
        unsafe { glUniform3iv(location, (values.len() / 3) as GLsizei, values.as_ptr()) }
    }

    fn Uniform4iv(&mut self, location: i32, values: &[i32]) {
        self.MakeCurrent();
        unsafe { glUniform4iv(location, (values.len() / 4) as GLsizei, values.as_ptr()) }
    }

    fn Uniform1i(&mut self, location: i32, value: i32) {
        self.MakeCurrent();
        unsafe { glUniform1i(location, value) }
    }

    fn Uniform2i(&mut self, location: i32, x: i32, y: i32) {
        self.MakeCurrent();
        unsafe { glUniform2i(location, x, y) }
    }

    fn Uniform3i(&mut self, location: i32, x: i32, y: i32, z: i32) {
        self.MakeCurrent();
        unsafe { glUniform3i(location, x, y, z) }
    }

    fn Uniform4i(&mut self, location: i32, x: i32, y: i32, z: i32, w: i32) {
        self.MakeCurrent();
        unsafe { glUniform4i(location, x, y, z, w) }
    }

    fn Uniform1uiv(&mut self, location: i32, values: &[u32]) {
        self.MakeCurrent();
        unsafe { glUniform1uiv(location, values.len() as GLsizei, values.as_ptr()) }
    }

    fn Uniform2uiv(&mut self, location: i32, values: &[u32]) {
        self.MakeCurrent();
        unsafe { glUniform2uiv(location, (values.len() / 2) as GLsizei, values.as_ptr()) }
    }

    fn Uniform3uiv(&mut self, location: i32, values: &[u32]) {
        self.MakeCurrent();
        unsafe { glUniform3uiv(location, (values.len() / 3) as GLsizei, values.as_ptr()) }
    }

    fn Uniform4uiv(&mut self, location: i32, values: &[u32]) {
        self.MakeCurrent();
        unsafe { glUniform4uiv(location, (values.len() / 4) as GLsizei, values.as_ptr()) }
    }

    fn Uniform1ui(&mut self, location: i32, value: u32) {
        self.MakeCurrent();
        unsafe { glUniform1ui(location, value) }
    }

    fn Uniform2ui(&mut self, location: i32, x: u32, y: u32) {
        self.MakeCurrent();
        unsafe { glUniform2ui(location, x, y) }
    }

    fn Uniform3ui(&mut self, location: i32, x: u32, y: u32, z: u32) {
        self.MakeCurrent();
        unsafe { glUniform3ui(location, x, y, z) }
    }

    fn Uniform4ui(&mut self, location: i32, x: u32, y: u32, z: u32, w: u32) {
        self.MakeCurrent();
        unsafe { glUniform4ui(location, x, y, z, w) }
    }

    fn CreateBuffer(&mut self) -> u32 {
        self.MakeCurrent();
        let mut id = 0;
        unsafe { glGenBuffers(1, &mut id) };
        id
    }
    fn BindBuffer(&mut self, target: u32, buffer: u32) {
        self.MakeCurrent();
        unsafe { glBindBuffer(target, buffer) }
    }
    fn BufferData(&mut self, target: u32, bytes: &[u8], usage: u32) {
        self.MakeCurrent();
        unsafe { glBufferData(target, bytes.len() as isize, bytes.as_ptr().cast(), usage) }
    }
    fn CreateTexture(&mut self) -> u32 {
        self.MakeCurrent();
        let mut id = 0;
        unsafe { glGenTextures(1, &mut id) };
        id
    }
    fn BindTexture(&mut self, target: u32, texture: u32) {
        self.MakeCurrent();
        unsafe { glBindTexture(target, texture) }
    }
    fn TexParameteri(&mut self, target: u32, name: u32, value: i32) {
        self.MakeCurrent();
        unsafe { glTexParameteri(target, name, value) }
    }
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
    ) {
        self.MakeCurrent();
        unsafe {
            glTexImage2D(
                target,
                level,
                internal_format,
                width,
                height,
                border,
                format,
                kind,
                pixels.map_or(ptr::null(), |pixels| pixels.as_ptr().cast()),
            )
        }
    }
    fn UseProgram(&mut self, program: u32) {
        self.MakeCurrent();
        unsafe { glUseProgram(program) }
    }
    fn AttribLocation(&mut self, program: u32, name: &str) -> i32 {
        self.MakeCurrent();
        CString::new(name).map_or(-1, |name| unsafe {
            glGetAttribLocation(program, name.as_ptr())
        })
    }
    fn EnableVertexAttribArray(&mut self, index: u32) {
        self.MakeCurrent();
        unsafe { glEnableVertexAttribArray(index) }
    }
    fn VertexAttribPointerF32(&mut self, index: u32, size: i32, stride: i32, offset: usize) {
        self.MakeCurrent();
        unsafe { glVertexAttribPointer(index, size, GL_FLOAT, 0, stride, offset as *const c_void) }
    }
    fn VertexAttrib2fv(&mut self, index: u32, values: &[f32]) {
        if values.len() < 2 {
            return;
        }
        self.MakeCurrent();
        unsafe { glVertexAttrib2fv(index, values.as_ptr()) }
    }
    fn CreateVertexArray(&mut self) -> u32 {
        self.MakeCurrent();
        let mut id = 0;
        unsafe { glGenVertexArrays(1, &mut id) };
        id
    }
    fn BindVertexArray(&mut self, vertex_array: u32) {
        self.MakeCurrent();
        unsafe { glBindVertexArray(vertex_array) }
    }
    fn Viewport(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.MakeCurrent();
        unsafe { glViewport(x, y, width, height) }
    }
    fn ClearColor(&mut self, red: f32, green: f32, blue: f32, alpha: f32) {
        self.MakeCurrent();
        unsafe { glClearColor(red, green, blue, alpha) }
    }
    fn ClearDepth(&mut self, depth: f64) {
        self.MakeCurrent();
        unsafe { glClearDepth(depth) }
    }
    fn ClearStencil(&mut self, stencil: i32) {
        self.MakeCurrent();
        unsafe { glClearStencil(stencil) }
    }
    fn Enable(&mut self, capability: u32) {
        self.MakeCurrent();
        unsafe { glEnable(capability) }
    }
    fn Disable(&mut self, capability: u32) {
        self.MakeCurrent();
        unsafe { glDisable(capability) }
    }
    fn DepthFunc(&mut self, function: u32) {
        self.MakeCurrent();
        unsafe { glDepthFunc(function) }
    }
    fn FrontFace(&mut self, mode: u32) {
        self.MakeCurrent();
        unsafe { glFrontFace(mode) }
    }
    fn CullFace(&mut self, mode: u32) {
        self.MakeCurrent();
        unsafe { glCullFace(mode) }
    }
    fn Scissor(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.MakeCurrent();
        unsafe { glScissor(x, y, width, height) }
    }
    fn DepthMask(&mut self, enabled: bool) {
        self.MakeCurrent();
        unsafe { glDepthMask(enabled.into()) }
    }
    fn ColorMask(&mut self, red: bool, green: bool, blue: bool, alpha: bool) {
        self.MakeCurrent();
        unsafe { glColorMask(red.into(), green.into(), blue.into(), alpha.into()) }
    }
    fn BlendEquation(&mut self, mode: u32) {
        self.MakeCurrent();
        unsafe { glBlendEquation(mode) }
    }
    fn BlendFuncSeparate(
        &mut self,
        source_rgb: u32,
        destination_rgb: u32,
        source_alpha: u32,
        destination_alpha: u32,
    ) {
        self.MakeCurrent();
        unsafe { glBlendFuncSeparate(source_rgb, destination_rgb, source_alpha, destination_alpha) }
    }
    fn BindDefaultFramebuffer(&mut self) {
        self.MakeCurrent();
        unsafe { glBindFramebuffer(GL_FRAMEBUFFER, self.framebuffer) }
    }
    fn DrawDefaultBuffer(&mut self) {
        self.MakeCurrent();
        unsafe { glDrawBuffer(GL_COLOR_ATTACHMENT0) }
    }
    fn Clear(&mut self, mask: u32) {
        self.MakeCurrent();
        unsafe { glClear(mask) }
    }
    fn DrawArrays(&mut self, mode: u32, first: i32, count: i32) {
        self.MakeCurrent();
        unsafe { glDrawArrays(mode, first, count) }
    }
    fn DrawElements(&mut self, mode: u32, count: i32, kind: u32, offset: usize) {
        self.MakeCurrent();
        unsafe { glDrawElements(mode, count, kind, offset as *const c_void) }
    }
    fn IntegerParameter(&mut self, parameter: u32) -> i32 {
        self.MakeCurrent();
        let mut value = 0;
        unsafe { glGetIntegerv(parameter, &mut value) };
        value
    }
    fn IntegerParameters(&mut self, parameter: u32, count: usize) -> Vec<i32> {
        self.MakeCurrent();
        let mut values = vec![0; count];
        unsafe { glGetIntegerv(parameter, values.as_mut_ptr()) };
        values
    }
    fn FloatParameters(&mut self, parameter: u32, count: usize) -> Vec<f32> {
        self.MakeCurrent();
        let mut values = vec![0.0; count];
        unsafe { glGetFloatv(parameter, values.as_mut_ptr()) };
        values
    }

    fn Readback(&mut self) -> Result<CanvasPixelSnapshot, String> {
        self.MakeCurrent();
        let row_bytes = self.width as usize * 4;
        let mut bottom_up = vec![0_u8; row_bytes * self.height as usize];
        unsafe {
            glFinish();
            glReadPixels(
                0,
                0,
                self.width as i32,
                self.height as i32,
                GL_RGBA,
                GL_UNSIGNED_BYTE,
                bottom_up.as_mut_ptr().cast(),
            );
        }
        let mut pixels = vec![0_u8; bottom_up.len()];
        for y in 0..self.height as usize {
            let source = (self.height as usize - 1 - y) * row_bytes;
            pixels[y * row_bytes..(y + 1) * row_bytes]
                .copy_from_slice(&bottom_up[source..source + row_bytes]);
        }
        Ok(CanvasPixelSnapshot {
            width: self.width,
            height: self.height,
            row_bytes,
            pixels,
        })
    }
}

impl Drop for CglWebGlContext {
    fn drop(&mut self) {
        self.MakeCurrent();
        unsafe {
            if self.framebuffer != 0 {
                glDeleteFramebuffers(1, &self.framebuffer);
            }
            if self.color_texture != 0 {
                glDeleteTextures(1, &self.color_texture);
            }
            if self.depth_stencil_renderbuffer != 0 {
                glDeleteRenderbuffers(1, &self.depth_stencil_renderbuffer);
            }
            CGLSetCurrentContext(ptr::null_mut());
            CGLDestroyContext(self.context);
        }
    }
}
