import { test } from 'node:test';
import assert from 'node:assert/strict';
import { DAY, parseStudy, reviewDue, priority } from '../../src/lib/study-state.ts';
import { studyChapters, studyTopics } from '../../src/data/study-map.ts';

test('NEC checklist contains exactly six stable sections in each of ten chapters', () => {
  assert.equal(studyChapters.length, 10);
  assert.equal(new Set(studyTopics.map(t => t.id)).size, 60);
  for (const chapter of studyChapters) {
    assert.deepEqual(chapter.topics.map(t => t.id), Array.from({ length: 6 }, (_, i) => `${chapter.id}.${i + 1}`));
    for (const topic of chapter.topics) assert.match(topic.lesson, /^nec\/[a-z0-9/-]+\/$/);
  }
});
test('study import rejects unknown versions, unsafe shapes, invalid confidence and future dates', () => {
  const now = 100 * DAY;
  for (const raw of ['{', 'null', '[]', '{"version":2,"topics":{}}', '{"version":1,"topics":null}']) {
    assert.deepEqual(parseStudy(raw, ['1.1'], now).topics, {});
  }
  const row = { confidence: 'ready', written: true, updated: now };
  const raw = JSON.stringify({version: 1, topics: {'1.1':row, 'unknown':row, '1.2':{...row,confidence:'expert'}, '1.3':{...row, updated:now+2*DAY}}});
  assert.deepEqual(parseStudy(raw, ['1.1','1.2','1.3'], now).topics, {'1.1':row});
});
test('review boundaries and priorities bring weak and overdue topics back before new work', () => {
  const p = { confidence: 'ready' as const, written: false, updated: 10 * DAY };
  assert.equal(reviewDue(p, 17 * DAY - 1), false);
  assert.equal(reviewDue(p, 17 * DAY), true);
  assert.equal(reviewDue(undefined, 20 * DAY), false);
  assert.ok(priority({...p, confidence:'review'},false,12*DAY) > priority(undefined,true,12*DAY));
  assert.ok(priority(undefined,true) > priority(undefined,false));
});
