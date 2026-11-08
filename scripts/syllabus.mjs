// One validated curriculum contract, shared by generation and the sidebar.
import { readFileSync, readdirSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { parse } from 'yaml';

const syllabusDir = join(dirname(fileURLToPath(import.meta.url)), '../syllabus');
export const LEVELS = {
  B: { key: 'B', name: 'Beginner' }, I: { key: 'I', name: 'Intermediate' },
  A: { key: 'A', name: 'Advanced' }, E: { key: 'E', name: 'Expert' },
};
export function slugify(text) {
  return text.toLowerCase().replace(/[`'’"]/g, '').replace(/[^a-z0-9]+/g, '-').replace(/^-+|-+$/g, '');
}
function requireText(value, where) {
  if (typeof value !== 'string' || !value.trim()) throw new Error(`${where}: expected a nonempty string`);
  return value;
}
function slug(value, where) {
  requireText(value, where);
  if (!/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(value) || value === 'index' || value === '404') {
    throw new Error(`${where}: unsafe or reserved slug ${value}`);
  }
  return value;
}
/** Validate before any generated file is written. Accept documents separately for fixture tests.
 * @param {{file:string, data:any}[]} documents
 * @returns {{ parts: import('./types').Part[], modules: import('./types').Module[], articleCount:number }}
 */
export function normalizeSyllabus(documents) {
  /** @type {import('./types').Part[]} */
  const parts = [];
  const partIds = new Set(), partSlugs = new Set(), moduleIds = new Set(), moduleSlugs = new Set();
  for (const {file, data: doc} of documents) {
    if (!doc?.part || !Array.isArray(doc.modules)) throw new Error(`${file}: expected part and modules`);
    const p = doc.part;
    if (!Number.isInteger(p.id) || p.id < 1 || partIds.has(p.id)) throw new Error(`${file}: invalid or duplicate part id ${p.id}`);
    partIds.add(p.id);
    slug(p.slug, `${file}: part.slug`);
    if (partSlugs.has(p.slug)) throw new Error(`${file}: duplicate part slug ${p.slug}`);
    partSlugs.add(p.slug);
    if (!/^#[0-9a-f]{6}$/i.test(p.color)) throw new Error(`${file}: part.color must be a six-digit hex color`);
    const part = { id:p.id, slug:p.slug, title:requireText(p.title, `${file}: part.title`),
      color:p.color, summary:requireText(p.summary, `${file}: part.summary`), modules:[] };
    for (const m of doc.modules) {
      const where = `${file}: module ${m?.id}`;
      if (typeof m?.id !== 'string' || !/^\d{2}$/.test(m.id) || moduleIds.has(m.id)) throw new Error(`${where}: invalid or duplicate module id`);
      moduleIds.add(m.id);
      slug(m.slug, `${where}.slug`);
      if (['roadmap','style-guide','lab-numbers'].includes(m.slug) || moduleSlugs.has(m.slug)) throw new Error(`${where}: reserved or duplicate module slug ${m.slug}`);
      moduleSlugs.add(m.slug);
      if (!Array.isArray(m.articles) || !m.articles.length) throw new Error(`${where}: articles must be nonempty`);
      for (const field of ['prereqs','outcomes']) {
        if (m[field] !== undefined && (!Array.isArray(m[field]) || m[field].some(x => typeof x !== 'string' || !x.trim()))) throw new Error(`${where}: invalid ${field}`);
      }
      const seen = new Set();
      const articles = m.articles.map((a, i) => {
        const label = `${where}: article ${i+1}`;
        const title = requireText(a.t, `${label}.t`);
        if (!Object.hasOwn(LEVELS,a.l)) throw new Error(`${label}: invalid level ${a.l}`);
        const articleSlug = slug(a.slug ?? slugify(title), `${label}.slug`);
        if (seen.has(articleSlug)) throw new Error(`${label}: duplicate slug ${articleSlug}`);
        seen.add(articleSlug);
        if (a.v !== undefined) requireText(a.v, `${label}.v`);
        return {index:i+1, title, slug:articleSlug, level:a.l, scope:requireText(a.d, `${label}.d`), visual:a.v ?? null, path:`${m.slug}/${articleSlug}`};
      });
      part.modules.push({id:m.id, slug:m.slug, title:requireText(m.title, `${where}.title`), goal:requireText(m.goal, `${where}.goal`),
        prereqs:m.prereqs ?? [], outcomes:m.outcomes ?? [], part:p.id, articles});
    }
    parts.push(part);
  }
  const modules = parts.flatMap(p => p.modules), byId = new Map(modules.map(m => [m.id,m]));
  const done = new Set(), active = new Set();
  function visit(id, chain = []) {
    if (active.has(id)) throw new Error(`Prerequisite cycle: ${[...chain,id].join(' -> ')}`);
    if (done.has(id)) return;
    const m = byId.get(id);
    if (!m) throw new Error(`Unknown prerequisite ${id} in ${chain.at(-1)}`);
    active.add(id);
    for (const prereq of m.prereqs) visit(prereq, [...chain,id]);
    active.delete(id);done.add(id);
  }
  modules.forEach(m => visit(m.id));
  return {parts,modules,articleCount:modules.reduce((n,m)=>n+m.articles.length,0)};
}
export function loadSyllabus(directory = syllabusDir) {
  return normalizeSyllabus(readdirSync(directory).filter(f=>f.endsWith('.yaml')).sort().map(file=>({file,data:parse(readFileSync(join(directory,file),'utf8'))})));
}
