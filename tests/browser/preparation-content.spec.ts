import {test,expect} from '@playwright/test';
const lessons = [
  'nec/preparation/01-automata-and-graphics-traces/',
  'nec/preparation/02-ai-search-and-neural-networks/',
  'nec/preparation/03-embedded-systems-rtos-vhdl/',
  'nec/preparation/04-project-economics-and-cpm/',
  'nec/preparation/05-engineering-drawing-and-professional-practice/',
  'nec/preparation/06-cfl-and-parametric-clipping/',
  'nec/preparation/07-probability-and-learning-traces/',
  'lok-sewa/preparation/',
  'lok-sewa/preparation/source-register/',
  'lok-sewa/preparation/written-answer-workshop/',
  'lok-sewa/preparation/ce-gap-units/',
  'lok-sewa/preparation/co-non-overlap/',
  'lok-sewa/preparation/05-paper-one-reasoning/',
  'lok-sewa/preparation/06-emerging-technology-cases/',
];
for(const lesson of lessons) test(`worked content fits mobile and loads cleanly: ${lesson}`,async({page})=>{
  const errors:string[]=[];
  page.on('pageerror',e=>errors.push(e.message));
  await page.setViewportSize({width:375,height:812});
  await page.goto(lesson);
  await expect(page.locator('h1')).toBeVisible();
  await expect(page.locator('main svg[role="img"]').first()).toBeVisible();
  const layout=await page.evaluate(()=>({fits:document.documentElement.scrollWidth<=innerWidth+1,
    overflow:Array.from(document.querySelectorAll('main *')).filter(el=>el.getBoundingClientRect().right>innerWidth+2).sort((a,b)=>b.getBoundingClientRect().right-a.getBoundingClientRect().right).slice(0,6).map(el=>({tag:el.tagName,classes:el.getAttribute('class'),width:el.getBoundingClientRect().width,text:el.textContent?.slice(0,60)}))}));
  expect(layout.fits,JSON.stringify(layout.overflow)).toBe(true);
  await expect(page.locator('.katex-error')).toHaveCount(0);
  if(lesson.startsWith('nec/')&&!lesson.includes('04-project'))expect(await page.locator('math').count()).toBeGreaterThan(20);
  expect(errors).toEqual([]);
});
