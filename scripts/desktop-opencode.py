"""Two real OpenCode TUIs, same directory, no prompt submission."""
from pathlib import Path
import subprocess,time,json
from pywinauto import Desktop, keyboard, mouse
root=Path(__file__).resolve().parents[1]
report=root/'artifacts/native-opencode-gate.json'
if report.exists():report.unlink()
process=subprocess.Popen([str(root/'target/debug/terminal-workspace.exe'),'--opencode-gate'],cwd=root,creationflags=subprocess.CREATE_NO_WINDOW)
try:
    window=Desktop(backend='win32').window(process=process.pid,title_re='Terminal Workspace.*')
    window.wait('visible',timeout=45)
    deadline=time.monotonic()+30
    data={}
    while time.monotonic()<deadline:
        if report.exists():
            try:data=json.loads(report.read_text())
            except (ValueError,PermissionError):pass
        if data.get('connected')==2 or data.get('error'):break
        time.sleep(.25)
    window.capture_as_image().save(root/'artifacts/native-opencode-gate.png')
    if data.get('connected')!=2:
        window.set_focus();rect=window.rectangle();mouse.click(coords=(rect.left+170,rect.top+610))
        keyboard.send_keys('^p');time.sleep(.5);keyboard.send_keys('plugin',pause=.1);time.sleep(.5);keyboard.send_keys('{ENTER}');time.sleep(1)
        keyboard.send_keys('{ENTER}');time.sleep(.7)
        window.capture_as_image().save(root/'artifacts/native-opencode-plugin-error.png')
    print(json.dumps(data))
    assert not data.get('error'),data
    assert data.get('connected')==2,'Both actual OpenCode TUI plugins must bind'
    assert data['staged']==[['notes.txt','folder','screenshot.png'],[]],data
finally:
    if process.poll() is None:
        process.terminate()
        process.wait(timeout=10)
