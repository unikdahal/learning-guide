// Loads syllabus/*.yaml into one normalized structure.
// Used by astro.config.mjs (sidebar) and scripts/generate.mjs (pages + data).
import { readFileSync, readdirSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { parse } from 'yaml';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const syllabusDir = join(root, 'syllabus');

export const LEVELS = {
  B: { key: 'B', name: 'Beginner' },
  I: { key: 'I', name: 'Intermediate' },
  A: { key: 'A', name: 'Advanced' },
  E: { key: 'E', name: 'Expert' },
};

export function slugify(text) {
  return text
    .toLowerCase()
    .replace(/[`'’"]/g, '')
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '');
}

export function loadSyllabus() {
  const files = readdirSync(syllabusDir)
    .filter((f) => f.endsWith('.yaml'))
    .sort();

  const parts = [];
  const moduleIds = new Set();
  const slugs = new Set();

  for (const file of files) {
    const doc = parse(readFileSync(join(syllabusDir, file), 'utf8'));
    const part = { ...doc.part, modules: [] };

    for (const m of doc.modules) {
      if (moduleIds.has(m.id)) throw new Error(`Duplicate module id ${m.id} in ${file}`);
      moduleIds.add(m.id);

      const seen = new Set();
      const articles = m.articles.map((a, i) => {
        if (!LEVELS[a.l]) throw new Error(`Bad level "${a.l}" on "${a.t}" in ${file}`);
        const slug = a.slug ?? slugify(a.t);
        if (seen.has(slug)) throw new Error(`Duplicate slug ${slug} in module ${m.id}`);
        seen.add(slug);
        return {
          index: i + 1,
          title: a.t,
          slug,
          level: a.l,
          scope: a.d,
          visual: a.v ?? null,
          path: `${m.slug}/${slug}`,
        };
      });

      if (slugs.has(m.slug)) throw new Error(`Duplicate module slug ${m.slug}`);
      slugs.add(m.slug);

      part.modules.push({
        id: m.id,
        slug: m.slug,
        title: m.title,
        goal: m.goal,
        prereqs: m.prereqs ?? [],
        outcomes: m.outcomes ?? [],
        part: part.id,
        articles,
      });
    }
    parts.push(part);
  }

  const modules = parts.flatMap((p) => p.modules);
  for (const m of modules) {
    for (const p of m.prereqs) {
      if (!moduleIds.has(p)) throw new Error(`Module ${m.id} lists unknown prereq ${p}`);
    }
  }

  const articleCount = modules.reduce((n, m) => n + m.articles.length, 0);
  return { parts, modules, articleCount };
}
