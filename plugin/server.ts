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
    const hook = await ctx.session.hook('prompt', async event => {
      const batch = await call(`/stage/${encodeURIComponent(event.sessionID)}`);
      preparePrompt(event,batch);
    });
    const abort = new AbortController();
    const events = (async () => {
      for await (const event of ctx.event.subscribe({types:['session.inbox.enqueued'],signal:abort.signal})) {
        const item = event.data.item;
        if (item.type !== 'user') continue;
        const revision = item.payload.metadata?.terminalWorkspace?.revision;
        if (typeof revision === 'number') {
          await call('/ack',{sessionID:event.data.sessionID,revision});
        }
      }
    })().catch(() => {});
    return async () => { abort.abort(); await hook.dispose(); await events; };
  },
});
