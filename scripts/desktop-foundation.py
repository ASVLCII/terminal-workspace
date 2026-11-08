"""Exercise real app-owned terminal input; no agent prompt is submitted."""
from pathlib import Path
import subprocess, time, json
from pywinauto import Desktop, keyboard, mouse

root = Path(__file__).resolve().parents[1]
artifacts = root / 'artifacts'
artifacts.mkdir(exist_ok=True)
marker = root / '.work' / 'keyboard-away-from-pointer.txt'
if marker.exists():
    marker.unlink()
process = subprocess.Popen([str(root/'target/debug/terminal-workspace.exe'),'--foundation'], cwd=root, creationflags=subprocess.CREATE_NO_WINDOW)
try:
    window = Desktop(backend='win32').window(process=process.pid, title_re='Terminal Workspace.*')
    window.wait('visible', timeout=25)
    window.set_focus()
    time.sleep(2)
    rect = window.rectangle()
    mouse.click(coords=(rect.left+200, rect.top+220))
    mouse.move(coords=(rect.left+400,rect.top+12))
    command = "Set-Content -LiteralPath '.work/keyboard-away-from-pointer.txt' -Value 'typed-with-pointer-away'"
    keyboard.send_keys(command, with_spaces=True, pause=0.015, vk_packet=False)
    keyboard.send_keys('{ENTER}')
    deadline=time.monotonic()+8
    while not marker.exists() and time.monotonic()<deadline:
        time.sleep(.1)
    window.capture_as_image().save(artifacts/'foundation.png')
    report={'keyboard_away_from_pointer':marker.exists(),'pid':process.pid}
    (artifacts/'desktop-foundation.json').write_text(json.dumps(report,indent=2))
    print(json.dumps(report))
    assert marker.exists(), 'Focused terminal discarded input when pointer left its bounds'
finally:
    if process.poll() is None:
        process.terminate()
        process.wait(timeout=10)
