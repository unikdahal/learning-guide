import syllabus from '../data/syllabus.json';

export type LevelKey = 'B' | 'I' | 'A' | 'E';

export const LEVELS: Record<LevelKey, { name: string; rank: number }> = {
  B: { name: 'Beginner', rank: 1 },
  I: { name: 'Intermediate', rank: 2 },
  A: { name: 'Advanced', rank: 3 },
  E: { name: 'Expert', rank: 4 },
};

export type Article = {
  index: number;
  title: string;
  slug: string;
  level: LevelKey;
  scope: string;
  visual: string | null;
  path: string;
  status: string;
};

export type Module = {
  id: string;
  slug: string;
  title: string;
  goal: string;
  prereqs: string[];
  outcomes: string[];
  part: number;
  articles: Article[];
};

export type Part = {
  id: number;
  slug: string;
  title: string;
  color: string;
  summary: string;
  modules: string[];
};

export const data = syllabus as unknown as {
  articleCount: number;
  parts: Part[];
  modules: Record<string, Module>;
};

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
