const $=id=>document.getElementById(id),examples=JSON.parse($('examples-data').textContent);
$('static-help').hidden=true;
let worker,epoch=0,revision=0,serial=0,active=null,ready=false,busy=false,startupTimer,computeTimer,canonical=null,result=null,fileToken=0,fileTimer,reading=false,original=null;
function controls(){$('run').disabled=!ready||busy||reading;$('report-download').disabled=!canonical;$('content-download').disabled=!result?.ok;$('original-download').disabled=!$('markdown').value&&original===null;}
function clear(){canonical=result=null;$('results').hidden=true;$('error').hidden=true;controls();}
function invalidate(){revision++;clear();if(ready)$('status').textContent=busy?'Inputs changed; waiting for the previous resolution.':'Inputs changed. Resolve again.';}
function cancelFile(){fileToken++;reading=false;clearTimeout(fileTimer);}
function edit(id){cancelFile();if(id==='markdown')original=null;invalidate();}
function diagnostic(phase,detail){$('error').hidden=false;$('error-title').textContent=phase+' needs attention';$('error-message').textContent=detail.message;$('diagnostic').textContent=JSON.stringify(detail,null,2);}
function engineError(message){epoch++;worker?.terminate();worker=null;ready=busy=false;active=null;clearTimeout(startupTimer);clearTimeout(computeTimer);clear();diagnostic('Engine',{message});$('retry').hidden=false;$('status').textContent='Engine unavailable. Inputs and independent downloads are retained.';controls();}
function request(){return JSON.stringify({markdown:$('markdown').value,path:$('path').value,uri:$('uri').value});}
function resolve(){
 if(!ready||busy||reading)return;clear();const text=request();if(new TextEncoder().encode(text).length>1048576){diagnostic('Request',{message:'Current request exceeds 1 MiB UTF-8.'});return;}
 busy=true;active={id:++serial,revision};const id=active.id,generation=epoch;controls();$('status').textContent='Resolving in Rust…';computeTimer=setTimeout(()=>{if(generation===epoch&&active?.id===id)engineError('Resolution exceeded ten seconds. Retry with your retained inputs.');},10000);
 try{worker.postMessage({type:'resolve',id,request:text});}catch{engineError('The worker could not accept this request.');}
}
function start(){
 epoch++;const generation=epoch;worker?.terminate();clearTimeout(startupTimer);clearTimeout(computeTimer);ready=busy=false;active=null;clear();$('retry').hidden=true;$('status').textContent='Loading Rust engine…';startupTimer=setTimeout(()=>{if(generation===epoch&&!ready)engineError('Rust did not start within twelve seconds. Retry to load it again.');},12000);
 try{worker=new Worker(new URL('./worker.js',import.meta.url),{type:'module'});}catch{engineError('This browser could not create the Rust worker.');return;}
 worker.onerror=e=>{e.preventDefault();if(generation===epoch)engineError('The Rust worker failed.');};
 worker.onmessage=({data})=>{
  if(generation!==epoch)return;if(data.type==='startup-error'){engineError('Rust could not be downloaded or initialized.');return;}
  if(data.type==='ready'){clearTimeout(startupTimer);ready=true;$('status').textContent='Rust engine ready.';controls();if(revision===0)resolve();return;}
  if(!active||data.id!==active.id)return;clearTimeout(computeTimer);const submitted=active.revision;busy=false;active=null;
  if(data.type==='compute-error'){engineError('The Rust worker could not complete resolution.');return;}
  if(submitted!==revision){$('status').textContent='Inputs changed. Resolve again.';controls();return;}
  try{result=JSON.parse(data.canonical);}catch{engineError('The worker returned an unreadable report.');return;}canonical=data.canonical;
  if(!result.ok){diagnostic(result.phase,result.diagnostic);$('status').textContent='Inspect the typed error, then adjust the address or document.';controls();return;}
  const resolved=result.resolved;$('element-type').textContent=resolved.element_type+(resolved.kind?' · '+resolved.kind:'');$('location').textContent=`${resolved.file} · source lines ${resolved.line_start}–${resolved.line_end}`;$('label').textContent=`Label: ${resolved.label??'none'} · Section: ${resolved.section_heading??'none'}`;$('content').textContent=resolved.content;$('address').textContent=JSON.stringify({address:result.address,document:result.document},null,2);$('results').hidden=false;$('status').textContent='Resolved using the supplied document and default classifier.';controls();
 };
}
function download(name,text,type='application/json'){const url=URL.createObjectURL(new Blob([text],{type:type+';charset=utf-8'}));const link=document.createElement('a');link.href=url;link.download=name;link.click();setTimeout(()=>URL.revokeObjectURL(url),1000);}
function example(name){cancelFile();original=null;for(const id of ['markdown','path','uri'])$(id).value=examples[name][id];$('document-file').value='';invalidate();$('uri').focus();}
for(const id of ['markdown','path','uri'])$(id).addEventListener('input',()=>edit(id));
for(const button of document.querySelectorAll('[data-example]'))button.addEventListener('click',()=>example(button.dataset.example));
$('workbench').addEventListener('submit',e=>{e.preventDefault();resolve();});$('reset').addEventListener('click',()=>example('named'));$('retry').addEventListener('click',start);
$('original-download').addEventListener('click',()=>download('input.md',original??$('markdown').value,'text/markdown'));
$('request-download').addEventListener('click',()=>download('mdpath-request.json',request()));$('report-download').addEventListener('click',()=>{if(canonical)download('mdpath-report.json',canonical);});$('content-download').addEventListener('click',()=>{if(result?.ok)download('resolved.txt',result.resolved.content,'text/plain');});
$('document-file').addEventListener('change',async()=>{
 const file=$('document-file').files[0];if(!file)return;cancelFile();invalidate();const token=fileToken;
 if(file.size>131072){diagnostic('Document',{message:'Choose a UTF-8 file no larger than 128 KiB.'});$('document-file').value='';return;}
 reading=true;controls();fileTimer=setTimeout(()=>{if(token===fileToken){cancelFile();diagnostic('Document',{message:'Local read exceeded five seconds. Previous draft retained.'});controls();}},5000);
 try{const text=new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(await file.arrayBuffer());if(token===fileToken){$('markdown').value=text;$('document-file').value='';edit('markdown');original=text;}}
 catch{if(token===fileToken)diagnostic('Document',{message:'File could not be read as valid UTF-8. Previous draft retained.'});}
 finally{if(token===fileToken){reading=false;clearTimeout(fileTimer);$('document-file').value='';}controls();}
});
start();
