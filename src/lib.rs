extern crate console_error_panic_hook;

use web_sys::WebGlBuffer;


mod circle_program;
mod graphics;
mod console;
use graphics::*;
use crate::graphics::Args;
use crate::graphics::create_draw_system;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGlProgram, WebGl2RenderingContext, WebGlShader};
use web_sys::{ErrorEvent, MessageEvent, WebSocket};
use shclient_gen::*;

static mut STATE: Option<Manager> = None;
static mut PROGRAM:Option<DrawSys> = None;

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
    let cursor=axgeom::vec2(x,y);

    let m=unsafe{STATE.as_mut().unwrap()};

    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();

    let window_dim_x=canvas.width() as f32;
    let window_dim_y=canvas.height() as f32;
    


    let mycommit=if clicked{
        let myplayerid=m.get_playerid();
        let target=cursor.inner_into();
        let half=axgeom::vec2(window_dim_x,window_dim_y)/2.0;
        let p=*m.get_camera();
        //let p=m.get_game().state.bots[myplayerid.0 as usize].0.body.pos;
        let mtarget=-half+target+p;
        
        Some(mtarget.into())
    }else{
        None
    };

    let k=m.premove(mycommit);
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
pub fn game_draw(width:i32,height:i32){

    let game=unsafe{STATE.as_ref().unwrap().get_game()};

    //TODO get context every time?
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();


    let context = canvas
        .get_context("webgl2").unwrap()
        .unwrap()
        .dyn_into::<WebGl2RenderingContext>().unwrap();

        context.viewport(0, 0, width, height);
          
    let mut p=unsafe{PROGRAM.as_mut().unwrap()};


    let m=unsafe{STATE.as_ref().unwrap()};
    
    let verts:Vec<_>=m.get_game().state.bots.iter().map(|(b,_)|{
        let [xx,yy]:[f32;2]=b.body.pos.into();
        Vertex([xx,yy,b.head.rot])
    }).collect();


    let mut squares=Vec::new();
    let walls=&game.nonstate.walls;
    let grid_viewport=&game.nonstate.grid_viewport;
     for x in 0..walls.dim().x {
        for y in 0..walls.dim().y {
            let curr=axgeom::vec2(x,y);
            if walls.get(curr) {
                let pos=grid_viewport.to_world_center(axgeom::vec2(x, y));
                let [xx,yy]:[f32;2]=pos.into();
                squares.push(Vertex([xx,yy,1.0]));
            }
        }
    }

    let window_dim_x=canvas.width() as f32;
    let window_dim_y=canvas.height() as f32;
    
    let dim=[window_dim_x,window_dim_y];
    context.clear_color(0.0, 0.0, 0.0, 1.0);
    context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

    let pp=m.get_camera();
    let offset=-(pp.inner_into::<f32>())+axgeom::vec2(window_dim_x,window_dim_y)/2.0;
    let offset:[f32;2]=offset.into();

    let wall_point_size=grid_viewport.cell_radius();
    let bot_point_size=game.nonstate.radius*2.0;


    if let &Some([x,y])=m.get_target(){
        let squares=vec!(Vertex([x,y,0.0]));
        let args=Args{
            context:&context,
            vertices:&squares,
            game_dim:dim,
            as_square:true,
            color:&[1.0,1.0,1.0,1.0],
            offset:&offset,
            point_size:6.0
        };
        p(args);
    }

    let args=Args{
        context:&context,
        vertices:&verts,
        game_dim:dim,
        as_square:false,
        color:&[1.0,1.0,1.0,1.0],
        offset:&offset,
        point_size:bot_point_size
    };
    p(args);



    let args=Args{
        context:&context,
        vertices:&squares,
        game_dim:dim,
        as_square:true,
        color:&[0.0,1.0,0.0,1.0],
        offset:&offset,
        point_size:wall_point_size
    };
    p(args);
}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    //TODO get context every time?
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
    let context = canvas
        .get_context("webgl2").unwrap()
        .unwrap()
        .dyn_into::<WebGl2RenderingContext>().unwrap();
        let p=match create_draw_system(&context){
            Ok(o)=>Box::new(o),
            Err(e)=>{console_log!("{}",e);panic!("faail")}
        };
        
    unsafe{PROGRAM=Some(p)};
    Ok(())


}