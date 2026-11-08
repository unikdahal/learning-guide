// Generates content from syllabus/*.yaml:
//   - src/data/syllabus.json (used by the roadmap and article components)
//   - src/content/docs/<module>/index.mdx  (module overviews; regenerated unless edited by hand)
//   - src/content/docs/<module>/<article>.mdx (stubs; NEVER overwrites an existing article)
import { existsSync, mkdirSync, readFileSync, writeFileSync, readdirSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { loadSyllabus } from './syllabus.mjs';
import { articleStatus, frontmatter } from './frontmatter.mjs';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const docsDir = join(root, 'src/content/docs');
const dataDir = join(root, 'src/data');

const { parts, modules, articleCount } = loadSyllabus();

const yamlStr = (s) => JSON.stringify(s); // JSON strings are valid YAML scalars

const check = process.argv.includes('--check');
const problems = [];
const outputs = new Map();
const write = (file, text) => {
  if (check) {
    if (!existsSync(file) || readFileSync(file,'utf8') !== text) problems.push(`${file}: generated output is stale; run npm run generate`);
  } else outputs.set(file,text);
};
// Published status lives in each article's frontmatter, so read it back for the data file.
function readStatus(file) {
  if (!existsSync(file)) return 'planned';
  return articleStatus(readFileSync(file, 'utf8'), file);
}

let created = 0;
let overviews = 0;

for (const m of modules) {
  const dir = join(docsDir, m.slug);

  // Module overview: regenerate only while it still carries the generated marker.
  const indexFile = join(dir, 'index.mdx');
  const isGenerated = !existsSync(indexFile) || frontmatter(readFileSync(indexFile,'utf8'),indexFile).data.generated === true;
  if (isGenerated) {
    write(
      indexFile,
      `---
title: ${yamlStr(m.title)}
description: ${yamlStr(m.goal)}
generated: true
sidebar:
  order: 0
  label: Overview
---
{/* Generated from syllabus/. Delete the "generated: true" line above to take ownership of this page. */}
import ModuleOverview from '@components/ModuleOverview.astro';

<ModuleOverview id="${m.id}" />
`,
    );
    overviews++;
  }

  for (const a of m.articles) {
    const file = join(dir, `${a.slug}.mdx`);
    if (existsSync(file)) {
      const text = readFileSync(file,'utf8');
      const fm = frontmatter(text,file).data;
      for (const [key,want] of Object.entries({title:a.title,level:a.level,module:m.id})) {
        if (fm[key] !== want) problems.push(`${file}: ${key} differs from syllabus (${want})`);
      }
      if (fm.sidebar?.order !== a.index) problems.push(`${file}: sidebar.order must be ${a.index}`);
      const status = articleStatus(text,file);
      if ((fm.sidebar?.attrs?.['data-status'] ?? 'published') !== status) problems.push(`${file}: sidebar publication marker differs from status; use npm run publish`);
      continue;
    }
    write(
      file,
      `---
title: ${yamlStr(a.title)}
description: ${yamlStr(a.scope)}
level: ${a.level}
status: planned
module: "${m.id}"
tableOfContents: false
pagefind: false
sidebar:
  order: ${a.index}
  attrs:
    data-status: planned
---
{/* Stub generated from syllabus/. Replace the body with the article and set status: published. */}
import ArticlePlan from '@components/ArticlePlan.astro';

<ArticlePlan module="${m.id}" slug="${a.slug}" />
`,
    );
    created++;
  }
}

// Warn about article files that are no longer in the syllabus (renamed or removed).
const known = new Set(modules.flatMap((m) => m.articles.map((a) => `${m.slug}/${a.slug}.mdx`)));
for (const m of modules) {
  const dir = join(docsDir, m.slug);
  for (const f of existsSync(dir) ? readdirSync(dir) : []) {
    if (f === 'index.mdx' || !f.endsWith('.mdx')) continue;
    if (!known.has(`${m.slug}/${f}`)) problems.push(`${m.slug}/${f} is not in the syllabus`);
  }
}

const data = {
  articleCount,
  parts: parts.map((p) => ({
    id: p.id,
    slug: p.slug,
    title: p.title,
    color: p.color,
    summary: p.summary,
    modules: p.modules.map((m) => m.id),
  })),
  modules: Object.fromEntries(
    modules.map((m) => [
      m.id,
      {
        ...m,
        articles: m.articles.map((a) => ({
          ...a,
          status: readStatus(join(docsDir, m.slug, `${a.slug}.mdx`)),
        })),
      },
    ]),
  ),
};

write(join(dataDir, 'syllabus.json'), JSON.stringify(data, null, 2) + '\n');

const published = Object.values(data.modules).flatMap((m) => m.articles).filter((a) => a.status === 'published').length;
console.log(
  `syllabus: ${parts.length} parts, ${modules.length} modules, ${articleCount} articles (${published} published)`,
);
console.log(`generated: ${created} new stubs, ${overviews} module overviews`);

for (const entry of readdirSync(docsDir,{withFileTypes:true})) {
  if (entry.isDirectory() && !modules.some(m=>m.slug===entry.name)) problems.push(`${entry.name}: orphan module directory`);
}
if (problems.length) throw new Error(problems.join('\n'));

// Validate the complete curriculum before changing any generated files.
for (const [file,text] of outputs) {
  mkdirSync(dirname(file), {recursive:true});
  writeFileSync(file,text);
}
