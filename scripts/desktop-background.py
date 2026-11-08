from pathlib import Path
from pywinauto import Desktop,mouse
import subprocess,time,os,json,win32con,win32gui
root=Path.cwd();pid=int((root/'.work/ui-test-pid').read_text());w=Desktop(backend='win32').window(process=pid,title='Terminal Workspace');w.set_focus();r=w.rectangle()
def click(x,y):mouse.click(coords=(r.left+x,r.top+y))
click(420,438);click(545,493);time.sleep(1)
assert not win32gui.IsWindowVisible(w.handle),'Background choice did not hide window'
state=json.loads((root/'.work/closing-ui-state/state.json').read_text());assert state['settings']['closing']=='Background',state['settings']
# A second launch must activate the same window, rather than create a second app.
q=subprocess.Popen([str(root/'target/debug/terminal-workspace.exe')],cwd=root,env=dict(os.environ,TERMINAL_WORKSPACE_DATA_DIR=str(root/'.work/closing-ui-state')),creationflags=subprocess.CREATE_NO_WINDOW);assert q.wait(timeout=5)==0;time.sleep(.7);assert win32gui.IsWindowVisible(w.handle)
win32gui.PostMessage(w.handle,win32con.WM_CLOSE,0,0);time.sleep(1);assert not win32gui.IsWindowVisible(w.handle),'Saved background action was not applied'
q=subprocess.Popen([str(root/'target/debug/terminal-workspace.exe')],cwd=root,creationflags=subprocess.CREATE_NO_WINDOW);q.wait(timeout=5);time.sleep(.5)
print(json.dumps({'remembered_background':True,'same_instance_reopened':True,'subsequent_close_hides':True,'pid':pid}))
