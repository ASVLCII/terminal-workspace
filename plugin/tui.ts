import { Plugin } from '@opencode/plugin/tui';

export default Plugin.define({
  id:'terminal-workspace.tui',
  async setup(ctx) {
    const endpoint = process.env.TERMINAL_WORKSPACE_BRIDGE;
    const token = process.env.TERMINAL_WORKSPACE_TOKEN;
    const terminalID = process.env.TERMINAL_WORKSPACE_TERMINAL;
    if (!endpoint || !token || !terminalID) return;
    await fetch(`${endpoint}/plugin`,{method:'POST',headers:{Authorization:`Bearer ${token}`,'Content-Type':'application/json'},body:JSON.stringify({version:1,kind:'tui',terminalID})});
    let revision = -1;
    let stopped = false;
    let pending = false;
    const poll = async () => {
      if(stopped || pending) return;
      pending=true;
      try {
        const route = ctx.ui.router.current();
        if(route.type !== 'session') return;
        const result = await fetch(`${endpoint}/tui`,{
          method:'POST', headers:{Authorization:`Bearer ${token}`,'Content-Type':'application/json'},
          body:JSON.stringify({version:1,terminalID,sessionID:route.sessionID}),signal:AbortSignal.timeout(1500),
        });
        if(!result.ok) throw new Error('connection');
        const batch = await result.json();
        if(batch.revision !== revision && batch.files.length) {
          ctx.ui.toast.show({message:`${batch.files.length} attachments staged for this chat`,variant:'info'});
        }
        revision=batch.revision;
      } catch (error) {
        await fetch(`${endpoint}/plugin-error`,{method:'POST',headers:{Authorization:`Bearer ${token}`,'Content-Type':'application/json'},body:JSON.stringify({message:String(error)})}).catch(()=>{});
      } finally {pending=false;}
    };
    await poll();
    const timer = setInterval(poll,500);
    return () => {stopped=true; clearInterval(timer);};
  },
});
