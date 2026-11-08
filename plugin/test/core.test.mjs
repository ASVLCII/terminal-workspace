import test from 'node:test';
import assert from 'node:assert/strict';
import { preparePrompt } from '../core.mjs';

test('append files only to the bound session without replacing typed text', () => {
  const event = { sessionID: 'a', messageID: 'm1', prompt: { text: 'keep my text', files: [{uri:'file:///existing.txt'}] } };
  preparePrompt(event, {sessionID:'a',revision:1,files:[{uri:'file:///layout.png'},{uri:'file:///src/'}]});
  assert.equal(event.prompt.text, 'keep my text');
  assert.deepEqual(event.prompt.files.map(f=>f.uri), ['file:///existing.txt','file:///layout.png','file:///src/']);
  assert.equal(event.metadata.terminalWorkspace.revision, 1);
});

test('same-directory different session is not changed', () => {
  const event = {sessionID:'b',messageID:'m2',prompt:{text:'untouched'}};
  preparePrompt(event,{sessionID:'a',revision:1,files:[{uri:'file:///layout.png'}]});
  assert.deepEqual(event,{sessionID:'b',messageID:'m2',prompt:{text:'untouched'}});
});

test('durably admitted references are not repeated while acknowledgement is retried', () => {
 const event={sessionID:'a',prompt:{text:'second prompt'}};
 preparePrompt(event,{sessionID:'a',revision:2,files:[{uri:'file:///old',revision:1},{uri:'file:///new',revision:2}]},1);
 assert.deepEqual(event.prompt.files,[{uri:'file:///new'}]);
});
