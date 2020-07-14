use web_sys::WebGlBuffer;
use wasm_bindgen::JsCast;
use web_sys::{WebGlProgram, WebGl2RenderingContext, WebGlShader};
use web_sys::{ErrorEvent, MessageEvent, WebSocket};


pub struct MyProgram{
    square_program:WebGlProgram,
    circle_program:WebGlProgram,
    buffer:WebGlBuffer
}
impl MyProgram{
    pub fn new(context:&WebGl2RenderingContext)->Result<MyProgram,String>{
        let vert_shader = compile_shader(
            &context,
            WebGl2RenderingContext::VERTEX_SHADER,
            r#"#version 300 es
            in vec2 position;
            out vec2 pos;
            uniform vec2 offset;
            uniform mat3 mmatrix;
            uniform float point_size;
            void main() {
                gl_PointSize = point_size;
                vec3 pp=vec3(position+offset,1.0);
                gl_Position = vec4(mmatrix*pp, 1.0);
            }
        "#,
        )?;
        let frag_shader_circle = compile_shader(
            &context,
            WebGl2RenderingContext::FRAGMENT_SHADER,
            r#"#version 300 es
            precision mediump float;
            out vec4 out_color;
            uniform vec4 bg;
            
            void main() {
                //coord is between -0.5 and 0.5
                vec2 coord = gl_PointCoord - vec2(0.5,0.5);
                float dissqr=dot(coord,coord);
                if(dissqr > 0.25){                  //outside of circle radius?
                    discard;
                }
                out_color = bg;
            }
        "#,
        )?;

        let frag_shader_square = compile_shader(
            &context,
            WebGl2RenderingContext::FRAGMENT_SHADER,
            r#"#version 300 es
            precision mediump float;
            out vec4 out_color;
            uniform vec4 bg;
            

            void main() {
                //coord is between -0.5 and 0.5
                vec2 coord = gl_PointCoord - vec2(0.5,0.5);
                /*
                float foo=coord.x*coord.x*coord.x*coord.x+coord.y*coord.y*coord.y*coord.y;
                if(foo > 0.25*0.25){                  //outside of circle radius?
                    discard;
                } 
                */           
                out_color = bg;
            }
        "#,
        )?;
        let buffer = context.create_buffer().ok_or("failed to create buffer")?;

        let circle_program = link_program(&context, &vert_shader, &frag_shader_circle)?;
        let square_program = link_program(&context, &vert_shader, &frag_shader_square)?;
        Ok(MyProgram{square_program,circle_program,buffer})
    }
    pub fn draw(&mut self,
        context:&WebGl2RenderingContext,
        vertices:&[f32],
        game_dim:[f32;2],
        as_square:bool,
        color:&[f32;4],
        offset:&[f32;2],
        point_size:f32){

        
        let buffer=&self.buffer;

        let program=if as_square{
            &self.square_program
        }else{
            &self.circle_program
        };
        context.use_program(Some(program));
        

        let scalex = 2.0 / game_dim[0];
        let scaley = 2.0 / game_dim[1];
        let tx = -1.0;
        let ty = 1.0;
        let matrix = [scalex, 0.0, 0.0, 0.0, -scaley, 0.0, tx, ty, 1.0];
        let mat=context.get_uniform_location(program,"mmatrix");
        context.uniform_matrix3fv_with_f32_array(mat.as_ref(),false,&matrix);
        

        let oo=context.get_uniform_location(program,"point_size");
        context.uniform1f(oo.as_ref(),point_size);
        

        let oo=context.get_uniform_location(program,"offset");
        context.uniform2f(oo.as_ref(),offset[0],offset[1]);
        

        let bf=context.get_uniform_location(program,"bg");
        context.uniform4fv_with_f32_array(bf.as_ref(),color);
        let foo=  context.get_attrib_location(program, "position") as u32;

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




