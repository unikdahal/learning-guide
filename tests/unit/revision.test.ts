import {test} from 'node:test';
import assert from 'node:assert/strict';
import {DAY,emptyStudy} from '../../src/lib/study-state.ts';
import {emptyState} from '../../src/components/preparation/state.ts';
import {emptyRevision,parseRevision,recallDue,sessionBlocks,validExamDate,daysUntil,parsePreparationBackup,writePreparationStores} from '../../src/lib/revision-state.ts';
import {writtenPrompts} from '../../src/data/written-practice.ts';
import {emptyNotebook} from '../../src/lib/notebook-state.ts';
import {createMock,answerMock,finaliseMock,persistMockFinalisation,MOCK_KEY,parseMock} from '../../src/lib/mock-state.ts';
import {parseState,STORAGE_KEY} from '../../src/components/preparation/state.ts';
import {mockNeedsRecovery} from '../../src/lib/revision-state.ts';

test('calendar dates reject rollover dates and compare calendar days rather than elapsed hours',()=>{
  assert.equal(validExamDate('2027-02-29'),false);
  assert.equal(validExamDate('2028-02-29'),true);
  assert.equal(validExamDate('2027-13-01'),false);
  assert.equal(daysUntil('2027-01-05',new Date(2027,0,4,23,59)),1);
  assert.equal(daysUntil('',new Date()),null);
  assert.equal(daysUntil('2027-01-03',new Date(2027,0,4)), -1);
});
test('revision storage sanitises settings, dates, recall grades and note sizes',()=>{
  const now=100*DAY;const state=emptyRevision();
  state.recall['1.1']={grade:'ready',reviewed:now,reviews:2};
  state.drafts[writtenPrompts[0]!.id]={text:'My answer',checks:writtenPrompts[0]!.checks.map(()=>false),updated:now};
  assert.deepEqual(parseRevision(JSON.stringify(state),now),state);
  const bad={...state,settings:{target:'unknown',minutes:0,examDate:'2027-02-30'},recall:{'1.1':{grade:'expert',reviewed:now,reviews:1}},drafts:{subnet:{text:'x'.repeat(8001),checks:[],updated:now}}};
  assert.deepEqual(parseRevision(JSON.stringify(bad),now),emptyRevision());
  for(const raw of ['{','null','[]','{"version":2}'])assert.deepEqual(parseRevision(raw,now),emptyRevision());
});
test('recall deadlines have explicit 1/3/7-day boundaries and unseen cards are due',()=>{
  assert.equal(recallDue(undefined),true);
  assert.equal(recallDue({grade:'building',reviewed:10*DAY,reviews:1},13*DAY-1),false);
  assert.equal(recallDue({grade:'building',reviewed:10*DAY,reviews:1},13*DAY),true);
});
test('every session budget sums to its exact requested duration',()=>{
  for(let minutes=20;minutes<=180;minutes++)for(const target of ['nec','ce','co','combined'] as const) {
    const blocks=sessionBlocks(minutes,target);
    assert.equal(blocks.reduce((sum,b)=>sum+b.minutes,0),minutes);
    assert.ok(blocks.every(b=>b.minutes>0));
  }
  assert.throws(()=>sessionBlocks(19,'nec'));
});
test('backup round-trip includes all namespaces and rejects a partial envelope',()=>{
  const backup={format:'nec-loksewa-preparation',version:1,exportedAt:new Date().toISOString(),coverage:emptyStudy(),practice:emptyState(),revision:emptyRevision()};
  assert.deepEqual(parsePreparationBackup(JSON.stringify(backup)),backup);
  assert.throws(()=>parsePreparationBackup(JSON.stringify({...backup,practice:null})));
  assert.throws(()=>parsePreparationBackup(JSON.stringify({...backup,format:'other'})));
});
test('new backups retain notebook and a running mock; legacy envelopes stay compatible',()=>{
  const now=Date.now(),backup={format:'nec-loksewa-preparation' as const,version:1 as const,exportedAt:new Date(now).toISOString(),coverage:emptyStudy(),practice:emptyState(),revision:emptyRevision(),notebook:emptyNotebook(),mock:createMock(60,'portable',10,0,now)};
  backup.notebook.mistakes['nec-original-01']={cause:'concept',correction:'Power is V squared / R.',nextAction:'Retry',reviewed:true,updated:now};
  assert.deepEqual(parsePreparationBackup(JSON.stringify(backup),now),backup);
  assert.throws(()=>parsePreparationBackup(JSON.stringify({...backup,notebook:{version:2}})));
  assert.throws(()=>parsePreparationBackup(JSON.stringify({...backup,mock:{version:2}})));
});
test('backup preserves a pending mock journal after practice was written, without replaying it',()=>{
  const now=Date.now(),values=new Map<string,string>(),storage={getItem:(key:string)=>values.get(key)??null,setItem:(key:string,value:string)=>{if(key===MOCK_KEY&&JSON.parse(value).session?.recorded)throw new Error('commit interrupted');values.set(key,value);}};
  let mock=createMock(20,'backup-journal',1,0,now);
  for(const id of mock.session!.ids.slice(0,3))mock=answerMock(mock,id,0,now);
  mock=finaliseMock(mock,'manual',now+1);
  assert.throws(()=>persistMockFinalisation(storage,mock));
  const pending=parseMock(values.get(MOCK_KEY)!,now+2),practice=parseState(values.get(STORAGE_KEY)!);
  const envelope={format:'nec-loksewa-preparation',version:1,exportedAt:new Date(now).toISOString(),coverage:emptyStudy(),practice,revision:emptyRevision(),mock:pending};
  const restored=parsePreparationBackup(JSON.stringify(envelope),now+2);
  assert.ok(restored.mock?.pending);assert.deepEqual(restored.practice,practice);assert.equal(mockNeedsRecovery(restored),false);
  const conflicted={...restored,practice:emptyState()};
  // The journal's before snapshot is empty too, so use a genuinely unrelated bookmark change.
  conflicted.practice.bookmarks=['nec-original-01'];
  assert.equal(mockNeedsRecovery(conflicted),true);
  assert.deepEqual(parsePreparationBackup(JSON.stringify(conflicted),now+2).practice,conflicted.practice);
});
test('written prompts have unique IDs, actual destination paths and checkable outlines',()=>{
  assert.equal(new Set(writtenPrompts.map(p=>p.id)).size,writtenPrompts.length);
  for(const p of writtenPrompts){assert.ok(p.checks.length>=6);assert.ok(p.outline.length>100);assert.match(p.lesson,/^(nec|lok-sewa)\/[a-z0-9/-]+\/$/);}
});
test('a quota failure during backup import restores earlier stores and absent keys',()=>{
  const data=new Map([['coverage','original']]);let writes=0;
  const storage={getItem:(key:string)=>data.get(key)??null,setItem:(key:string,value:string)=>{if(++writes===2)throw new Error('quota');data.set(key,value);},removeItem:(key:string)=>{data.delete(key);}};
  assert.throws(()=>writePreparationStores(storage,[['coverage','replacement'],['practice','new'],['revision','new']]),/original preparation stores restored/);
  assert.deepEqual([...data.entries()],[['coverage','original']]);
});
