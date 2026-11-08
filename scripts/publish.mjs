// Usage: npm run publish -- 01 article-slug [planned|draft|published]
import {readFileSync,writeFileSync} from 'node:fs';
import {join} from 'node:path';
import {loadSyllabus} from './syllabus.mjs';
import {setPublication} from './frontmatter.mjs';
const [id,slug,status='published']=process.argv.slice(2);
const m=loadSyllabus().modules.find(m=>m.id===id);
if (!m?.articles.some(a=>a.slug===slug)) throw new Error('Usage: npm run publish -- <module-id> <article-slug> [planned|draft|published]');
const file=join(import.meta.dirname,'../src/content/docs',m.slug,slug+'.mdx');
writeFileSync(file,setPublication(readFileSync(file,'utf8'),status));
console.log(`Set ${m.id}/${slug} to ${status}. Run npm run generate and npm run verify before committing.`);
