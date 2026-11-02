// Reading progress, stored only in this browser (localStorage). Nothing is sent anywhere:
// the site is static, so there is no server that could read it.
//
// What is stored, per article: title, module, furthest point reached (0–1), the section you were
// last reading, whether you finished it, and when. Plus the path of the last article you read.

export type ArticleProgress = {
  t: string; // title
  m: string; // module id
  mt: string; // module title
  p: number; // furthest progress, 0..1
  s?: string; // id of the section heading last in view
  st?: string; // that section's title
  done?: boolean;
  ts: number; // last visit, epoch ms
};

export type ProgressState = {
  v: 1;
  last?: string;
  articles: Record<string, ArticleProgress>;
};

const KEY = 'qe:progress:v1';
const DONE_AT = 0.92;
const STARTED_AT = 0.04;

export function load(): ProgressState {
  try {
    const raw = localStorage.getItem(KEY);
    if (raw) {
      const s = JSON.parse(raw);
      if (s && s.v === 1 && typeof s.articles === 'object') return s as ProgressState;
    }
  } catch {
    // Private browsing, blocked storage or corrupt data: behave as a first visit.
  }
  return { v: 1, articles: {} };
}

export function save(state: ProgressState): void {
  try {
    localStorage.setItem(KEY, JSON.stringify(state));
  } catch {
    // Storage full or blocked. Progress is a convenience; never break the page over it.
  }
}

export function clear(): void {
  try {
    localStorage.removeItem(KEY);
  } catch {
    /* ignore */
  }
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
    } else {
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
  const prior = state.articles[path];

  const bar = document.querySelector<HTMLElement>('[data-reading-bar]');
  const headings = [...content.querySelectorAll<HTMLElement>('h2[id], h3[id]')].filter(
    (h) => !h.closest('.not-content'),
  );

  const entry: ArticleProgress = prior
    ? { ...prior, t: meta.title, m: meta.module, mt: meta.moduleTitle }
    : { t: meta.title, m: meta.module, mt: meta.moduleTitle, p: 0, ts: Date.now() };
  let dirty = false;
  let becameLast = false;

  const measure = () => {
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
    if (p > entry.p + 0.01) {
      entry.p = p;
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
    const latest = load(); // another tab may have written meanwhile
    entry.ts = Date.now();
    latest.articles[path] = entry;
    if (becameLast) latest.last = path;
    save(latest);
  };

  let queued = false;
  addEventListener(
    'scroll',
    () => {
      if (queued) return;
      queued = true;
      requestAnimationFrame(() => {
        queued = false;
        measure();
      });
    },
    { passive: true },
  );
  addEventListener('resize', measure, { passive: true });
  setInterval(flush, 2000);
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
