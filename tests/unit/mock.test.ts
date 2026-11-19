import { test } from 'node:test';
import assert from 'node:assert/strict';
import { practiceQuestions } from '../../src/data/preparation-practice.ts';
import { STORAGE_KEY, emptyState, recordAnswer, parseState } from '../../src/components/preparation/state.ts';
import { MOCK_KEY, createMock, selectMockQuestions, parseMock, exportMock, emptyMock, answerMock, finaliseMock, persistMockFinalisation, remainingMock, mockReport, retryWrong } from '../../src/lib/mock-state.ts';

const now = 1_000_000;
function memory() { const values = new Map<string, string>(); return { values, getItem: (k: string) => values.get(k) ?? null, setItem: (k: string, v: string) => { values.set(k, v); } }; }
test('seeded selection is repeatable, duplicate-free, balanced and section-complete across many seeds', () => {
  for (let seed = 0; seed < 100; seed++) for (const count of [20, 60, 100] as const) {
    const ids = selectMockQuestions(count, String(seed)), qs = ids.map(id => practiceQuestions.find(q => q.id === id)!);
    assert.deepEqual(selectMockQuestions(count, String(seed)), ids);
    assert.equal(ids.length, count); assert.equal(new Set(ids).size, count);
    for (let chapter = 1; chapter <= 10; chapter++) assert.equal(qs.filter(q => q.section.startsWith(`${chapter}.`)).length, count / 10);
    assert.equal(new Set(qs.map(q => q.section)).size, Math.min(count, 60));
  }
  assert.notDeepEqual(selectMockQuestions(60, 'a'), selectMockQuestions(60, 'b'));
  assert.throws(() => createMock(20, 'a', 0, 0));
});
test('parser rejects tampered identity/order/times/status and sanitises unknown answers, flags and cursor', () => {
  const state = createMock(60, 'test', 10, 0.2, now);
  assert.deepEqual(parseMock(JSON.stringify(state), now), state);
  for (const patch of [{ ids: ['unknown'] }, { ids: [...state.session!.ids].reverse() }, { deadline: now - 1 }, { started: now + 1 }, { finished: now + 1 }, { recorded: true }, { penalty: '0.2' }, { reason: 'expiry' }]) {
    assert.deepEqual(parseMock(JSON.stringify({ ...state, session: { ...state.session, ...patch } }), now), emptyMock());
  }
  const id = state.session!.ids[0]!;
  const dirty = { ...state, session: { ...state.session, answers: { [id]: 2, unknown: 3 }, flagged: [id, id, 'unknown'], cursor: 999 } };
  const clean = parseMock(JSON.stringify(dirty), now);
  assert.deepEqual(clean.session!.answers, { [id]: 2 }); assert.deepEqual(clean.session!.flagged, [id]); assert.equal(clean.session!.cursor, 59);
  for (const raw of ['{', 'null', '[]', '{"version":9}', 'x'.repeat(300001)]) assert.deepEqual(parseMock(raw, now), emptyMock());
  assert.deepEqual(exportMock(clean, now), clean);
});
test('answers may change until absolute deadline; expiry finalises once and scores blanks separately', () => {
  let state = createMock(20, 'timer', 1, 0.2, now);
  const [a, b] = state.session!.ids, qa = practiceQuestions.find(q => q.id === a)!;
  state = answerMock(state, a!, (qa.answer + 1) % 4, now);
  state = answerMock(state, a!, qa.answer, now + 1);
  state = answerMock(state, b!, 1, now + 2); state = answerMock(state, b!, null, now + 3);
  assert.equal(mockReport(state.session!).correct, 1); assert.equal(mockReport(state.session!).blank, 19);
  assert.equal(remainingMock(state.session!, now + 59000), 1000);
  assert.strictEqual(finaliseMock(state, 'expiry', now + 59999), state);
  assert.strictEqual(answerMock(state, b!, 1, now + 60000), state);
  state = finaliseMock(state, 'manual', now + 60001);
  assert.equal(state.session!.reason, 'expiry'); assert.equal(state.session!.finished, now + 60000);
  assert.strictEqual(finaliseMock(state, 'expiry', now + 70000), state);
  assert.strictEqual(answerMock(state, b!, 1, now + 70000), state);
});
test('finalisation preserves history/bookmarks and is idempotent even with a stale caller', () => {
  const storage = memory(); let state = createMock(20, 'history', 2, 0, now);
  const q = practiceQuestions.find(q => q.id === state.session!.ids[0])!;
  let practice = recordAnswer(emptyState(), q, q.answer, now - 1); practice.bookmarks = [q.id];
  storage.setItem(STORAGE_KEY, JSON.stringify(practice));
  state = finaliseMock(answerMock(state, q.id, (q.answer + 1) % 4, now), 'manual', now + 1);
  const done = persistMockFinalisation(storage, state);
  assert.equal(done.session!.recorded, true);
  persistMockFinalisation(storage, state); persistMockFinalisation(storage, parseMock(storage.getItem(MOCK_KEY), now + 2));
  const result = parseState(storage.getItem(STORAGE_KEY)); assert.equal(result.answers[q.id]!.attempts, 2); assert.equal(result.answers[q.id]!.correct, 1); assert.deepEqual(result.bookmarks, [q.id]);
});
test('recover each interrupted write without lost or double-counted attempts', () => {
  for (const failure of [1, 2, 3]) {
    const storage = memory(); let state = createMock(20, 'crash', 1, 0, now);
    const id = state.session!.ids[0]!; state = finaliseMock(answerMock(state, id, 0, now), 'manual', now + 1);
    let writes = 0;
    const failing = { getItem: storage.getItem, setItem: (k: string, v: string) => { if (++writes === failure) throw new Error('quota'); storage.setItem(k, v); } };
    assert.throws(() => persistMockFinalisation(failing, state), /quota/);
    const recovered = parseMock(storage.getItem(MOCK_KEY), now + 2);
    const done = persistMockFinalisation(storage, recovered.session ? recovered : state);
    persistMockFinalisation(storage, done);
    assert.equal(parseState(storage.getItem(STORAGE_KEY)).answers[id]!.attempts, 1);
  }
});
test('a shuffled multi-answer journal survives reload after history was already written',()=>{
  const storage=memory();let state=createMock(20,'multi-answer-recovery',1,0,now);
  const ids=state.session!.ids.slice(0,5);
  for(const id of ids)state=answerMock(state,id,0,now);
  state=finaliseMock(state,'manual',now+1);
  const failAfterHistory={getItem:storage.getItem,setItem:(key:string,value:string)=>{
    if(key===MOCK_KEY&&JSON.parse(value).session?.recorded)throw new Error('interrupted commit');
    storage.setItem(key,value);
  }};
  assert.throws(()=>persistMockFinalisation(failAfterHistory,state),/interrupted commit/);
  const recovered=parseMock(storage.getItem(MOCK_KEY),now+2);assert.ok(recovered.pending);
  persistMockFinalisation(storage,recovered);
  for(const id of ids)assert.equal(parseState(storage.getItem(STORAGE_KEY)).answers[id]!.attempts,1);
});
test('conflicting history is preserved and imported journals cannot inject arbitrary changes', () => {
  const storage = memory(); let state = createMock(20, 'conflict', 1, 0, now);
  state = finaliseMock(answerMock(state, state.session!.ids[0]!, 0, now), 'manual', now + 1);
  const fail = { getItem: storage.getItem, setItem: (k: string, v: string) => { if (k === STORAGE_KEY) throw new Error('quota'); storage.setItem(k, v); } };
  assert.throws(() => persistMockFinalisation(fail, state));
  const pending = parseMock(storage.getItem(MOCK_KEY), now + 2); assert.ok(pending.pending);
  const tampered = structuredClone(pending); tampered.pending!.after.bookmarks = [practiceQuestions[0]!.id];
  assert.deepEqual(parseMock(JSON.stringify(tampered), now + 2), emptyMock());
  const external = recordAnswer(emptyState(), practiceQuestions[0]!, 1, now + 2);
  storage.setItem(STORAGE_KEY, JSON.stringify(external));
  assert.throws(() => persistMockFinalisation(storage, pending), /history changed/);
  assert.deepEqual(parseState(storage.getItem(STORAGE_KEY)), external);
});
test('retry contains only wrong answers and a new finalisation is a new attempt', () => {
  let state = createMock(20, 'retry', 1, 0.2, now);
  const q = practiceQuestions.find(q => q.id === state.session!.ids[0])!;
  state = finaliseMock(answerMock(state, q.id, (q.answer + 1) % 4, now), 'manual', now + 1);
  assert.equal(mockReport(state.session!).score, -0.2);
  const retry = retryWrong(state.session!, 'again', 1, now + 2);
  assert.deepEqual(retry.session!.ids, [q.id]); assert.deepEqual(retry.session!.answers, {});
  assert.equal(retry.session!.kind, 'retry'); assert.deepEqual(parseMock(JSON.stringify(retry), now + 2), retry);
  const storage = memory(); persistMockFinalisation(storage, state);
  storage.setItem(MOCK_KEY, JSON.stringify(retry));
  const finished = finaliseMock(answerMock(retry, q.id, q.answer, now + 3), 'manual', now + 4);
  persistMockFinalisation(storage, finished);
  assert.equal(parseState(storage.getItem(STORAGE_KEY)).answers[q.id]!.attempts, 2);
});

test('a stale finished caller cannot replace a newer durable round or replay attempts', () => {
  const storage = memory();
  let old = createMock(20, 'old', 10, 0, now, 'old');
  old = answerMock(old, old.session!.ids[0]!, 0, now + 1);
  old = finaliseMock(old, 'manual', now + 2);
  persistMockFinalisation(storage, old);
  const history = storage.getItem(STORAGE_KEY);
  const newer = createMock(20, 'new', 10, 0, now + 3, 'new');
  storage.setItem(MOCK_KEY, JSON.stringify(newer));
  assert.throws(() => persistMockFinalisation(storage, old), /replaced/);
  assert.equal(storage.getItem(STORAGE_KEY), history);
  assert.equal(parseMock(storage.getItem(MOCK_KEY), now + 3).session!.id, 'new');
});
