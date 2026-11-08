export function preparePrompt(event, batch) {
  if (!batch || batch.sessionID !== event.sessionID || !batch.files.length) return event;
  const existing = event.prompt.files ?? [];
  event.prompt.files = [...existing];
  for (const file of batch.files) {
    if (!event.prompt.files.some(current => current.uri === file.uri)) {
      event.prompt.files.push({uri:file.uri});
    }
  }
  event.metadata = {...event.metadata,terminalWorkspace:{revision:batch.revision}};
  return event;
}
