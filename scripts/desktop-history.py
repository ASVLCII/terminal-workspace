from pathlib import Path
from pywinauto import Desktop,mouse
import subprocess,os,time,win32gui
root=Path(__file__).resolve().parents[1]
p=subprocess.Popen([str(root/'target/debug/terminal-workspace.exe')],cwd=root,env=dict(os.environ,TERMINAL_WORKSPACE_DATA_DIR=str(root/'.work/ui-test-state')),creationflags=subprocess.CREATE_NO_WINDOW)
try:
 w=Desktop(backend='win32').window(process=p.pid,title_re='Terminal Workspace.*');w.wait('visible',timeout=25);w.set_focus();time.sleep(1);r=w.rectangle();mouse.click(coords=(r.left+65,r.top+287));time.sleep(10);w.set_focus();assert win32gui.GetForegroundWindow()==w.handle;w.capture_as_image().save(root/'artifacts/history-native.png');print('History window captured')
finally:p.terminate();p.wait(timeout=10)
