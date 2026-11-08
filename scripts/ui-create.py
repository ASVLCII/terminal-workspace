from pathlib import Path
from pywinauto import Desktop,keyboard,mouse
import time,json
root=Path(__file__).resolve().parents[1];pid=int((root/'.work/ui-test-pid').read_text())
w=Desktop(backend='win32').window(process=pid,title_re='Terminal Workspace.*');w.set_focus();r=w.rectangle()
def click(x,y):mouse.click(coords=(r.left+x,r.top+y))
click(540,400);keyboard.send_keys('Native test',with_spaces=True);click(540,463);keyboard.send_keys(str(root),with_spaces=True,vk_packet=False);click(565,525);time.sleep(3)
w.capture_as_image().save(root/'artifacts/workspace-running.png')
click(500,300);keyboard.send_keys("Set-Content -LiteralPath '.work/workspace-terminal.txt' -Value 'real-terminal'",with_spaces=True,vk_packet=False,pause=.01);keyboard.send_keys('{ENTER}');time.sleep(2)
print(json.dumps({'command_executed':(root/'.work/workspace-terminal.txt').exists(),'state_saved':(root/'.work/ui-test-state/state.json').exists()}))
