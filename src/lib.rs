extern crate console_error_panic_hook;

use web_sys::WebGlBuffer;



mod graphics;
mod console;
use graphics::*;



use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGlProgram, WebGl2RenderingContext, WebGlShader};
use web_sys::{ErrorEvent, MessageEvent, WebSocket};
use shclient_gen::*;

static mut STATE: Option<Manager> = None;


#[wasm_bindgen]
pub fn game_initial(gameid:u32,name:js_sys::JsString)->js_sys::ArrayBuffer{
    let gameid=GameID(gameid);
    
    let name=PlayerName(name.into());
    let k=ClientToServer::JoinRequest{gameid,name};
    let bytes=bincode::serialize(&k).unwrap();
    let k:js_sys::Array=bytes.into_iter().map(|a|JsValue::from(a)).collect();
    js_sys::Uint8Array::new(&k).buffer()
}

use crate::console::log;
#[wasm_bindgen]
pub fn game_new(gameid:u32,name:js_sys::JsString,a:js_sys::ArrayBuffer,b:js_sys::ArrayBuffer){
    console_log!("YOOO");
    let gameid=GameID(gameid);
    let a=unsafe{
        js_sys::Uint8Array::new(&a).to_vec()
    };
    let b=unsafe{
        js_sys::Uint8Array::new(&b).to_vec()
    };
    console_log!("YOOO");
    let a:ServerToClient=bincode::deserialize(&a).map(|a|{println!("Received {:?}",a);a}).unwrap();
    let b:ServerToClient=bincode::deserialize(&b).map(|b|{println!("Received {:?}",b);b}).unwrap();
    console_log!("YOOO");
    let name=PlayerName(name.into());
    unsafe{
        STATE=Some(Manager::new(gameid,name,a,b));
    }
    console_log!("YOOO");
}

#[wasm_bindgen]
pub fn game_premove(x:f32,y:f32,clicked:bool)->Option<js_sys::ArrayBuffer>{
    let mycommit=if clicked{
        Some([x,y])
    }else{
        None
    };

    let k=unsafe{STATE.as_mut().unwrap().premove(mycommit)};
    if let Some(k)=k{
        let bytes=bincode::serialize(&k).unwrap();
        let k:js_sys::Array=bytes.into_iter().map(|a|JsValue::from(a)).collect();
        Some(js_sys::Uint8Array::new(&k).buffer())
    }else{
        None
    }
    
}


#[wasm_bindgen]
pub fn game_process(s:Option<js_sys::ArrayBuffer>){
    let a=if let Some(a)=s{
        let a=unsafe{
            js_sys::Uint8Array::new(&a).to_vec()
        };
        let a:ServerToClient=bincode::deserialize(&a).map(|a|{println!("Received {:?}",a);a}).unwrap();
        Some(a)
    }else{
        None
    };
    unsafe{STATE.as_mut().unwrap().recv(a)}
}

#[wasm_bindgen]
pub fn game_draw(){
    let game=unsafe{STATE.as_ref().unwrap().get_game()};
    //TODO draw
}



#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;


    dbg!("YOO");
    let context = canvas
        .get_context("webgl2")?
        .unwrap()
        .dyn_into::<WebGl2RenderingContext>()?;

    
    
    let mut p=MyProgram::new(&context)?;

    //let vertices= [-0.7f32, -0.7, 0.7, -0.7, 0.0, 0.7];

    let vertices= [400.0, 500.0];

    context.clear_color(0.0, 0.0, 0.0, 1.0);
    context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

    let dim=[800.0,600.0];
    p.draw(&context,&vertices,dim,true);
    

    
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
