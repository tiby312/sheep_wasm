import init, {init2,game_initial,game_new,game_premove,game_process,game_draw} from './pkg/without_a_bundler.js';


console.log("worker started!");

onmessage=function(e){
    console.log("received message in worker",e);
}