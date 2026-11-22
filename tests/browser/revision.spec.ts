import {test,expect} from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import {readFile} from 'node:fs/promises';
const route='nec/revision-studio/';

test('recall is section-complete and a self-rating leaves the quiz history separate',async({page})=>{
  await page.goto(route);
  const app=page.locator('revision-studio');
  await expect(app.locator('[data-card-count]')).toContainText('60 cards');
  await expect(app.locator('[data-recall-answer]')).toBeHidden();
  await app.getByRole('button',{name:'Reveal answer',exact:true}).click();
  await expect(app.locator('[data-card-answer]')).toContainText('24 W');
  await app.locator('[data-grade="ready"]').click();
  await expect(app.locator('[data-card-count]')).toContainText('59 cards');
  const quiz=await page.evaluate(()=>localStorage.getItem('learning-guide:preparation:v1'));
  expect(quiz).toBeNull();
  await page.reload();await expect(app.locator('[data-card-count]')).toContainText('59 cards');
});
test('written draft and self-checks survive reload; timer expiry does not erase a draft',async({page})=>{
  await page.clock.install();await page.goto(route);
  const app=page.locator('revision-studio');
  await app.locator('[data-draft]').fill('Borrow two bits. /26. Explain the reserved addresses.');
  await app.locator('[data-writing-guide] summary').click();
  await app.locator('[data-checks] input').first().check();
  await page.reload();
  await expect(app.locator('[data-draft]')).toHaveValue('Borrow two bits. /26. Explain the reserved addresses.');
  await app.locator('[data-writing-guide] summary').click();
  await expect(app.locator('[data-checks] input').first()).toBeChecked();
  await app.locator('[data-timer-start]').click();await page.clock.fastForward(10000);
  await app.locator('[data-timer-pause]').click();
  const paused=await app.locator('[data-writing-clock]').textContent();
  await page.clock.fastForward(300000);await expect(app.locator('[data-writing-clock]')).toHaveText(paused!);
  await app.locator('[data-timer-start]').click();await page.clock.fastForward(16*60000);
  await expect(app.locator('[data-notice]')).toContainText('Writing time ended');
  await expect(app.locator('[data-draft]')).toBeEditable();
  await expect(app.locator('[data-draft]')).toHaveValue('Borrow two bits. /26. Explain the reserved addresses.');
});
test('one backup round-trips checklist, answers, bookmarks, recall and drafts',async({page})=>{
  await page.addInitScript(()=>{
    localStorage.setItem('exam-preparation:coverage:v1',JSON.stringify({version:1,topics:{'1.1':{confidence:'ready',written:true,updated:Date.now()}}}));
    localStorage.setItem('learning-guide:preparation:v1',JSON.stringify({version:1,answers:{'nec-original-01':{selected:2,attempts:1,correct:1,updatedAt:Date.now()}},bookmarks:['nec-original-01']}));
  });
  await page.goto(route);const app=page.locator('revision-studio');
  await app.locator('[data-draft]').fill('A portable draft.');
  await app.locator('[data-reveal]').click();await app.locator('[data-grade="building"]').click();
  const downloadPromise=page.waitForEvent('download');await app.locator('[data-backup-export]').click();
  const download=await downloadPromise;const path=(await download.path())!;
  const backup=JSON.parse(await readFile(path,'utf8'));
  expect(backup.coverage.topics['1.1'].confidence).toBe('ready');expect(backup.practice.bookmarks).toEqual(['nec-original-01']);
  expect(Object.keys(backup.revision.recall)).toHaveLength(1);expect(backup.revision.drafts.subnet.text).toBe('A portable draft.');
  await app.locator('[data-draft]').fill('Replacement');
  page.once('dialog',dialog=>dialog.accept());await app.locator('[data-backup-import]').setInputFiles(path);
  await expect(app.locator('[data-notice]')).toContainText('Imported');
  await expect(app.locator('[data-draft]')).toHaveValue('A portable draft.');
});
test('invalid settings and backups preserve valid saved work',async({page})=>{
  await page.goto(route);const app=page.locator('revision-studio');
  await app.locator('[data-draft]').fill('Keep this answer.');
  await app.locator('[data-minutes]').fill('3');await app.locator('[data-minutes]').blur();
  await expect(app.locator('[data-notice]')).toContainText('20–180');
  await app.locator('[data-backup-import]').setInputFiles({name:'wrong.json',mimeType:'application/json',buffer:Buffer.from('{"format":"other"}')});
  await expect(app.locator('[data-notice]')).toContainText('Import failed');
  await expect(app.locator('[data-draft]')).toHaveValue('Keep this answer.');
});
test('studio fits a phone and is accessible in both themes',async({page})=>{
  await page.setViewportSize({width:375,height:812});await page.goto(route);
  for(const theme of ['light','dark']) {
    await page.evaluate(theme=>document.documentElement.setAttribute('data-theme',theme),theme);
    const layout=await page.evaluate(()=>({fits:document.documentElement.scrollWidth<=innerWidth+1,overflow:Array.from(document.querySelectorAll('main *')).filter(el=>el.getBoundingClientRect().right>innerWidth+2).sort((a,b)=>b.getBoundingClientRect().right-a.getBoundingClientRect().right).slice(0,5).map(el=>({tag:el.tagName,classes:el.getAttribute('class'),width:el.getBoundingClientRect().width,text:el.textContent?.slice(0,60)}))}));
    expect(layout.fits,JSON.stringify(layout.overflow)).toBe(true);
    const audit=await new AxeBuilder({page}).include('main').analyze();
    expect(audit.violations.filter(v=>v.impact==='serious'||v.impact==='critical')).toEqual([]);
  }
});
test('blocked storage leaves writing and recall usable',async({page})=>{
  await page.addInitScript(()=>Object.defineProperty(window,'localStorage',{get(){throw new Error('blocked');}}));
  await page.goto(route);const app=page.locator('revision-studio');
  await expect(app.locator('[data-notice]')).toContainText('storage is unavailable');
  await app.locator('[data-draft]').fill('An in-memory answer.');
  await app.locator('[data-reveal]').click();await app.locator('[data-grade="again"]').click();
  await expect(app.locator('[data-draft]')).toHaveValue('An in-memory answer.');
});
test('all recall cards and written prompts remain available without JavaScript',async({browser,baseURL})=>{
  const context=await browser.newContext({javaScriptEnabled:false,baseURL});const page=await context.newPage();await page.goto(route);
  await expect(page.locator('.static-recall')).toHaveCount(60);await expect(page.locator('.static-prompt')).toHaveCount(13);
  await page.locator('[data-static]>details>summary').first().click();await expect(page.locator('.static-recall').first()).toBeVisible();
  await context.close();
});

test('backup restores notebook and mock without replaying answers; legacy backup clears mock',async({page})=>{
  await page.goto('nec/mock-centre/');
  await page.locator('mock-centre [type="submit"]').click();
  await page.locator('mock-centre input[name="mock-answer"]').first().check();
  // Wait for the asynchronous save before navigating away.
  await expect.poll(()=>page.evaluate(()=>Object.keys(JSON.parse(localStorage.getItem('exam-preparation:mock:v1')!).session.answers).length)).toBe(1);
  await page.goto(route);
  const workId=(await import('../../src/data/psc-workstreams')).pscWorkstreams[0].id;
  await page.evaluate(id=>localStorage.setItem('exam-preparation:notebook:v1',JSON.stringify({version:1,work:{[id]:{status:'learning',notes:'Keep this reference note',sourceUrl:'https://psc.gov.np/',checkedDate:'2026-11-17',updated:Date.now()}},mistakes:{}})),workId);
  const app=page.locator('revision-studio');
  const wait=page.waitForEvent('download');await app.locator('[data-backup-export]').click();
  const path=(await (await wait).path())!;const backup=JSON.parse(await readFile(path,'utf8'));
  expect(backup.notebook.work[workId].notes).toBe('Keep this reference note');expect(Object.keys(backup.mock.session.answers)).toHaveLength(1);
  await page.evaluate(()=>{localStorage.removeItem('exam-preparation:notebook:v1');localStorage.removeItem('exam-preparation:mock:v1');});
  page.once('dialog',d=>d.accept());await app.locator('[data-backup-import]').setInputFiles(path);
  await expect(app.locator('[data-notice]')).toContainText('Imported');
  expect(await page.evaluate(()=>JSON.parse(localStorage.getItem('exam-preparation:notebook:v1')!).work)).toEqual(backup.notebook.work);
  expect(await page.evaluate(()=>JSON.parse(localStorage.getItem('learning-guide:preparation:v1')!).answers)).toEqual({});
  delete backup.mock;delete backup.notebook;
  page.once('dialog',d=>d.accept());await app.locator('[data-backup-import]').setInputFiles({name:'legacy.json',mimeType:'application/json',buffer:Buffer.from(JSON.stringify(backup))});
  await expect(app.locator('[data-notice]')).toContainText('Imported');
  expect(await page.evaluate(()=>JSON.parse(localStorage.getItem('exam-preparation:mock:v1')!).session)).toBeNull();
});
