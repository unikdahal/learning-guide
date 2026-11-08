export type LevelKey = 'B' | 'I' | 'A' | 'E';
export type ArticleStatus = 'planned' | 'draft' | 'published';
export interface Article {
  index: number; title: string; slug: string; level: LevelKey;
  scope: string; visual: string | null; path: string;
}
export interface Module {
  id: string; slug: string; title: string; goal: string; prereqs: string[];
  outcomes: string[]; part: number; articles: Article[];
}
export interface Part {
  id: number; slug: string; title: string; color: string; summary: string; modules: Module[];
}
export interface SiteData {
  articleCount: number;
  parts: (Omit<Part, 'modules'> & { modules: string[] })[];
  modules: Record<string, Omit<Module, 'articles'> & { articles: (Article & { status: ArticleStatus })[] }>;
}
