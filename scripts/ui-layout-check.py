from pathlib import Path
from pywinauto import Desktop,mouse,keyboard
import psutil,time,json
root=Path(__file__).resolve().parents[1];pid=int((root/'.work/ui-test-pid').read_text());w=Desktop(backend='win32').window(process=pid,title_re='Terminal Workspace.*');w.set_focus();r=w.rectangle()
def click(x,y):mouse.click(coords=(r.left+x,r.top+y))
def children():return sorted(c.pid for c in psutil.Process(pid).children(recursive=True) if c.name().lower()=='powershell.exe')
before=children();assert len(before)==2,before
# Resize the straight divider, maximize the right pane, restore it.
mouse.move(coords=(r.left+776,r.top+400));mouse.press();mouse.move(coords=(r.left+850,r.top+400));mouse.release();time.sleep(.4)
w.capture_as_image().save(root/'artifacts/workspace-columns.png')
click(930,126);time.sleep(.3);keyboard.send_keys('{ESC}');time.sleep(.3)
after=children();assert before==after,(before,after)
time.sleep(1);state=json.loads((root/'.work/ui-test-state/state.json').read_text());print(json.dumps({'two_shells':len(after),'same_processes_after_layout':before==after,'saved_terminals':len(state['workspaces'][0]['terminals'])}))

