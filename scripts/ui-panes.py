from pathlib import Path
import subprocess,time,os,json
from pywinauto import Desktop,keyboard,mouse
root=Path(__file__).resolve().parents[1];state=root/'.work/ui-test-state';env=dict(os.environ,TERMINAL_WORKSPACE_DATA_DIR=str(state))
p=subprocess.Popen([str(root/'target/debug/terminal-workspace.exe')],cwd=root,env=env,creationflags=subprocess.CREATE_NO_WINDOW)
w=Desktop(backend='win32').window(process=p.pid,title_re='Terminal Workspace.*');w.wait('visible',timeout=25);w.set_focus();time.sleep(1);r=w.rectangle()
def click(x,y):mouse.click(coords=(r.left+x,r.top+y))
# Reopening shows a stopped definition; explicit Start launches it.
w.capture_as_image().save(root/'artifacts/workspace-restored.png')
click(350,210);time.sleep(2)
keyboard.send_keys('^+t');time.sleep(2)
click(480,53);time.sleep(.3)
w.capture_as_image().save(root/'artifacts/layout-menu.png')
(root/'.work/ui-test-pid').write_text(str(p.pid));print(p.pid)
