"""Release private-byte workloads; no working-set trimming or agent prompts."""
from pathlib import Path
import subprocess,os,time,json,statistics,uuid,psutil,hashlib
from PIL import Image
from pywinauto import Desktop
import win32gui
root=Path(__file__).resolve().parents[1];binary=root/'target/release/terminal-workspace.exe'
fixture=root/'.work/ram-1920x1080.png'
if not fixture.exists():Image.effect_noise((1920,1080),60).convert('RGB').save(fixture)
def sample(p):
 host=p.memory_info().private
 children=[]
 for c in p.children(recursive=True):
  try:children.append({'pid':c.pid,'name':c.name(),'private_bytes':c.memory_info().private})
  except psutil.Error:pass
 return {'app_private_bytes':host,'children':children,'tree_private_bytes':host+sum(c['private_bytes'] for c in children)}
report={'binary':str(binary),'sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'image':{'width':1920,'height':1080,'format':'RGB PNG noise fixture'},'defaults':{'font_size':14,'font_family':'CaskaydiaMono NFM','scrollback':10000},'workloads':{}}
for name,args,budget in [('empty',[],200),('four_shells_image',['--benchmark-workload'],350)]:
 data=root/'.work/ram-tests'/str(uuid.uuid4());data.mkdir(parents=True)
 env=dict(os.environ,TERMINAL_WORKSPACE_DATA_DIR=str(data),TERMINAL_WORKSPACE_BENCH_IMAGE=str(fixture))
 child=subprocess.Popen([str(binary)]+args,cwd=root,env=env,creationflags=subprocess.CREATE_NO_WINDOW);p=psutil.Process(child.pid)
 try:
  w=Desktop(backend='win32').window(process=child.pid,title='Terminal Workspace').wrapper_object() if False else Desktop(backend='win32').window(process=child.pid,title='Terminal Workspace')
  w.wait('visible',timeout=30);time.sleep(12)
  samples=[]
  for _ in range(12):samples.append(sample(p));time.sleep(.5)
  if args:assert sum(c['name'].lower()=='powershell.exe' for c in samples[-1]['children'])==4,'Workload did not start four shells'
  gpu_command="Get-CimInstance Win32_PerfFormattedData_GPUPerformanceCounters_GPUProcessMemory -Filter \"Name LIKE 'pid_"+str(child.pid)+"_%'\" | Select-Object Name,DedicatedUsage,SharedUsage,TotalCommitted | ConvertTo-Json -Compress"
  gpu_run=subprocess.run(['powershell.exe','-NoProfile','-Command',gpu_command],capture_output=True,text=True,timeout=20,creationflags=subprocess.CREATE_NO_WINDOW)
  try:gpu=json.loads(gpu_run.stdout) if gpu_run.returncode==0 and gpu_run.stdout.strip() else {'unavailable':gpu_run.stderr.strip() or 'No per-process GPU counters'}
  except json.JSONDecodeError:gpu={'unavailable':'Counter response was not JSON'}
  peak=max(s['app_private_bytes'] for s in samples);median=statistics.median(s['app_private_bytes'] for s in samples)
  w.set_focus();time.sleep(.1)
  if win32gui.GetForegroundWindow()==w.handle:w.capture_as_image().save(root/f'artifacts/ram-{name}.png')
  report['workloads'][name]={'gpu_counters':gpu,'app_private_mib_median':median/1048576,'app_private_mib_peak':peak/1048576,'budget_mib':budget,'within_budget':peak<=budget*1048576,'samples':samples}
  print(name,round(peak/1048576,2),'MiB',flush=True)
 finally:
  child.terminate();child.wait(timeout=10)
(root/'artifacts/ram-release.json').write_text(json.dumps(report,indent=2))
assert all(w['within_budget'] for w in report['workloads'].values())
