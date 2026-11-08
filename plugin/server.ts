import { Plugin } from '@opencode/plugin';
import { preparePrompt } from './core.mjs';

export default Plugin.define({
  id: 'terminal-workspace.bridge',
  async setup(ctx) {
    const endpoint = process.env.TERMINAL_WORKSPACE_BRIDGE;
    const token = process.env.TERMINAL_WORKSPACE_TOKEN;
    if (!endpoint || !token) return;
    const call = async (path: string, body?: unknown) => {
      const result = await fetch(`${endpoint}${path}`, {
        method: body === undefined ? 'GET' : 'POST',
        headers: {Authorization:`Bearer ${token}`,'Content-Type':'application/json'},
        body: body === undefined ? undefined : JSON.stringify(body),
        signal: AbortSignal.timeout(2000),
      });
      if (!result.ok) throw new Error(`Workspace bridge unavailable (${result.status})`);
      return result.json();
    };
    await call('/plugin', {version:1,kind:'server',opencode:ctx.app.version});
    const admitted = new Map<string,number>();
    const acknowledgements = new Map<string,number>();
    const retry = setInterval(async () => {for(const [sessionID,revision] of acknowledgements){try{await call('/ack',{sessionID,revision});if(acknowledgements.get(sessionID)===revision)acknowledgements.delete(sessionID);}catch{}}},1000);
    const hook = await ctx.session.hook('prompt', async event => {
      const batch = await call(`/stage/${encodeURIComponent(event.sessionID)}`);
      for(const file of batch.files){if(/^file:/i.test(file.uri)&&/\.(png|jpe?g|gif|webp|bmp)$/i.test(file.uri)){await call('/image',{sessionID:event.sessionID,uri:file.uri});}}
      preparePrompt(event,batch,admitted.get(event.sessionID) ?? 0);
    });
    const abort = new AbortController();
    const events = (async () => {
      for await (const event of ctx.event.subscribe({types:['session.inbox.enqueued'],signal:abort.signal})) {
        const item = event.data.item;
        if (item.type !== 'user') continue;
        const revision = item.payload.metadata?.terminalWorkspace?.revision;
        if (typeof revision === 'number') {
          const sessionID=event.data.sessionID;
          admitted.set(sessionID,Math.max(admitted.get(sessionID) ?? 0,revision));
          acknowledgements.set(sessionID,Math.max(acknowledgements.get(sessionID) ?? 0,revision));
          await call('/ack',{sessionID,revision}).then(()=>{if(acknowledgements.get(sessionID)===revision)acknowledgements.delete(sessionID);}).catch(()=>{});
        }
      }
    })().catch(() => {});
    return async () => { clearInterval(retry);abort.abort(); await hook.dispose(); await events; };
  },
});
