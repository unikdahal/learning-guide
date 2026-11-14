// Builds the Lok Sewa answer bank: one page listing every timed written question (with a model
// answer) across the NEC courses, grouped by Lok Sewa Computer Engineer syllabus unit.
// Runs as part of `npm run generate`, so the page can never drift from the lessons.
//   node scripts/answer-bank.mjs          write the page
//   node scripts/answer-bank.mjs --check  fail if the page is out of date
import { readFileSync, readdirSync, writeFileSync, existsSync } from 'node:fs';
import { join } from 'node:path';

const ROOT = new URL('..', import.meta.url).pathname;
const DOCS = join(ROOT, 'src/content/docs');
const OUT = join(DOCS, 'lok-sewa/computer-engineer/answer-bank.mdx');
const BASE = '/learning-guide';

const COURSES = ['electronics', 'digital', 'organization', 'networks', 'dsa', 'dbms', 'os', 'se', 'cpp'];

const UNITS = {
  1: 'Unit 1 · Computer networks',
  2: 'Unit 2 · Architecture, organisation and microprocessors',
  3: 'Unit 3 · Digital design',
  4: 'Unit 4 · Basic electrical and electronics',
  6: 'Unit 6 · Structured and object-oriented programming',
  7: 'Unit 7 · Data structures and algorithms',
  8: 'Unit 8 · Software engineering',
  9: 'Unit 9 · Database management systems',
  10: 'Unit 10 · Operating systems',
  15: 'Unit 15 · Emerging topics (cryptography and security)',
  16: 'Section D · Case study',
};

const unitFor = (course, file) => {
  if (course === 'digital' && /^1[1-4]-/.test(file)) return 2;
  if (course === 'networks' && file.startsWith('08-')) return 15;
  if (course === 'se' && file.startsWith('06-')) return 16;
  return { electronics: 4, digital: 3, organization: 2, networks: 1, dsa: 7, dbms: 9, os: 10, se: 8, cpp: 6 }[course];
};

// Questions reported from past Lok Sewa papers (file → note). Kept here so the bank can flag them.
const REPORTED = {
  'organization/01-architecture-basics': 'CE 2079',
  'networks/01-basics-models': 'CE 2079',
  'networks/04-ip-subnetting': 'CE 2078',
  'networks/06-transport': 'CE 2078',
  'networks/07-application': 'CO 2083',
  'networks/08-security': 'CE 2078',
  'dbms/03-normalisation': 'CE 2078',
  'dbms/04-transactions': 'CO 2083',
  'os/02-scheduling': 'CE 2078',
  'os/04-deadlock': 'CE 2078',
  'os/05-memory': 'CO 2083',
  'se/01-process-requirements': 'CE 2079',
  'se/03-design': 'CE 2078',
  'se/06-case-study': 'CE 2079',
};

// Lessons with several answers from different papers: flag by question prefix instead.
const REPORTED_Q = [
  ['Why choose OOP', 'CE 2079'],
  ['Polymorphism and inheritance', 'CE 2079'],
  ['Java access modifiers', 'CO 2083'],
  ['Role of language translators', 'CO 2083'],
];
const flag = (e) => REPORTED[`${e.course}/${e.slug}`] ?? REPORTED_Q.find(([p]) => e.q.startsWith(p))?.[1];

const frontTitle = (src) => (src.match(/^title:\s*"?(.*?)"?\s*$/m) || [])[1] ?? '';
const clean = (s) => s.replace(/\*\*/g, '').replace(/\s+/g, ' ').trim();

const entries = [];
for (const course of COURSES) {
  const dir = join(DOCS, 'nec', course);
  if (!existsSync(dir)) continue;
  for (const file of readdirSync(dir).filter((f) => f.endsWith('.mdx')).sort()) {
    const src = readFileSync(join(dir, file), 'utf8');
    const slug = file.replace(/\.mdx$/, '');
    const href = `${BASE}/nec/${course}/${slug}/`;
    const title = frontTitle(src);
    const lines = src.split('\n');
    lines.forEach((line, i) => {
      // "## Lok Sewa: write it (10 marks · 16 minutes)" followed by a **Question.** paragraph
      let m = line.match(/^## Lok Sewa: write it \((\d+) marks · (\d+) minutes\)/);
      if (m) {
        const q = lines.slice(i + 1, i + 6).find((l) => l.startsWith('**Question.**'));
        if (q) entries.push({ course, slug, href, title, marks: +m[1], mins: +m[2], q: clean(q.replace('**Question.**', '')) });
        return;
      }
      // "## 1. Why choose OOP …? (10 marks · 16 minutes)" — headed written answers
      m = line.match(/^## \d+\. (.+?) \((\d+) marks · (\d+) minutes\)/);
      if (m) entries.push({ course, slug, href: `${href}`, title, marks: +m[2], mins: +m[3], q: clean(m[1]) });
    });
    if (course === 'se' && slug === '06-case-study')
      entries.push({ course, slug, href, title, marks: 20, mins: 32, q: 'Online revenue payment system: feasibility, use case diagram and description, sequence diagram, ER design with normalised tables, architecture and testing' });
  }
}

const byUnit = new Map();
for (const e of entries) {
  const u = unitFor(e.course, e.slug + '.mdx');
  if (!byUnit.has(u)) byUnit.set(u, []);
  byUnit.get(u).push(e);
}

const total = entries.length;
const marks = entries.reduce((a, e) => a + e.marks, 0);
const reported = entries.filter((e) => flag(e)).length;

let out = `---
title: Lok Sewa answer bank
description: Every timed written question with a model answer across the NEC courses — ${total} answers grouped by Lok Sewa Computer Engineer unit, with reported past questions flagged.
sidebar:
  label: Answer bank
  order: 2
---
import { Aside } from '@astrojs/starlight/components';

{/* Generated by scripts/answer-bank.mjs from the course lessons — edit the lessons, not this page. */}

Every question below has a **timed model answer** on its lesson page (open the "Model answer" fold after writing yours). **${total} answers, ${marks} marks of practice**; ${reported} come from reported past Lok Sewa papers and are marked ★.

<Aside type="tip" title="How to use it">
Paper II is written: 3 hours, 100 marks, so about **1.6 minutes per mark** — 8 minutes for a 5-mark answer, 16 for a 10-mark answer. Pick a question, set a timer, write it by hand **without** opening the lesson, then compare with the model answer and mark yourself honestly (structure, definitions, diagram, example, conclusion). Do the ★ questions first, then work unit by unit with Section A and B units getting most of your time.
</Aside>

`;

for (const u of [1, 2, 3, 4, 6, 7, 8, 9, 10, 15, 16]) {
  const list = byUnit.get(u);
  if (!list) continue;
  out += `## ${UNITS[u]}\n\n| Question | Marks · minutes | Lesson |\n| --- | --- | --- |\n`;
  for (const e of list) {
    const star = flag(e) ? ` ★ (${flag(e)})` : '';
    out += `| ${e.q.replace(/\|/g, '\\|')}${star} | ${e.marks} · ${e.mins} | [${e.title.replace(/\|/g, '\\|')}](${e.href}) |\n`;
  }
  out += '\n';
}

out += `## Not here yet

- **Unit 5 (electronic communications), 11–14 (AI, theory of computation, compiler design, graphics) and the rest of unit 15** — Phase 2 (January–March), after the NEC exam.
- **Section D technical writing (16.1, 10 marks)** — weekly practice from January.
- **Paper I** — general awareness and reasoning; separate preparation.
`;

if (process.argv.includes('--check')) {
  const cur = existsSync(OUT) ? readFileSync(OUT, 'utf8') : '';
  if (cur !== out) {
    console.error('answer-bank.mdx is out of date — run node scripts/answer-bank.mjs');
    process.exit(1);
  }
} else {
  writeFileSync(OUT, out);
  console.log(`answer bank: ${total} answers, ${marks} marks, ${reported} reported`);
}
