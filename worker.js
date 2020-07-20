import init, {init2,game_initial,game_new,game_premove,game_process,game_draw} from './pkg/without_a_bundler.js';


console.log("worker started!");
let clicks = []
        

var width=0;
var height=0;

var offscreen;
onmessage=function(e){
    console.log(e);
    var data = e.data;
    switch (data.cmd) {
        case 'context':
            offscreen=data.canvas;    
            width=data.width;
            height=data.height;
            break;
        case 'click':
            console.log(data);
            clicks.push(data.mouse);
            break;
        case 'shutdown':
            self.close();
            break;
        default:
            self.postMessage('Unknown command: ' + data.msg);
    };

    offscreen=e.data.canvas;
    console.log(offscreen);
    //width=e.data[0];
    //height=e.data[1];
    //console.log("received message in worker",e);
}



let messages = []
function socket_connect() {
    return new Promise(function(resolve, reject) {
        //var server = new WebSocket('ws://127.0.0.1:12345');
        var server = new WebSocket('ws://192.168.4.40:12345');
        
        
        server.onopen = function() {
            resolve(server);
        };
        server.onerror = function(err) {
            reject(err);
        };
        server.onmessage=function(m){
          messages.push(m);
        } 

    });
}

function receive(socket){
    return new Promise(function(resolve,reject){
         (function waitForFoo(){
             if (messages.length>0){
               return resolve(messages.shift().data);
             } 
             console.log("WAITING");
             setTimeout(waitForFoo, 1);
         })();
    });
 }


async function run() {
    // First up we need to actually load the wasm file, so we use the
    // default export to inform it where the wasm file is located on the
    // server, and then we wait on the returned promise to wait for the
    // wasm to be loaded.
    //
    // It may look like this: `await init('./pkg/without_a_bundler_bg.wasm');`,
    // but there is also a handy default inside `init` function, which uses
    // `import.meta` to locate the wasm file relatively to js file.
    //
    // Note that instead of a string you can also pass in any of the
    // following things:
    //
    // * `WebAssembly.Module`
    //
    // * `ArrayBuffer`
    //
    // * `Response`
    //
    // * `Promise` which returns any of the above, e.g. `fetch("./path/to/wasm")`
    //
    // This gives you complete control over how the module is loaded
    // and compiled.
    //
    // Also note that the promise, when resolved, yields the wasm module's
    // exports which is the same as importing the `*_bg` module in other
    // modes
    //await init();

    await init();
        
    console.log(offscreen);
    var context=offscreen.getContext("webgl2");

    init2(context);


    try {
        let socket = await socket_connect();
        socket.binaryType="arraybuffer";
        // ... use server
        let o=game_initial(0,"hay");
        console.log(o);
        socket.send(o);

        {
          let g1=await receive(socket);
          //let g2=await receive(socket);
          game_new(0,"hay",g1,socket);
        }
        function delay(ms) {
          return new Promise(resolve => setTimeout(resolve, ms));
        }
        

        function render(time) {
            console.log(width,height);
            context.viewport(0, 0, width,height);
            
            game_draw(width,height,context);
            // ... some drawing using the gl context ...
            requestAnimationFrame(render);
        }
        requestAnimationFrame(render);
      
        let framerate=16;
        let diff=0;
        while(true){
          if (framerate-diff>0){
            await delay(framerate-diff)
          }
          //await delay(5);
          const t0 = performance.now();
          let foo={x:0.0,y:0.0};
          let clicked=false;
          if (clicks.length>0){
            
            foo=clicks.pop();
            clicks=[];
            clicked=true;
          }

          if (game_premove(width,height,foo.x,foo.y,clicked,socket)){
            console.log("SENT");
            let g1=await receive(socket)
            console.log("FINALLY RECEIVED");
            let gg1=new Uint8Array(g1);
            game_process(gg1);
          }else{
            game_process(null);
          }
          

          
          const t1 = performance.now();
          diff=t1-t0;

        }
        
        
  
    } catch (error) {
        console.log("ooops ", error)
    }
    // And afterwards we can use all the functionality defined in wasm.
    //const result = add(1, 2);
    //console.log(`1 + 2 = ${result}`);
    //if (result !== 3)
    //  throw new Error("wasm addition doesn't work!");
  }
  run();