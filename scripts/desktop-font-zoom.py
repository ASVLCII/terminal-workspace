from pathlib import Path
from pywinauto import Desktop,mouse,keyboard
import time,json,psutil,win32gui
root=Path.cwd();pid=int((root/'.work/ui-test-pid').read_text());w=Desktop(backend='win32').window(process=pid,title='Terminal Workspace').wrapper_object();u=Desktop(backend='uia').window(process=pid,title='Terminal Workspace');w.set_focus();time.sleep(.3);r=w.rectangle()
def percentages():return [c.window_text() for c in u.descendants(control_type='Text') if c.window_text().endswith('%')]
before=percentages();mouse.scroll(coords=(r.right-160,r.top+420),wheel_dist=4);time.sleep(1);after=percentages();w.set_focus();time.sleep(.1);assert win32gui.GetForegroundWindow()==w.handle;w.capture_as_image().save(root/'artifacts/design-wheel-zoom.png')
keyboard.send_keys('^,');time.sleep(.7);w.set_focus();time.sleep(.1);w.capture_as_image().save(root/'artifacts/design-settings.png')
assert before != after, 'Wheel did not change image zoom'
data=Path((root/'.work/design-state-dir').read_text(encoding='utf-8'))
owned={child.pid for child in psutil.Process(pid).children(recursive=True)}
for family in ['Consolas','CaskaydiaMono NFM']:
 u.descendants(control_type='ComboBox')[0].click_input()
 u.child_window(title=family,control_type='Button').click_input()
 time.sleep(1.2)
 assert json.loads((data/'state.json').read_text(encoding='utf-8'))['settings']['font_family']==family
assert owned <= {child.pid for child in psutil.Process(pid).children(recursive=True)}, 'Font change restarted a process'
w.set_focus();assert win32gui.GetForegroundWindow()==w.handle
w.capture_as_image().save(root/'artifacts/design-settings.png')
print(json.dumps({'zoom_before':before,'zoom_after':after,'font_changes_saved':True,'existing_processes_preserved':True}))
