from pathlib import Path
from pywinauto import Desktop
import subprocess,os,time,win32gui
root=Path(__file__).resolve().parents[1];state=root/'.work/closing-ui-state';state.mkdir(exist_ok=True);(state/'state.json').write_bytes((root/'.work/ui-test-state/state.json').read_bytes())
p=subprocess.Popen([str(root/'target/debug/terminal-workspace.exe')],cwd=root,env=dict(os.environ,TERMINAL_WORKSPACE_DATA_DIR=str(state)),creationflags=subprocess.CREATE_NO_WINDOW)
w=Desktop(backend='win32').window(process=p.pid,title_re='Terminal Workspace.*');w.wait('visible',timeout=25);w.set_focus();time.sleep(1);w.close();time.sleep(.5);w.set_focus();assert win32gui.GetForegroundWindow()==w.handle;w.capture_as_image().save(root/'artifacts/closing-native.png');(root/'.work/ui-test-pid').write_text(str(p.pid));print(p.pid)
