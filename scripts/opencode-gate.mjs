// Real isolated OpenCode process and its plugin loader; never invokes a model.
import { spawn } from 'node:child_process';
import { createServer } from 'node:http';
import { randomUUID } from 'node:crypto';
import { mkdir, writeFile, readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import assert from 'node:assert/strict';

const root=resolve(import.meta.dirname,'..');
await mkdir(resolve(root,'artifacts'),{recursive:true});
const token=randomUUID();
const requests=[];
const staged=new Map();
const bridge=createServer(async(req,res)=>{
  if(req.headers.authorization!==`Bearer ${token}`){res.writeHead(401).end();return;}
  let text='';for await(const chunk of req) text+=chunk;
  const data=text?JSON.parse(text):undefined;
  requests.push({method:req.method,path:req.url,data});
  res.setHeader('content-type','application/json');
  if(req.url.startsWith('/stage/')) {
    const sessionID=decodeURIComponent(req.url.slice(7));
    res.end(JSON.stringify({sessionID,revision:1,files:staged.get(sessionID)??[]}));
  } else res.end(JSON.stringify({version:1,ok:true}));
});
await new Promise(r=>bridge.listen(0,'127.0.0.1',r));
const bridgeURL=`http://127.0.0.1:${bridge.address().port}`;
const ocExe=process.env.OPENCODE_EXE ?? 'Z:/Tools/CodexCLI/node_modules/@opencode/cli/bin/opencode.exe';
const child=spawn(ocExe,['serve','--stdio','--hostname','127.0.0.1','--port','0'],{
  cwd:root,windowsHide:true,stdio:['pipe','pipe','pipe'],
  env:{...process.env,OPENCODE_PASSWORD:token,OPENCODE_SERVER_PASSWORD:token,
    OPENCODE_CONFIG_CONTENT:JSON.stringify({plugins:[resolve(root,'plugin')]}),
    TERMINAL_WORKSPACE_BRIDGE:bridgeURL,TERMINAL_WORKSPACE_TOKEN:token,
    OPENCODE_DISABLE_MODELS_FETCH:'1'},
});
let stderr='';child.stderr.on('data',b=>stderr+=b);
const serverURL=await new Promise((ok,fail)=>{
  let text='';const timer=setTimeout(()=>fail(new Error('Backend startup timeout: '+stderr.slice(-2000))),25000);
  child.stdout.on('data',b=>{text+=b;for(const line of text.split('\n')){try{const row=JSON.parse(line);if(row.url){clearTimeout(timer);ok(row.url);}}catch{}}});
  child.on('exit',code=>{clearTimeout(timer);fail(new Error(`OpenCode exited ${code}: ${stderr.slice(-2000)}`));});
});
const headers={Authorization:`Basic ${Buffer.from(`opencode:${token}`).toString('base64')}`,'Content-Type':'application/json'};
async function api(path,body,method) {
  const result=await fetch(`${serverURL}${path}`,{method:method??(body?'POST':'GET'),headers,body:body?JSON.stringify(body):undefined,signal:AbortSignal.timeout(15000)});
  const text=await result.text();if(!result.ok)throw new Error(`${result.status} ${path}: ${text.slice(0,1200)}`);
  return text?JSON.parse(text):null;
}
const sessions=[];
try {
  const info=await api('/api/info');
  const a=await api('/api/session',{title:'Terminal Workspace gate A',location:{directory:root}});
  const b=await api('/api/session',{title:'Terminal Workspace gate B',location:{directory:root}});
  sessions.push(a.data?.id??a.id,b.data?.id??b.id);
  let plugins;
  for(let i=0;i<20;i++) {
    plugins=await api(`/api/plugin?location[directory]=${encodeURIComponent(root)}`);
    if(requests.some(r=>r.path==='/plugin')) break;
    await new Promise(r=>setTimeout(r,200));
  }
  const fixture=resolve(root,'.work/gate-fixtures');
  await mkdir(resolve(fixture,'folder'),{recursive:true});
  await writeFile(resolve(fixture,'notes.txt'),'Gate fixture, no model execution.');
  await writeFile(resolve(fixture,'screenshot.png'),await readFile(resolve(root,'docs/design/01-workspace-and-layouts-v2.png')));
  staged.set(sessions[0],['notes.txt','folder','screenshot.png'].map(name=>({uri:pathToFileURL(resolve(fixture,name)).href,name})));
  const beforeA=await api(`/api/session/${sessions[0]}/inbox`);
  const beforeB=await api(`/api/session/${sessions[1]}/inbox`);
  const admitted=await api(`/api/session/${sessions[0]}/prompt`,{text:'Do not run: attachment admission fixture',resume:false});
  const afterB=await api(`/api/session/${sessions[1]}/inbox`);
  const report={info,sessions,plugins:plugins.data.filter(p=>p.id.startsWith('terminal-workspace')),requests,beforeA,beforeB,admitted,afterB,stderr:stderr.slice(-4000),modelExecutionRequested:false};
  await writeFile(resolve(root,'artifacts/opencode-gate.json'),JSON.stringify(report,null,2));
  console.log(JSON.stringify({sessions,plugin:report.plugins,handshake:requests.some(r=>r.path==='/plugin'),admittedFiles:(admitted.data?.payload?.files??[]).map(f=>({name:f.name,mime:f.mime})),otherChatUnchanged:JSON.stringify(afterB)===JSON.stringify(beforeB),modelExecutionRequested:false},null,2));
  if(!requests.some(r=>r.path==='/plugin')) throw new Error('Plugin did not complete host handshake');
  assert.equal(admitted.payload?.files?.length??admitted.data?.payload?.files?.length,3,'file, folder, screenshot admitted to A');
  assert.deepEqual(afterB,beforeB,'Other terminal session changed');
} finally {
  for(const id of sessions.filter(Boolean)){try{await api(`/api/session/${encodeURIComponent(id)}`,undefined,'DELETE');}catch{}}
  child.stdin.end();bridge.close();
  setTimeout(()=>{if(child.exitCode===null)child.kill();},3000).unref();
}
