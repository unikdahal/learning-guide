import {test,expect} from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import {pscWorkstreams} from '../../src/data/psc-workstreams';
const route='lok-sewa/preparation/notebook/';
test('PSC work statuses and personal references persist without becoming an exam score',async({page})=>{
  await page.goto(route);const app=page.locator('preparation-notebook');
  await expect(app.locator('[data-work]')).toHaveCount(pscWorkstreams.length);
  await app.locator('[data-work-target]').selectOption('co');await app.locator('[data-work-kind]').selectOption('practical');
  await expect(app.locator('[data-work]:visible')).toHaveCount(4);
  const row=app.locator('[data-work="co-practical-dbms"]');await row.locator('summary').click();
  await row.locator('[data-work-state]').selectOption('checked');await row.locator('[data-work-notes]').fill('Verified foreign keys and rollback in my practice database.');
  await row.locator('[data-work-url]').fill('https://example.com/db-notes');await row.locator('[data-work-url]').blur();
  await row.locator('[data-work-date]').fill('2020-01-01');await row.locator('[data-work-date]').blur();
  await expect(app.locator('[data-work-summary]')).toContainText('1 self-checked');
  await page.reload();await row.locator('summary').click();
  await expect(row.locator('[data-work-state]')).toHaveValue('checked');
  await expect(row.locator('[data-work-notes]')).toHaveValue('Verified foreign keys and rollback in my practice database.');
  await expect(row.locator('[data-work-reference]')).toHaveAttribute('href','https://example.com/db-notes');
  await row.locator('[data-work-url]').fill('javascript:alert(1)');await row.locator('[data-work-url]').blur();
  await expect(app.locator('[data-notebook-status]')).toContainText('not saved');
  await expect(row.locator('[data-work-reference]')).toHaveAttribute('href','https://example.com/db-notes');
});
test('mistake notes preserve actual wrong answers and become stale after another attempt',async({page,context})=>{
  await page.addInitScript(()=>localStorage.setItem('learning-guide:preparation:v1',JSON.stringify({version:1,answers:{'nec-original-01':{selected:0,attempts:1,correct:0,updatedAt:1}},bookmarks:[]})));
  await page.goto(route);const app=page.locator('preparation-notebook');
  await expect(app.locator('[data-mistake-summary]')).toContainText('1 matching');
  await app.locator('[data-note-cause]').selectOption('calculation');
  await app.locator('[data-note-correction]').fill('Power is V squared divided by R: 144/6 = 24 W.');
  await app.locator('[data-note-next]').fill('Redo with a different resistance.');await app.locator('[data-note-reviewed]').check();
  await expect(app.locator('[data-note-status]')).toContainText('marked checked');
  const attempts=await page.evaluate(()=>JSON.parse(localStorage.getItem('learning-guide:preparation:v1')!).answers['nec-original-01']);
  expect(attempts.correct).toBe(0);expect(attempts.attempts).toBe(1);
  const other=await context.newPage();await other.goto('nec/practice/');
  await other.locator('nec-practice').getByRole('radio').nth(1).check();await other.getByRole('button',{name:'Check answer',exact:true}).click();
  await expect(app.locator('[data-note-status]')).toContainText('newer quiz attempt');
  await expect(app.locator('[data-note-reviewed]')).not.toBeChecked();
  await other.close();
});
test('uncertainty notes are usable with no quiz history and blocked storage',async({page})=>{
  await page.addInitScript(()=>Object.defineProperty(window,'localStorage',{get(){throw new Error('blocked');}}));
  await page.goto(route);const app=page.locator('preparation-notebook');
  await expect(app.locator('[data-notebook-status]')).toContainText('storage is unavailable');
  await app.locator('[data-mistake-view]').selectOption('all');
  await app.locator('[data-note-correction]').fill('A temporary note.');
  await app.locator('[data-note-reviewed]').check();
  await expect(app.locator('[data-note-status]')).toContainText('marked checked');
});
test('notebook fits mobile and has no serious accessibility violations',async({page})=>{
  await page.setViewportSize({width:375,height:812});await page.goto(route);
  await page.locator('[data-work]').first().locator('summary').click();
  await page.locator('[data-mistake-view]').selectOption('all');
  for(const theme of ['light','dark']){
    await page.evaluate(t=>document.documentElement.dataset.theme=t,theme);
    expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);
    const audit=await new AxeBuilder({page}).include('main').analyze();expect(audit.violations.filter(v=>v.impact==='critical'||v.impact==='serious')).toEqual([]);
  }
});
test('workstreams and lesson links remain available without JavaScript',async({browser,baseURL})=>{
  const context=await browser.newContext({javaScriptEnabled:false,baseURL}),page=await context.newPage();await page.goto(route);
  await expect(page.locator('[data-work]')).toHaveCount(pscWorkstreams.length);await page.locator('[data-work]').first().locator('summary').click();
  await expect(page.locator('[data-work]').first().getByRole('link',{name:'Study the method'})).toBeVisible();await context.close();
});
