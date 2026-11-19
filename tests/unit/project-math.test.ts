import {test} from 'node:test';
import assert from 'node:assert/strict';
import {presentValue, projectSchedule} from '../../src/lib/project-math.ts';
test('discounting preserves time-zero outflow and handles zero interest', () => {
  assert.deepEqual(presentValue(100,40,3,0),[-100,40,40,40]);
  assert.ok(Math.abs(presentValue(100,40,3,10).reduce((a,b)=>a+b,0) + .52592036063112) < .00001);
  for (const value of [NaN,Infinity,-1,1.5,31]) assert.throws(()=>presentValue(100,40,value,10));
});
test('CPM backward pass handles merge, critical-path switch and multiple critical paths', () => {
  const baseline=projectSchedule(3);
  assert.equal(baseline.duration,8);
  assert.deepEqual(baseline.rows.filter(r=>r.float===0).map(r=>r.id),['A','B','C','D','E']);
  const slower=projectSchedule(5);
  assert.equal(slower.duration,10);
  assert.deepEqual(slower.rows.map(r=>r.float),[0,2,0,1,0]);
  const faster=projectSchedule(1);
  assert.equal(faster.duration,8);
  assert.deepEqual(faster.rows.map(r=>r.float),[0,0,2,0,2]);
  assert.throws(()=>projectSchedule(0));
});
