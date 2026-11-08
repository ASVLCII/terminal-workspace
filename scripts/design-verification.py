from pathlib import Path
from pywinauto import Desktop
import os,subprocess,time,json,uuid
root=Path(__file__).resolve().parents[1];data=root/'.work/design-verification'/str(uuid.uuid4());data.mkdir(parents=True)
zero={'min':{'x':0.,'y':0.},'max':{'x':0.,'y':0.}}
def leaf(pane):return {'Leaf':{'rect':zero,'viewport':zero,'tabs':[pane],'active':0,'scroll':0.,'collapsed':False,'tab_bar_hidden':False}}
def split(f):return {'Horizontal':{'rect':zero,'fraction':f,'fully_collapsed':False,'collapsed_leaf_count':0}}
wid=str(uuid.uuid4());a=str(uuid.uuid4());b=str(uuid.uuid4())
state={'version':1,'selected':wid,'settings':{'font_size':14.,'scrollback':10000,'closing':'Ask'},'workspaces':[{'id':wid,'name':'Daily work','directory':str(root),'terminals':[{'id':a,'name':'OpenCode','tool':'OpenCode','directory':str(root),'chat':None},{'id':b,'name':'PowerShell','tool':'PowerShell','directory':str(root),'chat':None}],'dock':{'surfaces':[{'Main':{'nodes':[split(.69),split(.5),leaf({'Image':str(root/'.work/p.png')}),leaf({'Terminal':a}),leaf({'Terminal':b}),'Empty','Empty'],'focused_node':3,'collapsed':False,'collapsed_leaf_count':0}}],'focused_surface':0}}]}
(data/'state.json').write_text(json.dumps(state),encoding='utf-8')
p=subprocess.Popen([str(root/'target/debug/terminal-workspace.exe')],cwd=root,env=dict(os.environ,TERMINAL_WORKSPACE_DATA_DIR=str(data)),creationflags=subprocess.CREATE_NO_WINDOW)
w=Desktop(backend='win32').window(process=p.pid,title='Terminal Workspace');w.wait('visible',timeout=25);w.set_focus();time.sleep(2);w.capture_as_image().save(root/'artifacts/design-workspace-stopped.png')
try:
 u=Desktop(backend='uia').window(process=p.pid,title='Terminal Workspace')
 buttons=[x for x in u.descendants(control_type='Button') if 'Start terminal' in x.window_text()]
 print('Accessible start buttons:',len(buttons),flush=True)
 for button in buttons:button.click_input();time.sleep(.2)
except Exception as error:print(type(error).__name__,flush=True)
(root/'.work/ui-test-pid').write_text(str(p.pid));(root/'.work/design-state-dir').write_text(str(data));print(p.pid,flush=True)
time.sleep(6);w.set_focus();w.capture_as_image().save(root/'artifacts/design-workspace-running.png')
