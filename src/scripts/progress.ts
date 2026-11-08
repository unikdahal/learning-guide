// Reading progress, stored only in this browser (localStorage). Nothing is sent anywhere:
// this application does not transmit reading history.
//
// What is stored, per article: title, module, furthest point reached (0–1), the section you were
// last reading, whether you finished it, and when. Plus the path of the last article you read.

import { emptyState, INITIAL_GENERATION, sanitizeState, mergeProgress, type ArticleProgress, type ProgressState } from './progress-state';
export type {ArticleProgress,ProgressState} from './progress-state';
export const KEY = 'qe:progress:v2';
const LEGACY_KEY = 'qe:progress:v1';
const GENERATION_KEY = 'qe:progress:generation';
const DONE_AT = 0.92;
const STARTED_AT = 0.04;
let knownPaths: ReadonlySet<string> | undefined;
export function configureProgress(paths: string[]) { knownPaths=new Set(paths); }
function generation(): string {
  try { return localStorage.getItem(GENERATION_KEY) || INITIAL_GENERATION; }
  catch { return INITIAL_GENERATION; }
}
export function load(): ProgressState {
  const g=generation();
  try {
    const current=localStorage.getItem(KEY);
    if (current) return sanitizeState(JSON.parse(current),g,knownPaths);
    if (g===INITIAL_GENERATION) {
      const old=localStorage.getItem(LEGACY_KEY);
      if (old) {
        const migrated=sanitizeState(JSON.parse(old),g,knownPaths);
        save(migrated);
        localStorage.removeItem(LEGACY_KEY);
        return migrated;
      }
    }
  } catch { /* Blocked storage and malformed JSON behave as a first visit. */ }
  return emptyState(g);
}
export function save(state: ProgressState): void {
  try { if (state.generation===generation()) localStorage.setItem(KEY,JSON.stringify(state)); }
  catch { /* Progress is optional: quota errors must never break the lesson. */ }
}
function transaction(write: () => void): void {
  if (navigator.locks) void navigator.locks.request('qe:progress',write).catch(()=>{});
  else write(); // Older browsers still receive monotonic merge and durable reset protection.
}
export function clear(): void {
  transaction(()=>{
  try {
    // A durable generation prevents an old tab (or v1 migration) reviving deleted history.
    const g=crypto.randomUUID();
    localStorage.setItem(GENERATION_KEY,g);
    localStorage.removeItem(LEGACY_KEY);
    localStorage.setItem(KEY,JSON.stringify(emptyState(g)));
  } catch { /* Storage may be disabled. */ }
  document.dispatchEvent(new Event('qe:progress-change'));
  });
}
/** Canonical form of a site path: always with a trailing slash, no hash or query. */
export function canonical(href: string): string {
  const u = new URL(href, location.href);
  return u.pathname.endsWith('/') ? u.pathname : `${u.pathname}/`;
}

export function statusOf(a: ArticleProgress | undefined): 'done' | 'started' | 'none' {
  if (!a) return 'none';
  if (a.done) return 'done';
  return a.p >= STARTED_AT ? 'started' : 'none';
}

/** Marks every tracked link (sidebar, module lists, roadmap) as started or read. */
export function annotateLinks(state: ProgressState): void {
  const links = document.querySelectorAll<HTMLAnchorElement>('nav.sidebar a[href], [data-track-links] a[href]');
  for (const a of links) {
    let path: string;
    try {
      path = canonical(a.getAttribute('href')!);
    } catch {
      continue;
    }
    const st = statusOf(state.articles[path]);
    if (st === 'none') {
      delete a.dataset.progress;
      if (a.dataset.progressTitle) { a.title=a.dataset.progressTitle; delete a.dataset.progressTitle; }
    } else {
      a.dataset.progressTitle ??= a.title;
      a.dataset.progress = st;
      a.title = st === 'done' ? 'Read' : `Started, ${Math.round(state.articles[path].p * 100)}%`;
    }
  }
}

type ArticleMetaData = { title: string; module: string; moduleTitle: string };

/** Tracks reading on an article page and offers to resume. */
export function trackArticle(meta: ArticleMetaData): void {
  const content = document.querySelector<HTMLElement>('.sl-markdown-content');
  if (!content) return;
  const path = canonical(location.href);
  const state = load();
  let activeGeneration=state.generation;
  const prior = state.articles[path];

  const bar = document.querySelector<HTMLElement>('[data-reading-bar]');
  const headings = [...content.querySelectorAll<HTMLElement>('h2[id], h3[id]')].filter(
    (h) => !h.closest('.not-content'),
  );

  let entry: ArticleProgress = prior
    ? { ...prior, t: meta.title, m: meta.module, mt: meta.moduleTitle }
    : { t: meta.title, m: meta.module, mt: meta.moduleTitle, p: 0, ts: Date.now() };
  let dirty = false;
  let becameLast = false;
  let waitingForReading = false;
  const reset = (g: string) => {
    activeGeneration=g; dirty=false; becameLast=false; waitingForReading=true;
    entry={t:meta.title,m:meta.module,mt:meta.moduleTitle,p:0,ts:Date.now()};
    const toast=document.querySelector<HTMLElement>('[data-resume-toast]');
    if (toast) toast.hidden=true;
    annotateLinks(load());
  };
  const storageChanged = () => {
    const latest=load();
    if (latest.generation!==activeGeneration) reset(latest.generation);
    annotateLinks(latest);
  };
  addEventListener('storage',storageChanged);
  document.addEventListener('qe:progress-change',storageChanged);

  const measure = (reading = false) => {
    const latestGeneration=generation();
    if (latestGeneration!==activeGeneration) reset(latestGeneration);
    if (waitingForReading && !reading) return;
    if (reading) waitingForReading=false;
    const rect = content.getBoundingClientRect();
    const seen = (window.innerHeight - rect.top) / rect.height;
    const p = Math.max(0, Math.min(1, seen));
    if (bar) bar.style.transform = `scaleX(${p})`;

    // The section being read: the last heading above the top fifth of the screen.
    const line = window.innerHeight * 0.2;
    let current: HTMLElement | undefined;
    for (const h of headings) {
      if (h.getBoundingClientRect().top <= line) current = h;
      else break;
    }
    if (current && current.id !== entry.s) {
      entry.s = current.id;
      entry.st = current.textContent?.trim() ?? '';
      dirty = true;
    }
    if (dirty) entry.ts=Date.now();
    if (p > entry.p + 0.01) {
      entry.p = p;
      entry.ts=Date.now();
      dirty = true;
    }
    if (!entry.done && entry.p >= DONE_AT) {
      entry.done = true;
      dirty = true;
    }
    // Only count the page as "where I was" once the reader has actually started reading it.
    if (!becameLast && p >= STARTED_AT && rect.top < 0) {
      becameLast = true;
      dirty = true;
    }
  };

  const flush = () => {
    if (!dirty) return;
    dirty = false;
    const incoming={...entry};
    transaction(()=>{
      const latest=load();
      if (latest.generation!==activeGeneration) { reset(latest.generation); return; }
      const merged=mergeProgress(latest.articles[path],incoming);
      latest.articles[path]=merged;
      entry=mergeProgress(entry,merged);
      if (becameLast && merged.ts >= (latest.last ? latest.articles[latest.last]?.ts ?? 0 : 0)) latest.last=path;
      save(latest);
    });
  };

  let queued = false;
  addEventListener(
    'scroll',
    () => {
      if (queued) return;
      queued = true;
      requestAnimationFrame(() => {
        queued = false;
        measure(true);
      });
    },
    { passive: true },
  );
  addEventListener('resize', () => measure(), { passive: true });
  const timer=setInterval(flush,2000);
  addEventListener('pagehide',()=>clearInterval(timer),{once:true});
  addEventListener('pageshow',event=>{ if (event.persisted) location.reload(); });
  addEventListener('pagehide', flush);
  document.addEventListener('visibilitychange', () => document.visibilityState === 'hidden' && flush());
  measure();

  // Offer to resume when coming back to an unfinished article without a specific anchor.
  if (prior && !prior.done && prior.s && prior.p > 0.08 && !location.hash) {
    const target = document.getElementById(prior.s);
    if (target) showResume(target, prior.st || 'where you left off', prior.p);
  }
}

function showResume(target: HTMLElement, sectionTitle: string, p: number): void {
  const toast = document.querySelector<HTMLElement>('[data-resume-toast]');
  if (!toast) return;
  toast.querySelector('[data-resume-section]')!.textContent = sectionTitle;
  toast.querySelector('[data-resume-pct]')!.textContent = `${Math.round(p * 100)}%`;
  toast.hidden = false;
  requestAnimationFrame(() => toast.classList.add('is-open'));

  const close = () => {
    toast.classList.remove('is-open');
    setTimeout(() => (toast.hidden = true), 200);
    removeEventListener('keydown', onKey);
  };
  const onKey = (e: KeyboardEvent) => e.key === 'Escape' && close();
  addEventListener('keydown', onKey);
  toast.querySelector('[data-resume-go]')!.addEventListener('click', () => {
    close();
    const smooth = !matchMedia('(prefers-reduced-motion: reduce)').matches;
    target.scrollIntoView({ behavior: smooth ? 'smooth' : 'auto', block: 'start' });
    history.replaceState(null, '', `#${target.id}`);
  });
  toast.querySelector('[data-resume-close]')!.addEventListener('click', close);
  // If the reader starts scrolling on their own, the offer has served its purpose.
  const startY = scrollY;
  addEventListener(
    'scroll',
    function onScroll() {
      if (Math.abs(scrollY - startY) > innerHeight * 0.75) {
        close();
        removeEventListener('scroll', onScroll);
      }
    },
    { passive: true },
  );
}
