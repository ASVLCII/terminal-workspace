from pathlib import Path
import subprocess,os,uuid,time,psutil,json
root=Path.cwd();results=[]
for backend in ['dx12','vulkan']:
 data=root/'.work/renderer-tests'/str(uuid.uuid4());data.mkdir(parents=True)
 env=dict(os.environ,TERMINAL_WORKSPACE_DATA_DIR=str(data),WGPU_BACKEND=backend)
 p=subprocess.Popen([str(root/'target/release/terminal-workspace.exe')],cwd=root,env=env,creationflags=subprocess.CREATE_NO_WINDOW)
 try:
  time.sleep(7)
  if p.poll() is None:results.append({'backend':backend,'private_mib':psutil.Process(p.pid).memory_info().private/1048576});print(results[-1],flush=True)
  else:results.append({'backend':backend,'exit':p.returncode});print(results[-1],flush=True)
 finally:
  if p.poll() is None:p.terminate();p.wait(timeout=10)
(root/'artifacts/renderer-memory.json').write_text(json.dumps(results,indent=2))
