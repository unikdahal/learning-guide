import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {normalizeSyllabus,loadSyllabus} from '../../scripts/syllabus.mjs';
import {articleStatus,setPublication,frontmatter} from '../../scripts/frontmatter.mjs';
const fixture=()=>[{file:'fixture.yaml',data:{part:{id:1,slug:'foundations',title:'Foundations',summary:'Start here',color:'#123456'},modules:[{id:'00',slug:'orientation',title:'Orientation',goal:'Learn',articles:[{t:'First lesson',l:'B',d:'A scope'}]}]}}];
test('reject malformed and unsafe curriculum before generation',()=>{
 for (const mutate of [
  d=>d.push(structuredClone(d[0])),
  d=>d[0].data.modules[0].slug='../outside',
  d=>d[0].data.modules[0].id=10,
  d=>d[0].data.modules[0].articles[0].slug='index',
  d=>d[0].data.modules[0].articles[0].l='X',
  d=>d[0].data.modules[0].prereqs=['99'],
  d=>d[0].data.modules[0].prereqs=['00'],
  d=>delete d[0].data.modules[0].articles[0].d,
 ]) {const d=fixture();mutate(d);assert.throws(()=>normalizeSyllabus(d));}
});
test('prerequisite cycles report a chain',()=>{
 const d=fixture(),m=d[0].data.modules[0];m.prereqs=['01'];d[0].data.modules.push({...structuredClone(m),id:'01',slug:'other',prereqs:['00']});
 assert.throws(()=>normalizeSyllabus(d),/00 -> 01 -> 00/);
});
test('beginner route includes every required module',()=>{
 const {modules}=loadSyllabus();const routes=JSON.parse(readFileSync(new URL('../../src/data/routes.json',import.meta.url),'utf8'));
 const route=routes.find(r=>r.id==='new');const ids=new Set(route.steps);
 for(const id of ids)for(const p of modules.find(m=>m.id===id).prereqs)assert.ok(ids.has(p),`${id} requires ${p}`);
 for(const r of routes)for(const id of r.steps)assert.ok(modules.some(m=>m.id===id));
});
test('publication parses only frontmatter and never rewrites authored bodies',()=>{
 const body='A body with status: published\n\n```sql\nSELECT 1;\n```\n';
 const text='---\ntitle: Test\nstatus: planned\nsidebar:\n  order: 1\n  attrs:\n    data-status: planned\n---\n'+body;
 const output=setPublication(text,'published');assert.equal(articleStatus(output,'test'),'published');assert.equal(frontmatter(output).body,body);
 assert.equal(frontmatter(output).data.sidebar.attrs?.['data-status'],undefined);
 assert.throws(()=>articleStatus('---\ntitle: Test\n---\nstatus: published','bad'));
});
