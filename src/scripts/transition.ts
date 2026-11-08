/** Transitions are decorative; a superseded animation must not reject into page code. */
export function transition(update: () => void): void {
  if (document.startViewTransition && !matchMedia('(prefers-reduced-motion: reduce)').matches) {
    const animation=document.startViewTransition(update);
    void animation.ready.catch(()=>{});
    void animation.finished.catch(()=>{});
  } else update();
}
