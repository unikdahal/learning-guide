import {test} from 'node:test';
import assert from 'node:assert/strict';
import {sanitizeState,mergeProgress,INITIAL_GENERATION} from '../../src/scripts/progress-state.ts';
const path='/learning-guide/orientation/what-a-query-engine-actually-does/';
const entry={t:'Title',m:'00',mt:'Orientation',p:0.4,ts:10,s:'section',st:'Section'};
test('corrupt containers and entries cannot escape validation',()=>{
 for (const articles of [null,[],1,'bad']) assert.deepEqual(sanitizeState({v:1,articles},INITIAL_GENERATION).articles,{});
 for (const patch of [{p:'bad'},{p:NaN},{p:Infinity},{p:-1},{p:1.1},{ts:-1},{done:'yes'},{m:'bad'}]) {
  assert.deepEqual(sanitizeState({v:1,articles:{[path]:{...entry,...patch}}},INITIAL_GENERATION).articles,{});
 }
});
test('migration preserves valid records and rejects unknown paths',()=>{
 const state=sanitizeState({v:1,last:path,articles:{[path]:entry,'https://other.invalid/x/':entry}},INITIAL_GENERATION,new Set([path]));
 assert.equal(state.v,2);assert.deepEqual(state.articles[path],entry);assert.equal(state.last,path);assert.equal(Object.keys(state.articles).length,1);
 assert.deepEqual(sanitizeState({v:1,articles:{[path]:entry}},'reset').articles,{});
 assert.deepEqual(sanitizeState({v:2,generation:'old',articles:{[path]:entry}},'new').articles,{});
});
test('both write orders retain completion and the latest section',()=>{
 const done={...entry,p:1,done:true,ts:20,s:'finished'};
 const recent={...entry,p:0.3,ts:30,s:'rereading'};
 for (const [a,b] of [[done,recent],[recent,done]]) {
  const merged=mergeProgress(a,b);assert.equal(merged.p,1);assert.equal(merged.done,true);assert.equal(merged.s,'rereading');
 }
});
