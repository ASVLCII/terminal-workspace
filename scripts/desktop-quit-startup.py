from pathlib import Path
from pywinauto import Desktop,keyboard
import subprocess,os,json,time,uuid,win32gui,win32con,psutil,sys
root=Path(__file__).resolve().parents[1];data=root/'.work/quit-start-tests'/str(uuid.uuid4());data.mkdir(parents=True)
state=json.loads((root/'.work/ui-test-state/state.json').read_text());workspace=state['workspaces'][0];workspace['terminals']=[];workspace['dock']={'surfaces':[{'Main':{'nodes':[],'focused_node':None,'collapsed':False,'collapsed_leaf_count':0}}],'focused_surface':0};state['settings']['closing']='Ask';(data/'state.json').write_text(json.dumps(state),encoding='utf-8')
shim=data/'slow-cli';shim.mkdir();native=Path('Z:/Tools/CodexCLI/node_modules/@opencode/cli/bin/opencode.exe');assert native.is_file()
(shim/'opencode.cmd').write_text('@echo off\n"C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe" -NoProfile -Command "Start-Sleep -Seconds 7"\n"'+str(native)+'" %*\n',encoding='utf-8')
flavor=sys.argv[1] if len(sys.argv)>1 else 'debug';p=subprocess.Popen([str(root/f'target/{flavor}/terminal-workspace.exe')],cwd=root,env=dict(os.environ,TERMINAL_WORKSPACE_DATA_DIR=str(data),PATH=str(shim)+os.pathsep+os.environ["PATH"]),creationflags=subprocess.CREATE_NO_WINDOW)
try:
 w=Desktop(backend='win32').window(process=p.pid,title='Terminal Workspace');w.wait('visible',timeout=25);w.set_focus();time.sleep(.5);u=Desktop(backend='uia').window(process=p.pid,title='Terminal Workspace')
 candidates=[x for x in u.descendants(control_type='Button') if 'Terminal' in x.window_text() and '+' in x.window_text()];assert candidates,'Terminal menu unavailable';candidates[0].click_input();u.child_window(title='OpenCode',control_type='Button').click_input();win32gui.PostMessage(w.handle,win32con.WM_CLOSE,0,0)
 quit_button=u.child_window(title='Stop and close',control_type='Button');quit_button.wait('visible',timeout=4);quit_button.click_input()
 deadline=time.monotonic()+40
 while p.poll() is None and time.monotonic()<deadline:
  force=u.child_window(title='Force stop and quit',control_type='Button')
  if force.exists(timeout=.1):force.click_input()
  time.sleep(.1)
 p.wait(timeout=2);saved=json.loads((data/'state.json').read_text());terminal=saved['workspaces'][0]['terminals'][0];report={'binary':flavor,'chat_reference_preserved':bool(terminal.get('chat'))};print(json.dumps(report));assert terminal.get('chat'),'Quitting during startup lost the created chat reference'
finally:
 if p.poll() is None:p.terminate();p.wait(timeout=10)
