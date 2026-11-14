import {test,expect} from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import {readFileSync} from 'node:fs';
const data=JSON.parse(readFileSync(new URL('../../src/data/syllabus.json',import.meta.url),'utf8'));
const published=Object.values(data.modules).flatMap((m:any)=>m.articles).filter((a:any)=>a.status==='published') as {path:string}[];
const article='relational-model-and-sql/relations-tuples-and-attributes/';
const key='qe:progress:v2';
test('availability and roadmap distinguish published articles from the blueprint',async({page})=>{
 await page.goto('./');await expect(page.locator('.lede')).toContainText(`${published.length} published`);
 await page.goto('roadmap/');await expect(page.locator('[data-results]>li:visible')).toHaveCount(published.length);
 await page.locator('[data-show-all]').click();await expect(page.locator('[data-results]>li:visible')).toHaveCount(data.articleCount);
 await page.locator('[data-publication]').selectOption('planned');await expect(page.locator('[data-results]>li:visible')).toHaveCount(data.articleCount-published.length);
});
test('corrupt storage does not break a lesson',async({page})=>{
 const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
 await page.addInitScript(()=>localStorage.setItem('qe:progress:v1',JSON.stringify({v:1,articles:null})));
 await page.goto(article);await expect(page.locator('h1')).toContainText('Relations');expect(errors).toEqual([]);
});
test('two tabs retain completion and respect a durable reset',async({page,context})=>{
 await page.goto(article);await page.evaluate(()=>scrollTo(0,document.body.scrollHeight*.2));await page.waitForTimeout(2300);
 const other=await context.newPage();await other.goto(article);await other.evaluate(()=>scrollTo(0,document.body.scrollHeight));await other.waitForTimeout(2300);
 await page.evaluate(()=>scrollBy(0,400));await page.waitForTimeout(2300);
 const path='/learning-guide/'+article;
 const state=await page.evaluate(k=>JSON.parse(localStorage.getItem(k)!),key);expect(state.articles[path].done).toBe(true);expect(state.articles[path].p).toBe(1);
 await other.goto('./');other.on('dialog',d=>d.accept());await other.locator('[data-clear]').click();
 await page.waitForTimeout(2300);expect(await page.evaluate(k=>Object.keys(JSON.parse(localStorage.getItem(k)!).articles).length,key)).toBe(0);
 await page.reload();expect(await page.evaluate(k=>Object.keys(JSON.parse(localStorage.getItem(k)!).articles).length,key)).toBe(0);
});
test('combined grouping controls and natural-join headings agree with SQL',async({page})=>{
 await page.goto('relational-model-and-sql/grouping-and-aggregation-semantics/');
 for(const v of ['empty','nogroup','having'])await page.locator(`qe-groups [data-v="${v}"]`).click();
 await expect(page.locator('qe-groups [data-note]')).toContainText('Final output: 0 rows.');
 await page.goto('relational-model-and-sql/joins-in-relational-algebra/');await page.locator('[data-cond="natural"]').click();
 expect(await page.locator('qe-joingrid [data-result] th').allTextContents()).toEqual(['cust_id','name','order_id','amount']);
});
test('scenario tabs support arrow keys and reduced motion pauses the hero',async({page})=>{
 await page.goto('orientation/measuring-things-honestly/');await page.locator('.qe-wvc [role=tab]').first().focus();await page.keyboard.press('ArrowRight');
 await expect(page.locator('.qe-wvc [aria-selected=true]')).toHaveText('Four threads');
 await page.goto('relational-model-and-sql/window-function-semantics/');
 const row=page.locator('qe-frames [data-list] button').first();await row.focus();await page.keyboard.press('Enter');
 await expect(page.locator('qe-frames [data-list] button').first()).toBeFocused();
 await expect(page.locator('qe-frames [data-list] button').first()).toHaveAttribute('aria-pressed','true');
 await page.emulateMedia({reducedMotion:'reduce'});await page.goto('./');expect(await page.locator('.hero-plan svg').evaluate((s:SVGSVGElement)=>s.animationsPaused())).toBe(true);
});
test('static worked examples remain without JavaScript',async({browser})=>{
 const context=await browser.newContext({javaScriptEnabled:false});const page=await context.newPage();
 await page.goto('http://127.0.0.1:4322/learning-guide/relational-model-and-sql/null-and-three-valued-logic/');await expect(page.locator('.static-example')).toContainText('WHERE');await context.close();
});
test('each guide shows only its own sidebar and title',async({page})=>{
 await page.goto('nec/roadmap/');await expect(page.locator('.site-title')).toContainText('NEC Licence');
 await expect(page.locator('nav[aria-label="Main"] a[href*="/lok-sewa/"]')).toHaveCount(0);
 await expect(page).toHaveTitle(/\| NEC Licence$/);
 await page.goto('lok-sewa/computer-engineer/');await expect(page.locator('.site-title')).toContainText('Lok Sewa');
 await expect(page.locator('nav[aria-label="Main"] a[href*="/nec/"]')).toHaveCount(0);
 await expect(page.locator('nav[aria-label="Main"] a[href*="/orientation/"]')).toHaveCount(0);
 await expect(page).toHaveTitle(/\| Lok Sewa$/);
 await page.goto('orientation/');await expect(page.locator('.site-title')).toContainText('Query Execution');
 await expect(page.locator('nav[aria-label="Main"] a[href*="/lok-sewa/"], nav[aria-label="Main"] a[href*="/nec/"]')).toHaveCount(0);
 await page.goto('guides/');await page.locator('main a[href$="/lok-sewa/"]').click();await expect(page).toHaveURL(/\/lok-sewa\/$/);
});
for(const path of ['', 'roadmap/', 'guides/', 'nec/', 'nec/roadmap/', 'nec/syllabus/', 'nec/exam/', 'nec/chapters/01-electrical-electronics/', 'nec/chapters/05-networks-security/', 'lok-sewa/computer-officer/', 'nec/electronics/02-charge-current-voltage/', 'nec/electronics/06-star-delta/', 'nec/electronics/08-review-dc-basics/', 'nec/electronics/03-resistance-ohm-power/', 'nec/electronics/04-series-and-parallel/', 'nec/electronics/10-thevenin/', 'nec/electronics/12-maximum-power-transfer/', 'nec/electronics/14-capacitors/', 'nec/electronics/16-rc-rl-transients/', 'nec/electronics/17-ac-basics/', 'nec/electronics/19-reactance-impedance/', 'nec/electronics/20-resonance/', 'nec/electronics/21-ac-power/', 'nec/electronics/22-three-phase/', 'nec/electronics/15b-magnetic-circuits-transformers/', 'nec/electronics/19b-filters-frequency-response/', 'nec/electronics/25-pn-junction-diode/', 'nec/electronics/26-rectifiers-regulators/', 'nec/electronics/27-bjt-basics/', 'nec/electronics/28-bjt-biasing/', 'nec/electronics/33-op-amps/', 'nec/electronics/35-review-devices/', 'nec/digital/00-start-here/', 'nec/digital/01-number-systems/', 'nec/digital/03-logic-gates/', 'nec/digital/05-karnaugh-maps/', 'nec/digital/06-arithmetic-circuits/', 'nec/digital/07-mux-decoder-encoder/', 'nec/digital/08-flip-flops/', 'nec/digital/09-registers-counters/', 'nec/digital/11-8085/', 'nec/digital/12-8086/', 'nec/digital/15-review/', 'lok-sewa/', 'lok-sewa/computer-engineer/',...published.map(a=>a.path+'/')]) {
 test(`accessible main content in both themes: ${path || 'home'}`,async({page})=>{
  const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));await page.goto(path || './');
  for(const theme of ['light','dark']) {
   await page.evaluate(t=>document.documentElement.dataset.theme=t,theme);await page.waitForTimeout(350);
   const result=await new AxeBuilder({page}).include('main').withTags(['wcag2a','wcag2aa','wcag21aa']).analyze();
   expect(result.violations.map(v=>({id:v.id,nodes:v.nodes.map(n=>n.target)}))).toEqual([]);
  }
  expect(errors).toEqual([]);await page.setViewportSize({width:375,height:812});await page.waitForTimeout(150);
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);
 });
}
