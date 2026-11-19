import {test} from 'node:test';
import assert from 'node:assert/strict';
import {existsSync} from 'node:fs';
import {pscWorkstreams} from '../../src/data/psc-workstreams.ts';
import {emptyNotebook,parseNotebook,referenceUrl,noteCurrent} from '../../src/lib/notebook-state.ts';
test('PSC workstreams have stable unique IDs and concrete source-linked outputs',()=>{
  assert.equal(new Set(pscWorkstreams.map(w=>w.id)).size,pscWorkstreams.length);
  assert.deepEqual(pscWorkstreams.flatMap(w=>{const match=w.scope.match(/^CE 15\.(\d+)/);return match?[Number(match[1])]:[];}).sort((a,b)=>a-b),Array.from({length:16},(_,i)=>i+1));
  for(const w of pscWorkstreams){assert.ok(w.output.length>45);assert.ok(w.targets.length>0);assert.ok(existsSync(`src/content/docs/${w.lesson.replace(/\/$/,'')}.mdx`)||existsSync(`src/content/docs/${w.lesson}index.mdx`),w.lesson);}
});
test('notebook state round-trips real IDs and rejects oversized or unsafe entries',()=>{
  const now=1_700_000_000_000;const state=emptyNotebook();
  state.work['shared-awareness']={status:'learning',notes:'Check the current plan.',sourceUrl:'https://example.com/report',checkedDate:'2023-11-01',updated:now};
  state.mistakes['nec-original-01']={cause:'calculation',correction:'Use V squared / R.',nextAction:'Redo with another resistance.',reviewed:true,updated:now};
  assert.deepEqual(parseNotebook(JSON.stringify(state),now),state);
  const invalid={...state,work:{bad:{},'shared-awareness':{...state.work['shared-awareness'],sourceUrl:'javascript:alert(1)'}},mistakes:{'nec-original-01':{...state.mistakes['nec-original-01'],correction:'x'.repeat(1201)}}};
  assert.deepEqual(parseNotebook(JSON.stringify(invalid),now),emptyNotebook());
  for(const raw of ['null','[]','{','{"version":2}'])assert.deepEqual(parseNotebook(raw,now),emptyNotebook());
});
test('reference URLs reject executable schemes and credentials',()=>{
  assert.equal(referenceUrl('javascript:alert(1)'),null);assert.equal(referenceUrl('https://user:secret@example.com/'),null);
  assert.equal(referenceUrl('https://example.com/a'),'https://example.com/a');assert.equal(referenceUrl(''),'');
});
test('a newer quiz attempt makes an older reviewed note stale',()=>{
  const note={cause:'concept' as const,correction:'Correct principle',nextAction:'Retry',reviewed:true,updated:100};
  assert.equal(noteCurrent(note,90),true);assert.equal(noteCurrent(note,101),false);
  assert.equal(noteCurrent({...note,correction:''},90),false);
});
