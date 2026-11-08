from pathlib import Path
from pywinauto import Desktop
import win32gui,time
root=Path(__file__).resolve().parents[1];pid=int((root/'.work/ui-test-pid').read_text());w=Desktop(backend='win32').window(process=pid,title_re='Terminal Workspace.*')
w.restore();w.set_focus();time.sleep(.3)
assert win32gui.GetForegroundWindow()==w.handle,'Own window is not foreground; capture cancelled'
w.capture_as_image().save(root/'artifacts/workspace-columns-verified.png')
print(w.rectangle())
