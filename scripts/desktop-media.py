from pathlib import Path
from pywinauto import Desktop,keyboard,mouse
from PIL import Image
import subprocess,time,os,io,win32clipboard,win32gui,json
root=Path(__file__).resolve().parents[1];state=root/'.work/media-ui-state';state.mkdir(exist_ok=True)
# Use the previously verified two-pane state, with a separate test data directory.
(state/'state.json').write_bytes((root/'.work/ui-test-state/state.json').read_bytes())
p=subprocess.Popen([str(root/'target/debug/terminal-workspace.exe')],cwd=root,env=dict(os.environ,TERMINAL_WORKSPACE_DATA_DIR=str(state)),creationflags=subprocess.CREATE_NO_WINDOW)
w=Desktop(backend='win32').window(process=p.pid,title_re='Terminal Workspace.*');w.wait('visible',timeout=25);w.set_focus();time.sleep(1);r=w.rectangle()
def click(x,y):mouse.click(coords=(r.left+x,r.top+y))
try:
    click(325,245);time.sleep(2);click(420,300)
    b=io.BytesIO();Image.new('RGB',(320,180),(70,120,130)).save(b,format='BMP')
    win32clipboard.OpenClipboard();win32clipboard.EmptyClipboard();win32clipboard.SetClipboardData(win32clipboard.CF_DIB,b.getvalue()[14:]);win32clipboard.CloseClipboard()
    keyboard.send_keys('^+i');time.sleep(2)
    files=list((state/'clipboard').glob('*.png'));w.set_focus();w.capture_as_image().save(root/'artifacts/media-paste-debug.png');assert len(files)==1,'Screenshot was not staged as a retained file'
    keyboard.send_keys('^c');time.sleep(.2)
    fixture=root/'.work/p.png';uri=fixture.as_uri()
    keyboard.send_keys("Clear-Host; Write-Output '"+uri+"'",with_spaces=True,vk_packet=False,pause=.01);keyboard.send_keys('{ENTER}');time.sleep(1)
    click(450,162);time.sleep(1)
    w.set_focus();time.sleep(.1);assert win32gui.GetForegroundWindow()==w.handle
    w.capture_as_image().save(root/'artifacts/media-native.png')
    print(json.dumps({'retained_screenshot':files[0].name,'window_pid':p.pid}))
finally:
    p.terminate();p.wait(timeout=10)


