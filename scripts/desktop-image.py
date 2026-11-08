from pathlib import Path
import subprocess,time,json,shutil
from pywinauto import Desktop,keyboard,mouse
root=Path(__file__).resolve().parents[1]
fixture=root/'.work/p.png'
shutil.copyfile(root/'docs/design/01-workspace-and-layouts-v2.png',fixture)
report=root/'artifacts/foundation-preview.json'
if report.exists():report.unlink()
log=open(root/'artifacts/desktop-image.stderr','w')
process=subprocess.Popen([str(root/'target/debug/terminal-workspace.exe'),'--foundation'],cwd=root,creationflags=subprocess.CREATE_NO_WINDOW,stdout=log,stderr=log)
try:
    window=Desktop(backend='win32').window(process=process.pid,title_re='Terminal Workspace.*');window.wait('visible',timeout=25);window=window.wrapper_object();window.set_focus();time.sleep(1)
    rect=window.rectangle();mouse.click(coords=(rect.left+170,rect.top+220))
    command=f"Clear-Host; Write-Output '{fixture.as_uri()}'"
    keyboard.send_keys(command,with_spaces=True,pause=.01,vk_packet=False);keyboard.send_keys('{ENTER}');time.sleep(1)
    # Clear-Host puts the emitted URI at the first terminal row.
    mouse.move(coords=(rect.left+65,rect.top+110));time.sleep(.3);mouse.click(coords=(rect.left+65,rect.top+110))
    deadline=time.monotonic()+5
    while not report.exists() and time.monotonic()<deadline:time.sleep(.1)
    window.capture_as_image().save(root/'artifacts/foundation-image.png')
    assert report.exists(),'Image link did not open native preview'
    print(report.read_text())
finally:
    try:window.close();process.wait(timeout=7)
    except Exception:
        if process.poll() is None:process.terminate();process.wait(timeout=7)
    log.close()
