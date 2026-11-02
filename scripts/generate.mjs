// Generates content from syllabus/*.yaml:
//   - src/data/syllabus.json (used by the roadmap and article components)
//   - src/content/docs/<module>/index.mdx  (module overviews; regenerated unless edited by hand)
//   - src/content/docs/<module>/<article>.mdx (stubs; NEVER overwrites an existing article)
import { existsSync, mkdirSync, readFileSync, writeFileSync, readdirSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { loadSyllabus } from './syllabus.mjs';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const docsDir = join(root, 'src/content/docs');
const dataDir = join(root, 'src/data');

const { parts, modules, articleCount } = loadSyllabus();

const yamlStr = (s) => JSON.stringify(s); // JSON strings are valid YAML scalars

// Published status lives in each article's frontmatter, so read it back for the data file.
function readStatus(file) {
  if (!existsSync(file)) return 'planned';
  const m = readFileSync(file, 'utf8').match(/^status:\s*(\w+)/m);
  return m ? m[1] : 'published';
}

let created = 0;
let overviews = 0;

for (const m of modules) {
  const dir = join(docsDir, m.slug);
  mkdirSync(dir, { recursive: true });

  // Module overview: regenerate only while it still carries the generated marker.
  const indexFile = join(dir, 'index.mdx');
  const isGenerated = !existsSync(indexFile) || /^generated:\s*true/m.test(readFileSync(indexFile, 'utf8'));
  if (isGenerated) {
    writeFileSync(
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
    if (existsSync(file)) continue;
    writeFileSync(
      file,
      `---
title: ${yamlStr(a.title)}
description: ${yamlStr(a.scope)}
level: ${a.level}
status: planned
module: "${m.id}"
tableOfContents: false
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
  for (const f of readdirSync(dir)) {
    if (f === 'index.mdx' || !f.endsWith('.mdx')) continue;
    if (!known.has(`${m.slug}/${f}`)) console.warn(`warning: ${m.slug}/${f} is not in the syllabus`);
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

mkdirSync(dataDir, { recursive: true });
writeFileSync(join(dataDir, 'syllabus.json'), JSON.stringify(data, null, 2) + '\n');

const published = Object.values(data.modules).flatMap((m) => m.articles).filter((a) => a.status === 'published').length;
console.log(
  `syllabus: ${parts.length} parts, ${modules.length} modules, ${articleCount} articles (${published} published)`,
);
console.log(`generated: ${created} new stubs, ${overviews} module overviews`);
