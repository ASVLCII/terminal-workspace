"""A history search must never paste/execute text in a neighboring terminal."""
from pathlib import Path
from pywinauto import Desktop,mouse,keyboard
import subprocess,os,time,json,uuid,sys,win32clipboard,win32gui
root=Path(__file__).resolve().parents[1];data=root/'.work/focus-tests'/str(uuid.uuid4());data.mkdir(parents=True)
state=json.loads((root/'.work/ui-test-state/state.json').read_text());nodes=state['workspaces'][0]['dock']['surfaces'][0]['Main']['nodes'];nodes[1]['Leaf']['tabs']=[{'Terminal':state['workspaces'][0]['terminals'][0]['id']}];nodes[1]['Leaf']['active']=0;nodes[2]['Leaf']['tabs']=['History'];nodes[2]['Leaf']['active']=0;state['settings']['closing']='Ask';(data/'state.json').write_text(json.dumps(state))
marker=data/'must-not-execute.txt';binary=root/('target/'+(sys.argv[1] if len(sys.argv)>1 else 'debug')+'/terminal-workspace.exe')
p=subprocess.Popen([str(binary)],cwd=root,env=dict(os.environ,TERMINAL_WORKSPACE_DATA_DIR=str(data),WGPU_BACKEND='dx12'),creationflags=subprocess.CREATE_NO_WINDOW)
try:
 w=Desktop(backend='win32').window(process=p.pid,title='Terminal Workspace');w.wait('visible',timeout=25);w.set_focus();time.sleep(1);r=w.rectangle()
 def click(x,y):mouse.click(coords=(r.left+x,r.top+y));time.sleep(.25)
 click(330,246);time.sleep(2);click(450,300);time.sleep(.3);click(900,209);time.sleep(.4)
 command="Set-Content -LiteralPath '"+str(marker)+"' -Value 'wrong-recipient'"
 win32clipboard.OpenClipboard();win32clipboard.EmptyClipboard();win32clipboard.SetClipboardText(command);win32clipboard.CloseClipboard();keyboard.send_keys('^v');time.sleep(.3);keyboard.send_keys('{ENTER}');time.sleep(1.5)
 w.set_focus();time.sleep(.1)
 if win32gui.GetForegroundWindow()==w.handle:w.capture_as_image().save(root/f'artifacts/focus-{sys.argv[1] if len(sys.argv)>1 else "debug"}.png')
 print(json.dumps({'binary':str(binary),'search_did_not_execute_a_shell_command':not marker.exists()}));assert not marker.exists(),'History search was delivered to the neighboring terminal'
finally:
 p.terminate();p.wait(timeout=10)
