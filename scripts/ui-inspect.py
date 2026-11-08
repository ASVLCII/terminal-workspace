from pathlib import Path
import subprocess,time,os
from pywinauto import Desktop, keyboard
root=Path(__file__).resolve().parents[1]
env=dict(os.environ,TERMINAL_WORKSPACE_DATA_DIR=str(root/'.work/ui-test-state'))
p=subprocess.Popen([str(root/'target/debug/terminal-workspace.exe')],cwd=root,env=env,creationflags=subprocess.CREATE_NO_WINDOW)
w=Desktop(backend='win32').window(process=p.pid,title_re='Terminal Workspace.*')
w.wait('visible',timeout=25);w.set_focus();time.sleep(1)
w.capture_as_image().save(root/'artifacts/workspace-empty.png')
keyboard.send_keys('^+n');time.sleep(.5)
w.capture_as_image().save(root/'artifacts/workspace-form.png')
(root/'.work/ui-test-pid').write_text(str(p.pid))
print(p.pid)
