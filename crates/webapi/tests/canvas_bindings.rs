use dom::dom_mutation::ApplyDOMMutations;
use javascript::{
    javascript_runtime::JavaScriptRuntime, quickjs_javascript_runtime::QuickJsJavaScriptRuntime,
};
use std::{cell::RefCell, rc::Rc};
use webapi::{dom_bindings::DOMJavaScriptBindings, window_bindings::WindowJavaScriptBindings};

struct NoNetwork;
impl xhr_transport::XMLHttpRequestTransport for NoNetwork {
    fn Start(
        &mut self,
        _: &xhr_transport::XMLHttpRequestData,
    ) -> std::io::Result<Box<dyn xhr_transport::XMLHttpRequestOperation>> {
        Err(std::io::Error::other("fixture has no network"))
    }
}

fn run(script: &str) {
    let document = Rc::new(RefCell::new(html::html_parser::ParseHTML(
        "<!doctype html><canvas id=canvas></canvas>",
    )));
    let changed = document.clone();
    let dom = Rc::new(RefCell::new(DOMJavaScriptBindings::new(
        document,
        Box::new(move |mutation| {
            ApplyDOMMutations(
                changed.borrow_mut().GetDocumentMut(),
                std::slice::from_ref(mutation),
            );
        }),
    )));
    let window = Rc::new(RefCell::new(WindowJavaScriptBindings::new(
        dom,
        Box::new(NoNetwork),
        "canvas-bindings-fixture".into(),
        None,
    )));
    let mut runtime = QuickJsJavaScriptRuntime::new();
    let realm = runtime.CreateRealm(window);
    for (source, name) in [
        (webapi::DOMBootstrapSource().to_owned(), "dom-webidl"),
        (webapi::WindowBootstrapSource(), "window-webidl"),
    ] {
        let result = runtime.Evaluate(&realm, &source, name);
        assert!(result.Succeeded(), "{name}: {:?}", result.exception);
    }
    let result = runtime.Evaluate(
        &realm,
        &format!("function check(v,m){{if(!v)throw Error(m)}};{script}"),
        "canvas-bindings.js",
    );
    assert!(result.Succeeded(), "{:?}", result.exception);
}

#[test]
fn canvas_dimensions_reflect_with_html_defaults_and_webidl_conversion() {
    run(r#"
        const canvas=document.getElementById('canvas');
        check(Object.getOwnPropertyDescriptor(HTMLCanvasElement.prototype,'width').enumerable,'width IDL enumerable');
        check(Object.getOwnPropertyDescriptor(HTMLCanvasElement.prototype,'getContext').enumerable,'method IDL enumerable');
        check(canvas.width===300&&canvas.height===150,'default dimensions');
        canvas.width=640;canvas.height=480;
        check(canvas.getAttribute('width')==='640'&&canvas.width===640,'width reflection');
        check(canvas.getAttribute('height')==='480'&&canvas.height===480,'height reflection');
        canvas.setAttribute('width','  +12');check(canvas.width===12,'HTML integer parse');
        canvas.setAttribute('width','27px');check(canvas.width===27,'HTML integer prefix');
        canvas.setAttribute('width','-0px');check(canvas.width===0,'minus zero accepted');
        canvas.setAttribute('height','bad');check(canvas.height===150,'invalid default');
        canvas.width=-1;
        check(canvas.getAttribute('width')==='300'&&canvas.width===300,'oversize IDL width stores default');
        canvas.setAttribute('width','4294967295');
        check(canvas.getAttribute('width')==='4294967295'&&canvas.width===300,'oversize content width exposes default');
        canvas.setAttribute('height','2147483648');check(canvas.height===150,'oversize height default');
        canvas.height=NaN;check(canvas.height===0,'NaN converts to zero');
    "#);
}

#[test]
#[cfg(feature = "webgl_vertical_slice")]
fn webgl_context_is_real_reused_and_renders_a_triangle() {
    run(r#"
        const canvas=document.getElementById('canvas');
        const gl=canvas.getContext('webgl');
        check(gl instanceof WebGLRenderingContext,'real WebGL context');
        check(canvas.getContext('experimental-webgl')===gl,'context identity');
        check(canvas.getContext('webgl2')===null,'different context family rejected');
        check(canvas.getContext('2d')===null,'Canvas2D provider absent');
        check(canvas.getContext('unknown')===null,'unknown context');
        let threw=false;try{canvas.getContext(Symbol())}catch(e){threw=e instanceof TypeError}
        check(threw,'DOMString conversion');

        const vertex=gl.createShader(gl.VERTEX_SHADER);
        gl.shaderSource(vertex,'attribute vec2 position;void main(){gl_Position=vec4(position,0.,1.);}');
        gl.compileShader(vertex);check(gl.getShaderParameter(vertex,gl.COMPILE_STATUS),gl.getShaderInfoLog(vertex));
        const fragment=gl.createShader(gl.FRAGMENT_SHADER);
        gl.shaderSource(fragment,'void main(){gl_FragColor=vec4(1.,0.,0.,1.);}');
        gl.compileShader(fragment);check(gl.getShaderParameter(fragment,gl.COMPILE_STATUS),gl.getShaderInfoLog(fragment));
        const program=gl.createProgram();gl.attachShader(program,vertex);gl.attachShader(program,fragment);gl.linkProgram(program);
        check(gl.getProgramParameter(program,gl.LINK_STATUS),gl.getProgramInfoLog(program));gl.useProgram(program);
        const buffer=gl.createBuffer();gl.bindBuffer(gl.ARRAY_BUFFER,buffer);
        gl.bufferData(gl.ARRAY_BUFFER,new Float32Array([-1,-1,1,-1,0,1]),gl.STATIC_DRAW);
        const position=gl.getAttribLocation(program,'position');gl.enableVertexAttribArray(position);
        gl.vertexAttribPointer(position,2,gl.FLOAT,false,0,0);gl.viewport(0,0,300,150);
        gl.clearColor(0,0,0,0);gl.clear(gl.COLOR_BUFFER_BIT);gl.drawArrays(gl.TRIANGLES,0,3);
        const pixels=gl.__snapshot(), center=(75*300+150)*4;
        check(pixels[center]>240&&pixels[center+3]>240,'GPU triangle readback');
        check(gl.getError()===gl.NO_ERROR,'no WebGL error');
    "#);
}

#[test]
#[cfg(feature = "webgl_vertical_slice")]
fn webgl2_context_renders_indexed_geometry_with_uniforms_and_vao() {
    run(r#"
        const canvas=document.getElementById('canvas');canvas.width=32;canvas.height=32;
        const gl=canvas.getContext('webgl2');
        check(gl instanceof WebGL2RenderingContext,'real WebGL2 context');
        check(canvas.getContext('webgl2')===gl,'WebGL2 context identity');
        check(canvas.getContext('webgl')===null,'different context family rejected');
        check(gl.getParameter(gl.VERSION).startsWith('WebGL 2.0'),'WebGL2 version string');
        check(gl.getParameter(gl.VENDOR)==='WebKit'&&gl.getParameter(gl.RENDERER)==='WebKit WebGL','masked vendor and renderer');
        const viewport=gl.getParameter(gl.VIEWPORT),scissor=gl.getParameter(gl.SCISSOR_BOX);
        check(viewport instanceof Int32Array&&viewport.length===4&&viewport[2]===32&&viewport[3]===32,'viewport parameter type');
        check(scissor instanceof Int32Array&&scissor.length===4,'scissor parameter type');
        const vertex=gl.createShader(gl.VERTEX_SHADER);
        gl.shaderSource(vertex,`#version 300 es
          in vec2 position;uniform mat4 matrix;uniform vec2 offset;
          void main(){gl_Position=matrix*vec4(position+offset,0.,1.);}`);
        gl.compileShader(vertex);check(gl.getShaderParameter(vertex,gl.COMPILE_STATUS),gl.getShaderInfoLog(vertex));
        const fragment=gl.createShader(gl.FRAGMENT_SHADER);
        gl.shaderSource(fragment,`#version 300 es
          precision highp float;out vec4 color;uniform vec4 tint;
          void main(){color=tint;}`);
        gl.compileShader(fragment);check(gl.getShaderParameter(fragment,gl.COMPILE_STATUS),gl.getShaderInfoLog(fragment));
        const program=gl.createProgram();gl.attachShader(program,vertex);gl.attachShader(program,fragment);gl.linkProgram(program);
        check(gl.getProgramParameter(program,gl.LINK_STATUS),gl.getProgramInfoLog(program));
        check(gl.getProgramParameter(program,gl.ACTIVE_ATTRIBUTES)===1,'active attribute count');
        check(gl.getProgramParameter(program,gl.ACTIVE_UNIFORMS)===3,'active uniform count');
        check(gl.getActiveAttrib(program,0).name==='position','active attribute');
        gl.useProgram(program);gl.deleteShader(vertex);gl.deleteShader(fragment);
        const vao=gl.createVertexArray();gl.bindVertexArray(vao);
        const vertices=gl.createBuffer();gl.bindBuffer(gl.ARRAY_BUFFER,vertices);
        gl.bufferData(gl.ARRAY_BUFFER,new Float32Array([-1,-1,1,-1,0,1]),gl.STATIC_DRAW);
        const position=gl.getAttribLocation(program,'position');gl.enableVertexAttribArray(position);
        gl.vertexAttribPointer(position,2,gl.FLOAT,false,0,0);
        const indices=gl.createBuffer();gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER,indices);
        gl.bufferData(gl.ELEMENT_ARRAY_BUFFER,new Uint16Array([0,1,2]),gl.STATIC_DRAW);
        const matrix=gl.getUniformLocation(program,'matrix');
        gl.uniformMatrix4fv(matrix,false,new Float32Array([1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1]));
        gl.uniform2f(gl.getUniformLocation(program,'offset'),0,0);
        gl.uniform4fv(gl.getUniformLocation(program,'tint'),new Float32Array([1,0,0,1]));
        gl.viewport(0,0,32,32);gl.clearColor(0,0,0,0);gl.clear(gl.COLOR_BUFFER_BIT|gl.DEPTH_BUFFER_BIT);
        gl.drawElements(gl.TRIANGLES,3,gl.UNSIGNED_SHORT,0);
        const pixels=gl.__snapshot(),center=(16*32+16)*4;
        check(pixels[center]>240&&pixels[center+3]>240,'WebGL2 indexed triangle readback');
        check(gl.getError()===gl.NO_ERROR,'no WebGL2 error');
    "#);
}

#[test]
#[cfg(not(feature = "webgl_vertical_slice"))]
fn incomplete_webgl_slice_does_not_change_feature_detection() {
    run(r#"
        const canvas=document.getElementById('canvas');
        check(canvas.getContext('webgl')===null,'incomplete WebGL stays unavailable');
        check(canvas.getContext('2d')===null,'Canvas2D provider absent');
        check(canvas.getContext('unknown')===null,'unknown context');
    "#);
}
