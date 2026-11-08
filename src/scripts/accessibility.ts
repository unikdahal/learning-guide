/** Keep dynamically overflowing instructional content reachable with a keyboard. */
export function enhanceScrollableExamples(): void {
  const main=document.querySelector('main');
  if (!main) return;
  let queued=false;
  const scan=()=>{
    queued=false;
    for (const element of main.querySelectorAll<HTMLElement>('pre, .tablewrap, .table-wrap, .grid, .plot, table')) {
      const overflow=getComputedStyle(element).overflowX;
      if (element.scrollWidth>element.clientWidth+1 && ['auto','scroll'].includes(overflow)) {
        element.tabIndex=0;
        if (element.tagName!=='TABLE') element.setAttribute('role','region');
        if (!element.hasAttribute('aria-label')) element.setAttribute('aria-label',element.tagName==='PRE' ? 'Scrollable code example' : 'Scrollable worked example');
      }
    }
  };
  const schedule=()=>{if(!queued){queued=true;requestAnimationFrame(scan);}};
  new MutationObserver(schedule).observe(main,{childList:true,subtree:true});
  new ResizeObserver(schedule).observe(main);
  document.fonts.ready.then(schedule);schedule();
}
