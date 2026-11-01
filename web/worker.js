import init,{inspect} from './pkg/mdpath_web.js';
try{await init({module_or_path:new URL('./pkg/mdpath_web_bg.wasm',import.meta.url)});postMessage({type:'ready'});}catch(error){postMessage({type:'startup-error',message:String(error)});}
onmessage=({data})=>{if(data.type==='resolve')try{postMessage({type:'result',id:data.id,canonical:inspect(data.request)});}catch(error){postMessage({type:'compute-error',id:data.id,message:String(error)});}};
