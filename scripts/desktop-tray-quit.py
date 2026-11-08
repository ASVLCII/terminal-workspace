from pathlib import Path
from pywinauto import Desktop,mouse,keyboard
import win32gui,win32process,win32con,time,json,psutil
root=Path.cwd();pid=int((root/'.work/ui-test-pid').read_text());w=Desktop(backend='win32').window(process=pid,title='Terminal Workspace').wrapper_object();w.set_focus();time.sleep(.3);r=w.rectangle()
mouse.click(coords=(r.left+75,r.top+145));time.sleep(.4);mouse.click(coords=(r.left+330,r.top+246));time.sleep(2)
children=[c.pid for c in psutil.Process(pid).children(recursive=True) if c.name().lower()=='powershell.exe'];assert children,children
trays=[]
win32gui.EnumWindows(lambda h,_:trays.append(h) if win32process.GetWindowThreadProcessId(h)[1]==pid and win32gui.GetClassName(h)=='tray_icon_app' else None,None)
assert len(trays)==1;win32gui.PostMessage(trays[0],6002,0,win32con.WM_RBUTTONUP);time.sleep(.5);keyboard.send_keys('{END}{ENTER}');time.sleep(.6)
w.set_focus();time.sleep(.1);w.capture_as_image().save(root/'artifacts/tray-quit-warning.png');print(json.dumps({'owned_shell_pids':children,'tray_menu_quit_dispatched':True}));(root/'.work/quit-children.json').write_text(json.dumps(children))
