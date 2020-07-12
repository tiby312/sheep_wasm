extern crate console_error_panic_hook;

use web_sys::WebGlBuffer;

struct SimpleBuffer{
    buffer:WebGlBuffer
}
impl SimpleBuffer{
    fn new(context:&WebGl2RenderingContext)->Result<SimpleBuffer,String>{
        let buffer = context.create_buffer().ok_or("failed to create buffer")?;

        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));

        context.vertex_attrib_pointer_with_i32(0, 3, WebGl2RenderingContext::FLOAT, false, 0, 0);
        context.enable_vertex_attrib_array(0);


        Ok(SimpleBuffer{buffer})
    }

    fn draw(&self,vertices:&[f32],context:&WebGl2RenderingContext,program:&WebGlProgram){
        context.use_program(Some(&program));

        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&self.buffer));


        // Note that `Float32Array::view` is somewhat dangerous (hence the
        // `unsafe`!). This is creating a raw view into our module's
        // `WebAssembly.Memory` buffer, but if we allocate more pages for ourself
        // (aka do a memory allocation in Rust) it'll cause the buffer to change,
        // causing the `Float32Array` to be invalid.
        //
        // As a result, after `Float32Array::view` we have to be very careful not to
        // do any memory allocations before it's dropped.
        unsafe {
            let vert_array = js_sys::Float32Array::view(&vertices);

            context.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                &vert_array,
                WebGl2RenderingContext::STATIC_DRAW,
            );
        }


        context.draw_arrays(
            WebGl2RenderingContext::TRIANGLES,
            0,
            (vertices.len() / 3) as i32,
        );

    }
}


pub struct MyProgram{
    program:WebGlProgram
}
impl MyProgram{
    pub fn new(context:&WebGl2RenderingContext)->Result<MyProgram,String>{
        let vert_shader = compile_shader(
            &context,
            WebGl2RenderingContext::VERTEX_SHADER,
            r#"#version 300 es
            in vec2 position;
            out vec2 pos;
            void main() {
                gl_PointSize = 20.3;
                vec3 pp=vec3(position,1.0);
                gl_Position = vec4(pp, 1.0);
            }
        "#,
        )?;
        let frag_shader = compile_shader(
            &context,
            WebGl2RenderingContext::FRAGMENT_SHADER,
            r#"#version 300 es
            precision mediump float;
            out vec4 out_color;
            uniform vec4 bg;
            void main() {
            
                vec2 coord = gl_PointCoord - vec2(0.5,0.5);
                float dis=dot(coord,coord);
                if(dis > 0.25){                  //outside of circle radius?
                    discard;
                }
            
                out_color = bg;
            }
        "#,
        )?;
        let program = link_program(&context, &vert_shader, &frag_shader)?;
        Ok(MyProgram{program})
    }
    pub fn draw(&self,context:&WebGl2RenderingContext,buffer:&WebGlBuffer,vertices:&[f32]){
        context.use_program(Some(&self.program));
        
        let bf=context.get_uniform_location(&self.program,"bg");
        context.uniform4fv_with_f32_array(bf.as_ref(),&[1.0,0.0,1.0,1.0]);
        let foo=  context.get_attrib_location(&self.program, "position") as u32;

        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(buffer));
        context.vertex_attrib_pointer_with_i32(foo, 2, WebGl2RenderingContext::FLOAT, false, 0, 0);
        context.enable_vertex_attrib_array(0);

        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(buffer));

        // Note that `Float32Array::view` is somewhat dangerous (hence the
        // `unsafe`!). This is creating a raw view into our module's
        // `WebAssembly.Memory` buffer, but if we allocate more pages for ourself
        // (aka do a memory allocation in Rust) it'll cause the buffer to change,
        // causing the `Float32Array` to be invalid.
        //
        // As a result, after `Float32Array::view` we have to be very careful not to
        // do any memory allocations before it's dropped.
        unsafe {
            let vert_array = js_sys::Float32Array::view(  vertices);

            context.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                &vert_array,
                WebGl2RenderingContext::STATIC_DRAW,
            );
        }


        context.draw_arrays(
            WebGl2RenderingContext::POINTS,
            0,
            (vertices.len()/2) as i32,
        );

    }
}


fn make_triangle_program(context:&WebGl2RenderingContext)->Result<WebGlProgram,String>{
    let vert_shader = compile_shader(
        &context,
        WebGl2RenderingContext::VERTEX_SHADER,
        r#"
        attribute vec4 position;
        void main() {
            gl_Position = position;
        }
    "#,
    )?;
    let frag_shader = compile_shader(
        &context,
        WebGl2RenderingContext::FRAGMENT_SHADER,
        r#"
        void main() {
            gl_FragColor = vec4(1.0, 0.0, 1.0, 1.0);
        }
    "#,
    )?;
    let triangle_program = link_program(&context, &vert_shader, &frag_shader)?;
    Ok(triangle_program)
}


#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    // The `console.log` is quite polymorphic, so we can bind it with multiple
    // signatures. Note that we need to use `js_name` to ensure we always call
    // `log` in JS.
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_u32(a: u32);

    // Multiple arguments too!
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_many(a: &str, b: &str);
}

// Next let's define a macro that's like `println!`, only it works for
// `console.log`. Note that `println!` doesn't actually work on the wasm target
// because the standard library currently just eats all output. To get
// `println!`-like behavior in your app you'll likely want a macro like this.

macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}


use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGlProgram, WebGl2RenderingContext, WebGlShader};
use web_sys::{ErrorEvent, MessageEvent, WebSocket};

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    log("YOO WATUPS");
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;


    dbg!("YOO");
    let context = canvas
        .get_context("webgl2")?
        .unwrap()
        .dyn_into::<WebGl2RenderingContext>()?;

    
    let buffer = context.create_buffer().ok_or("failed to create buffer")?;

    let p=MyProgram::new(&context)?;

    let vertices= [-0.7f32, -0.7, 0.7, -0.7, 0.0, 0.7];
    
    context.clear_color(0.0, 0.0, 0.0, 1.0);
    context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

    //TODO assert point size max is big enough.
    p.draw(&context,&buffer,&vertices);
    /*
    let triangle_program=make_triangle_program(&context)?;
    
    let buffer=SimpleBuffer::new(&context)?;

    let vertices: [f32; 9] = [-0.7, -0.7, 0.0, 0.7, -0.7, 0.0, 0.0, 0.7, 0.0];

    context.clear_color(0.0, 0.0, 0.0, 1.0);
    context.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

    buffer.draw(&vertices,&context,&triangle_program);
    */
    
    Ok(())
}

pub fn compile_shader(
    context: &WebGl2RenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = context
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    if context
        .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(context
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader")))
    }
}

pub fn link_program(
    context: &WebGl2RenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = context
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    context.attach_shader(&program, vert_shader);
    context.attach_shader(&program, frag_shader);
    context.link_program(&program);

    if context
        .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(context
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}




