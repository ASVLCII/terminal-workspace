"""Install/upgrade/uninstall only the isolated local installer smoke target."""
from pathlib import Path
import subprocess,os,time,uuid,json,winreg
root=Path(__file__).resolve().parents[1]
target=(root/'.work/installer-smoke').resolve()
assert target.is_relative_to((root/'.work').resolve())
try:
 with winreg.OpenKey(winreg.HKEY_CURRENT_USER,r'Software\TerminalWorkspace') as key:
  existing=Path(winreg.QueryValueEx(key,'InstallDir')[0]).resolve()
  assert existing==target, f'Refusing to replace another installation: {existing}'
except FileNotFoundError:pass
installer=root/'dist/TerminalWorkspace-0.1.0-dev-x64-setup.exe'
assert installer.is_file()
target.mkdir(parents=True,exist_ok=True)
temp=root/'.work/installer-temp';temp.mkdir(exist_ok=True)
env=dict(os.environ,TEMP=str(temp),TMP=str(temp))
data=Path(os.environ['LOCALAPPDATA'])/'TerminalWorkspace'
data.mkdir(parents=True,exist_ok=True)
canary=data/('installer-preservation-'+str(uuid.uuid4())+'.txt')
canary.write_text('retain application data',encoding='utf-8')
extra=target/'keep-user-file.txt';extra.write_text('retain unknown user file',encoding='utf-8')
shortcut=Path(os.environ['APPDATA'])/'Microsoft/Windows/Start Menu/Programs/Terminal Workspace.lnk'
report={}
try:
 for phase in ['install','upgrade']:
  subprocess.run([str(installer),'/S','/D='+str(target)],env=env,check=True,timeout=60)
  assert (target/'terminal-workspace.exe').is_file()
  assert (target/'plugin/server.ts').is_file()
  assert shortcut.is_file()
  assert canary.read_text(encoding='utf-8')=='retain application data'
  assert extra.read_text(encoding='utf-8')=='retain unknown user file'
  report[phase]=True
 subprocess.run([str(target/'Uninstall.exe'),'/S'],env=env,check=True,timeout=60)
 deadline=time.monotonic()+15
 # NSIS delegates to a temporary uninstaller; its initial process may exit first.
 while any(path.exists() for path in [target/'terminal-workspace.exe',target/'plugin/server.ts',target/'Uninstall.exe',shortcut]) and time.monotonic()<deadline:time.sleep(.2)
 assert not (target/'terminal-workspace.exe').exists()
 assert not (target/'plugin/server.ts').exists()
 assert not shortcut.exists(), 'Start-menu shortcut was left behind'
 assert canary.is_file() and extra.is_file()
 try:
  with winreg.OpenKey(winreg.HKEY_CURRENT_USER,r'Software\Microsoft\Windows\CurrentVersion\Uninstall\TerminalWorkspace'):raise AssertionError('Uninstall registry entry remains')
 except FileNotFoundError:pass
 report.update(uninstall=True,shortcut_removed=True,user_data_retained=True,unknown_files_retained=True)
 print(json.dumps(report))
 (root/'artifacts/installer-smoke.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
finally:
 canary.unlink(missing_ok=True)
