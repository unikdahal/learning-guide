import { test } from 'node:test';
import assert from 'node:assert/strict';
import { existsSync } from 'node:fs';
import { practiceQuestions, practiceTopics, practiceSections, validateQuestions } from '../../src/data/preparation-practice.ts';
import { emptyState, parseState, sanitizeState, recordAnswer, selectQuestions, topicPerformance } from '../../src/components/preparation/state.ts';
const first = practiceQuestions[0];
const wrong = (first.answer + 1) % 4;
test('120 original questions cover all 60 sections exactly twice and all study destinations exist', () => {
  assert.equal(practiceQuestions.length, 120);
  const expectedSections = Array.from({length:10}, (_,c) => Array.from({length:6}, (_,s) => `${c+1}.${s+1}`)).flat();
  assert.deepEqual(practiceSections, expectedSections);
  assert.equal(new Set(practiceQuestions.map(q => q.section)).size, expectedSections.length);
  for (const section of expectedSections) assert.equal(practiceQuestions.filter(q => q.section === section).length, 2, section);
  for (let chapter = 1; chapter <= 10; chapter++) assert.equal(practiceQuestions.filter(q => q.section.startsWith(`${chapter}.`)).length, 12);
  assert.equal(new Set(practiceQuestions.map(q => q.id)).size, practiceQuestions.length);
  assert.equal(new Set(practiceQuestions.map(q => q.prompt.trim().toLowerCase())).size, practiceQuestions.length);
  assert.deepEqual(practiceQuestions.map(q => q.id), practiceQuestions.map((_,i) => `nec-original-${String(i+1).padStart(2,'0')}`));
  assert.equal(first.prompt, 'A 12 V source is connected across a 6 Ω resistor. What power does the resistor dissipate?');
  assert.equal(first.answer, 2);
  assert.deepEqual(validateQuestions(practiceQuestions), []);
  assert.deepEqual([...new Set(practiceQuestions.map(q => Number(q.section.split('.')[0])))].sort((a,b) => a-b), [1,2,3,4,5,6,7,8,9,10]);
  for (const [topic,,, slug] of practiceTopics) {
    assert.ok(practiceQuestions.some(q => q.topic === topic), topic);
    assert.ok(existsSync(new URL(`../../src/content/docs/nec/chapters/${slug}.mdx`, import.meta.url)), slug);
  }
});
test('validation rejects duplicate IDs, wrong chapter mappings, bad options, answer indices and absent explanations', () => {
  for (const patch of [{section:'9.1'}, {options:['same','same','x','y']}, {answer:4}, {answer:1.5}, {explanation:''}]) {
    assert.ok(validateQuestions([{...first, ...patch} as typeof first]).length > 0);
  }
  assert.ok(validateQuestions([first, first]).some(error => error.includes('Duplicate')));
});
test('corrupt, old-version, and invalid container storage is safe', () => {
  for (const raw of ['', '{bad', 'null', '[]', 'false', '{"version":2}', '{"version":1,"answers":[],"bookmarks":{}}']) {
    assert.deepEqual(parseState(raw), emptyState());
  }
  assert.deepEqual(sanitizeState({version:1, answers:{'__proto__':{}}, bookmarks:['unknown',null,{},'__proto__']}), emptyState());
});
test('validation rejects impossible counts and last-answer records without discarding valid bookmarks', () => {
  const good = { selected:wrong, attempts:2, correct:1, updatedAt:123 };
  const invalid = [{selected:-1},{selected:4},{selected:'0'},{attempts:0},{attempts:1.5},{attempts:1_000_001},{correct:-1},{correct:3},{correct:2},{updatedAt:Infinity},{updatedAt:-1},{updatedAt:'today'}];
  for (const patch of invalid) {
    const state = sanitizeState({version:1, answers:{[first.id]:{...good,...patch}}, bookmarks:[first.id,first.id,'unknown']});
    assert.deepEqual(state.answers, {}, JSON.stringify(patch));
    assert.deepEqual(state.bookmarks,[first.id]);
  }
  assert.deepEqual(sanitizeState({version:1, answers:{[first.id]:{...good,selected:first.answer,correct:0}}}).answers, {});
});
test('persisted records round-trip and mutation does not alter an earlier state', () => {
  const blank = emptyState();
  const once = recordAnswer(blank, first, wrong, 100);
  const twice = recordAnswer(once, first, first.answer, 200);
  assert.deepEqual(blank,emptyState());
  assert.equal(once.answers[first.id].attempts,1);
  assert.deepEqual(twice.answers[first.id],{selected:first.answer,attempts:2,correct:1,updatedAt:200});
  assert.deepEqual(parseState(JSON.stringify(twice)),twice);
  assert.equal(recordAnswer(twice,first,NaN),twice);
});
test('mistake and unanswered filters use latest answers; bookmarks intersect the subject', () => {
  let state = recordAnswer(emptyState(),first,wrong,100);
  state.bookmarks = [first.id,practiceQuestions[4].id];
  assert.deepEqual(selectQuestions(state,'all','mistakes').map(q=>q.id),[first.id]);
  assert.equal(selectQuestions(state,'all','unanswered').length,practiceQuestions.length - 1);
  assert.deepEqual(selectQuestions(state,'electrical','bookmarks').map(q=>q.id),[first.id]);
  const snapshot = selectQuestions(state,'all','mistakes');
  state = recordAnswer(state,first,first.answer,200);
  assert.equal(selectQuestions(state,'all','mistakes').length,0);
  assert.equal(snapshot.length,1,'An existing round keeps its question snapshot');
  assert.equal(state.bookmarks.length,2);
  assert.equal(selectQuestions(state,'unknown','all').length,0);
});
test('topic signals have no invented data and retain incorrect retries in the denominator', () => {
  assert.ok(topicPerformance(emptyState()).every(row => row.attempts === 0 && row.correct === 0 && row.seen === 0));
  let state = recordAnswer(emptyState(),first,wrong,100);
  state = recordAnswer(state,first,first.answer,200);
  const row = topicPerformance(state).find(row => row.topic === first.topic)!;
  assert.equal(row.attempts,2); assert.equal(row.correct,1); assert.equal(row.seen,1); assert.equal(row.total,practiceQuestions.filter(q => q.topic === first.topic).length);
  assert.equal(selectQuestions(state,'all','all').length,practiceQuestions.length);
});
test('storage count cap keeps the record internally consistent', () => {
  const state = emptyState();
  state.answers[first.id] = {selected:wrong,attempts:1_000_000,correct:1,updatedAt:10};
  assert.equal(recordAnswer(state,first,first.answer),state);
  assert.deepEqual(sanitizeState(state),state);
});

test('section and literal case-insensitive question search intersect review and subject filters', () => {
  let state = recordAnswer(emptyState(),first,wrong,100);
  state.bookmarks = [first.id];
  const section = selectQuestions(state,'all','all','10.6');
  assert.equal(section.length,2);
  assert.ok(section.every(q => q.section === '10.6'));
  assert.deepEqual(selectQuestions(state,'all','all','all','  NEC 10.6  '), section);
  assert.deepEqual(selectQuestions(state,'all','all','all','12 V source'), [first]);
  assert.deepEqual(selectQuestions(state,'electrical','mistakes','1.1','12 v SOURCE'), [first]);
  assert.deepEqual(selectQuestions(state,'electrical','bookmarks','1.1','12 v'), [first]);
  assert.deepEqual(selectQuestions(state,'electrical','all','10.6'), []);
  assert.deepEqual(selectQuestions(state,'all','all','10.7'), []);
  for (const literal of ['<img src=x onerror=alert(1)>', '.*', '[']) {
    assert.deepEqual(selectQuestions(state,'all','all','all',literal), practiceQuestions.filter(q => q.prompt.toLowerCase().includes(literal.toLowerCase())));
  }
  state = recordAnswer(state,first,first.answer,200);
  assert.deepEqual(selectQuestions(state,'all','mistakes','1.1','12 v'), []);
  assert.deepEqual(selectQuestions(state,'all','bookmarks','1.1','12 v'), [first]);
});
test('existing bookmarks and answer IDs survive the expanded bank', () => {
  const old = {version:1, answers:{[first.id]:{selected:wrong,attempts:1,correct:0,updatedAt:100}},bookmarks:[first.id,'nec-original-40']};
  assert.deepEqual(sanitizeState(old),old);
  const last = practiceQuestions.at(-1)!;
  assert.deepEqual(parseState(JSON.stringify(recordAnswer(emptyState(),last,last.answer,100))).answers[last.id], {selected:last.answer,attempts:1,correct:1,updatedAt:100});
});
