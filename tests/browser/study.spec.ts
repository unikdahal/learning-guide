import {test,expect} from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

test('coverage filters, local confidence and revision queue survive reload',async({page})=>{
  await page.goto('nec/study-hub/');
  await expect(page.locator('[data-topic]')).toHaveCount(60);
  await page.locator('[data-topic="1.1"] [data-confidence]').selectOption('ready');
  await page.locator('[data-topic="1.1"] [data-written]').check();
  await expect(page.locator('[data-summary]')).toContainText('1 recall ready');
  await page.reload();
  await expect(page.locator('[data-topic="1.1"] [data-confidence]')).toHaveValue('ready');
  await expect(page.locator('[data-topic="1.1"] [data-written]')).toBeChecked();
  await page.locator('[data-path]').selectOption('co');
  await expect(page.locator('[data-topic="1.1"]')).toBeHidden();
  await page.locator('[data-search]').fill('7.3');
  await expect(page.locator('[data-topic]:visible')).toHaveCount(1);
  await expect(page.locator('[data-queue]')).toContainText('7.3');
});

test('corrupt storage and invalid imports leave the checklist usable',async({page})=>{
  await page.addInitScript(()=>localStorage.setItem('exam-preparation:coverage:v1','{"version":1,"topics":null}'));
  await page.goto('nec/study-hub/');
  await page.locator('[data-topic="2.1"] [data-confidence]').selectOption('learning');
  await page.locator('[data-import]').setInputFiles({name:'bad.json',mimeType:'application/json',buffer:Buffer.from('{"version":2}')});
  await expect(page.locator('[data-storage]')).toContainText('Import failed');
  await expect(page.locator('[data-topic="2.1"] [data-confidence]')).toHaveValue('learning');
});

test('project lab recomputes discounting and the critical-path switch',async({page})=>{
  await page.goto('nec/preparation/04-project-economics-and-cpm/');
  await page.locator('[data-rate]').fill('0');
  await expect(page.locator('[data-npv]')).toContainText('20.000');
  await page.locator('[data-duration]').fill('5');
  await expect(page.locator('[data-critical]')).toContainText('10 days');
  await expect(page.locator('[data-critical]')).toContainText('A–C–E');
  await page.locator('[data-years]').fill('1.5');
  await expect(page.locator('[data-error]')).toContainText('whole years');
});

test('study tools fit narrow screens and have no serious accessibility violations',async({page})=>{
  await page.setViewportSize({width:375,height:812});
  for(const path of ['nec/study-hub/','nec/practice/','nec/preparation/04-project-economics-and-cpm/']) {
    await page.goto(path);
    expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);
    const audit=await new AxeBuilder({page}).include('main').analyze();
    expect(audit.violations.filter(v=>v.impact==='critical'||v.impact==='serious')).toEqual([]);
  }
});

test('an overdue recall can be recorded again at the same confidence',async({page})=>{
  await page.addInitScript(()=>localStorage.setItem('exam-preparation:coverage:v1',JSON.stringify({version:1,topics:{'1.1':{confidence:'ready',written:false,updated:Date.now()-8*86400000}}})));
  await page.goto('nec/study-hub/');
  const row=page.locator('[data-topic="1.1"]');
  await expect(row.locator('[data-due]')).toHaveText('Revision due');
  await row.getByRole('button',{name:'Reviewed now for 1.1',exact:true}).click();
  await expect(row.locator('[data-due]')).toHaveText('');
  await expect(row.locator('[data-confidence]')).toHaveValue('ready');
});

test('syllabus and worked tables remain readable with JavaScript disabled',async({browser})=>{
  const context=await browser.newContext({javaScriptEnabled:false});
  const page=await context.newPage();
  await page.goto('http://127.0.0.1:4322/learning-guide/nec/study-hub/');
  await expect(page.locator('[data-topic]')).toHaveCount(60);
  await expect(page.locator('[data-topic="10.6"] a')).toBeVisible();
  await page.goto('http://127.0.0.1:4322/learning-guide/nec/preparation/04-project-economics-and-cpm/');
  await expect(page.locator('[data-critical]')).toContainText('8 days');
  await expect(page.locator('[data-cash] tr')).toHaveCount(4);
  await context.close();
});
