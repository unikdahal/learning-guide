/** Persisted progress is untrusted input, even when it came from this browser. */
export type ArticleProgress = {
  t: string; m: string; mt: string; p: number; ts: number;
  s?: string; st?: string; done?: boolean;
};
export type ProgressState = { v: 2; generation: string; last?: string; articles: Record<string, ArticleProgress> };
export const INITIAL_GENERATION = 'initial';
export const emptyState = (generation = INITIAL_GENERATION): ProgressState => ({v:2,generation,articles:{}});
const record = (x: unknown): x is Record<string, unknown> => !!x && typeof x === 'object' && !Array.isArray(x);
export function validArticlePath(path: string): boolean {
  return /^\/(?:[a-z0-9-]+\/)*[a-z0-9-]+\/[a-z0-9-]+\/$/.test(path);
}
export function sanitizeState(value: unknown, generation: string, known?: ReadonlySet<string>): ProgressState {
  const out = emptyState(generation);
  if (!record(value) || (value.v !== 1 && value.v !== 2) || !record(value.articles)) return out;
  if (value.v === 2 && value.generation !== generation) return out;
  if (value.v === 1 && generation !== INITIAL_GENERATION) return out;
  for (const [path,a] of Object.entries(value.articles)) {
    if (!validArticlePath(path) || (known && !known.has(path)) || !record(a)) continue;
    if (!['t','m','mt'].every(k=>typeof a[k]==='string') || !/^\d{2}$/.test(String(a.m))) continue;
    if (typeof a.p !== 'number' || !Number.isFinite(a.p) || a.p<0 || a.p>1 ||
        typeof a.ts !== 'number' || !Number.isFinite(a.ts) || a.ts<0 ||
        (a.done !== undefined && typeof a.done !== 'boolean')) continue;
    out.articles[path] = {t:a.t as string,m:a.m as string,mt:a.mt as string,p:a.p,ts:a.ts};
    const entry=out.articles[path];
    if (typeof a.s==='string' && /^[\w-]+$/.test(a.s)) entry.s=a.s;
    if (typeof a.st==='string') entry.st=a.st;
    if (typeof a.done==='boolean') entry.done=a.done;
  }
  if (typeof value.last==='string' && Object.hasOwn(out.articles,value.last)) out.last=value.last;
  return out;
}
/** Completion/furthest position are monotonic; the most recently read section wins. */
export function mergeProgress(prior: ArticleProgress | undefined, incoming: ArticleProgress): ArticleProgress {
  if (!prior) return {...incoming};
  const recent=incoming.ts >= prior.ts ? incoming : prior;
  return {...recent,p:Math.max(prior.p,incoming.p),done:!!(prior.done || incoming.done)};
}
