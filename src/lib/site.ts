import syllabus from '../data/syllabus.json';
import routeData from '../data/routes.json';
import type { SiteData, ArticleStatus, Article as CurriculumArticle, Module as CurriculumModule, LevelKey } from '../../scripts/types';
export type { ArticleStatus, LevelKey };
export const routes = routeData;

export const LEVELS: Record<LevelKey, { name: string; rank: number }> = {
  B: { name: 'Beginner', rank: 1 },
  I: { name: 'Intermediate', rank: 2 },
  A: { name: 'Advanced', rank: 3 },
  E: { name: 'Expert', rank: 4 },
};

export type Article = CurriculumArticle & {status: ArticleStatus};
export type Module = Omit<CurriculumModule,'articles'> & {articles: Article[]};
export type Part = SiteData['parts'][number];

// The generator and check-only content gate validate this contract before a build.
export const data = syllabus as SiteData;
export const publishedCount = Object.values(data.modules).flatMap(m=>m.articles).filter(a=>a.status==='published').length;
export const statusLabel = (status: ArticleStatus) => ({published:'Published',draft:'Draft',planned:'Planned'})[status];
export function routeSummary(steps: string[]) {
  const closure = new Set<string>();
  const visit = (id: string) => { if (closure.has(id)) return; closure.add(id); moduleById(id).prereqs.forEach(visit); };
  steps.forEach(visit);
  const articles = steps.flatMap(id=>moduleById(id).articles);
  const required = [...closure].flatMap(id=>moduleById(id).articles);
  return {total:articles.length,published:articles.filter(a=>a.status==='published').length,withPrerequisites:required.length};
}


/** Prefix an internal path with the site base, e.g. url('joins/') → '/learning-guide/joins/'. */
export function url(path = ''): string {
  const base = import.meta.env.BASE_URL.replace(/\/+$/, '');
  const clean = path.replace(/^\/+/, '');
  return `${base}/${clean}`;
}

export function moduleById(id: string): Module {
  const m = data.modules[id];
  if (!m) throw new Error(`Unknown module ${id}`);
  return m;
}

export function partOf(m: Module): Part {
  return data.parts.find((p) => p.id === m.part)!;
}

export function levelCounts(articles: Article[]): Record<LevelKey, number> {
  const c: Record<LevelKey, number> = { B: 0, I: 0, A: 0, E: 0 };
  for (const a of articles) c[a.level]++;
  return c;
}

export const allArticles = (): (Article & { module: Module; part: Part })[] =>
  data.parts.flatMap((p) =>
    p.modules.map(moduleById).flatMap((m) => m.articles.map((a) => ({ ...a, module: m, part: p }))),
  );

/** The guide's source repository, for links to lab code. */
export const REPO = 'https://github.com/unikdahal/learning-guide';

/** A link to a file in the lab, given its path under lab/crates/. */
export function labUrl(path: string): string {
  return `${REPO}/blob/main/lab/crates/${path}`;
}
