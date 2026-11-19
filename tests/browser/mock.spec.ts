import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
const route = 'nec/mock-centre/';
const mockKey = 'exam-preparation:mock:v1', practiceKey = 'learning-guide:preparation:v1';
// Shared-checkout dev validation must not reload mid-test when another task edits content.
test.beforeEach(async ({ page }) => { await page.routeWebSocket('**', () => {}); });

test('resume, navigator, editable answers, flags, confirmation and history survive reload', async ({ page }) => {
  await page.goto(route); const app = page.locator('mock-centre');
  await app.getByRole('button', { name: 'Start round', exact: true }).click();
  await expect(app.locator('[data-review] details')).toHaveCount(0);
  await app.getByRole('radio').nth(0).check();
  await app.getByRole('button', { name: 'Flag question', exact: true }).click();
  await app.getByRole('button', { name: 'Next question', exact: true }).click();
  await app.getByRole('radio').nth(1).check();
  await page.reload();
  await expect(app.getByRole('radio').nth(1)).toBeChecked();
  await expect(app.locator('[data-progress]')).toHaveText('2 answered · 18 unanswered · 1 flagged');
  await app.locator('[data-index="0"]').click();
  await expect(app.getByRole('radio').nth(0)).toBeChecked();
  await app.getByRole('radio').nth(2).check();
  await app.getByRole('button', { name: 'Submit round', exact: true }).click();
  await app.getByRole('button', { name: 'Keep answering', exact: true }).click();
  expect(await page.evaluate(key => localStorage.getItem(key), practiceKey)).toBeNull();
  await app.getByRole('button', { name: 'Submit round', exact: true }).click();
  await app.getByRole('button', { name: 'Confirm submission', exact: true }).click();
  await expect(app.locator('[data-summary]')).toContainText('18 blank');
  await expect(app.locator('[data-saved]')).toContainText('recorded once');
  await expect(app.getByRole('radio')).toHaveCount(0);
  const before = await page.evaluate(key => localStorage.getItem(key), practiceKey);
  await page.reload(); await expect(app.locator('[data-saved]')).toContainText('recorded once');
  expect(await page.evaluate(key => localStorage.getItem(key), practiceKey)).toEqual(before);
  const practice = JSON.parse(before!); expect(Object.values(practice.answers).map((r: any) => r.attempts)).toEqual([1, 1]);
});
test('absolute expiry on a closed/reloaded page records exactly once and locks answers', async ({ page }) => {
  await page.clock.install(); await page.goto(route); const app = page.locator('mock-centre');
  await app.locator('[name=minutes]').fill('1'); await app.getByRole('button', { name: 'Start round', exact: true }).click();
  await app.getByRole('radio').first().check();
  // Simulate returning after the tab was closed, rather than relying on interval ticks.
  await page.clock.setSystemTime(Date.now() + 120000);
  await page.reload(); await expect(app.locator('[data-summary]')).toContainText('Time expired');
  await expect(app.locator('[data-saved]')).toContainText('recorded once');
  const history = await page.evaluate(key => localStorage.getItem(key), practiceKey);
  await page.reload(); await expect(app.locator('[data-summary]')).toContainText('Time expired');
  expect(await page.evaluate(key => localStorage.getItem(key), practiceKey)).toEqual(history);
});
test('wrong-answer retry is a fresh limited round and scoring is configurable', async ({ page }) => {
  await page.goto(route); const app = page.locator('mock-centre');
  await app.locator('[name=penalty]').selectOption('0.2');
  await app.getByRole('button', { name: 'Start round', exact: true }).click();
  // Original bank's first chosen question: obtain its answer via module in the test runner.
  const { practiceQuestions } = await import('../../src/data/preparation-practice');
  await expect(app.locator('[data-round]')).toBeVisible();
  const id = await page.evaluate(key => JSON.parse(localStorage.getItem(key)!).session.ids[0], mockKey);
  const correct = practiceQuestions.find(q => q.id === id)!.answer;
  await app.getByRole('radio').nth((correct + 1) % 4).check();
  await app.getByRole('button', { name: 'Submit round', exact: true }).click(); await app.getByRole('button', { name: 'Confirm submission', exact: true }).click();
  await expect(app.locator('[data-summary]')).toContainText('Score -0.2');
  await app.getByRole('button', { name: 'Retry wrong answers', exact: true }).click();
  await expect(app.locator('[data-progress]')).toHaveText('0 answered · 1 unanswered · 0 flagged');
  await expect(app.locator('[data-meta]')).toContainText('Wrong-answer retry');
  await app.getByRole('radio').nth(correct).check();
  await app.getByRole('button', { name: 'Submit round', exact: true }).click(); await app.getByRole('button', { name: 'Confirm submission', exact: true }).click();
  await expect(app.locator('[data-saved]')).toContainText('recorded once');
  await expect(app.locator('[data-summary]')).toContainText('Score 1 / 1');
  await expect.poll(() => page.evaluate(({ key, id }) => JSON.parse(localStorage.getItem(key)!).answers[id].attempts, { key: practiceKey, id })).toBe(2);
});
test('phone layout and accessible setup, active round and report', async ({ page }) => {
  await page.setViewportSize({ width: 375, height: 812 }); await page.goto(route); const app = page.locator('mock-centre');
  for (const phase of ['setup', 'active', 'result']) {
    if (phase === 'active') await app.getByRole('button', { name: 'Start round', exact: true }).click();
    if (phase === 'result') { await app.getByRole('button', { name: 'Submit round', exact: true }).click(); await app.getByRole('button', { name: 'Confirm submission', exact: true }).click(); }
    for (const theme of ['light', 'dark']) {
      await page.evaluate(t => document.documentElement.setAttribute('data-theme', t), theme);
      expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1)).toBe(true);
      const audit = await new AxeBuilder({ page }).include('mock-centre').analyze();
      expect(audit.violations.filter(v => v.impact === 'serious' || v.impact === 'critical')).toEqual([]);
    }
  }
});
test('blocked storage still permits in-memory training', async ({ page }) => {
  await page.addInitScript(() => Object.defineProperty(window, 'localStorage', { get() { throw new Error('blocked'); } }));
  await page.goto(route); const app = page.locator('mock-centre');
  await expect(app.locator('[data-notice]')).toContainText('storage is unavailable');
  await app.getByRole('button', { name: 'Start round', exact: true }).click(); await app.getByRole('radio').first().check();
  await app.getByRole('button', { name: 'Submit round', exact: true }).click(); await app.getByRole('button', { name: 'Confirm submission', exact: true }).click();
  await expect(app.locator('[data-saved]')).toContainText('practice history was not saved');
});
test('no JavaScript gives clear links to the complete static bank', async ({ browser, baseURL }) => {
  const context = await browser.newContext({ javaScriptEnabled: false, baseURL }); const page = await context.newPage(); await page.goto(route);
  await expect(page.locator('mock-centre [data-live]')).toBeHidden();
  await expect(page.getByRole('link', { name: 'Read all 120 original practice questions', exact: true })).toBeVisible();
  await context.close();
});

test('a running timer finalises once and later ticks never add attempts', async ({ page }) => {
  await page.clock.install(); await page.goto(route); const app = page.locator('mock-centre');
  await app.locator('[name=minutes]').fill('1');
  await app.getByRole('button', { name: 'Start round', exact: true }).click();
  await app.getByRole('radio').first().check();
  await page.clock.fastForward(61000);
  await expect(app.locator('[data-summary]')).toContainText('Time expired');
  await expect(app.locator('[data-saved]')).toContainText('recorded once');
  const before = await page.evaluate(key => localStorage.getItem(key), practiceKey);
  await page.clock.fastForward(120000);
  expect(await page.evaluate(key => localStorage.getItem(key), practiceKey)).toEqual(before);
  await expect(app.getByRole('radio')).toHaveCount(0);
});

for (const failure of ['start', 'answer']) test(`quota failure on ${failure} preserves active answers and permits saving retry`, async ({page}) => {
  await page.goto(route);
  const app=page.locator('mock-centre');
  const failWrites=()=>page.evaluate(()=>{
    const original=Storage.prototype.setItem;
    (window as any).restoreStorage=()=>{Storage.prototype.setItem=original;};
    Storage.prototype.setItem=function(k,v){if(k==='exam-preparation:mock:v1')throw new DOMException('Quota','QuotaExceededError');original.call(this,k,v);};
  });
  if(failure==='start')await failWrites();
  await app.getByRole('button',{name:'Start round',exact:true}).click();
  if(failure==='answer')await failWrites();
  await app.getByRole('radio').first().check();
  await app.getByRole('button',{name:'Next question',exact:true}).click();
  await expect(app.locator('[data-progress]')).toContainText('1 answered');
  await app.locator('[data-index="0"]').click();
  await expect(app.getByRole('radio').first()).toBeChecked();
  await page.evaluate(()=>(window as any).restoreStorage());
  await app.locator('[data-save-active]').click();
  await expect(app.locator('[data-save-active]')).toBeHidden();
  await expect.poll(()=>page.evaluate(key=>Object.keys(JSON.parse(localStorage.getItem(key)!).session.answers).length,mockKey)).toBe(1);
  await page.reload();
  await expect(app.getByRole('radio').first()).toBeChecked();
});

test('practice submission and mock finalisation share the history lock across tabs', async({page,context})=>{
  await page.goto(route);const app=page.locator('mock-centre');
  await app.getByRole('button',{name:'Start round',exact:true}).click();
  await app.getByRole('radio').first().check();
  const practice=await context.newPage();await practice.goto('nec/practice/');
  await practice.evaluate(()=>{
    (window as any).lockHeld=false;
    void navigator.locks.request('exam-preparation:history',()=>new Promise<void>(resolve=>{
      (window as any).releaseHistory=resolve;(window as any).lockHeld=true;
    }));
  });
  await expect.poll(()=>practice.evaluate(()=>(window as any).lockHeld)).toBe(true);
  await app.getByRole('button',{name:'Submit round',exact:true}).click();
  await app.getByRole('button',{name:'Confirm submission',exact:true}).click();
  await practice.locator('nec-practice input[name="practice-answer"]').first().check();
  await practice.locator('[data-check]').click();
  expect(await page.evaluate(key=>localStorage.getItem(key),practiceKey)).toBeNull();
  await practice.evaluate(()=>(window as any).releaseHistory());
  await expect(app.locator('[data-saved]')).toContainText('recorded once');
  await expect.poll(()=>page.evaluate(key=>Object.values(JSON.parse(localStorage.getItem(key)!).answers).reduce((n:number,r:any)=>n+r.attempts,0),practiceKey)).toBe(2);
  await practice.close();
});
