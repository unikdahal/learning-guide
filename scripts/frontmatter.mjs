import { parse, parseDocument } from 'yaml';
export function frontmatter(text, name = 'article') {
  const match = text.match(/^---\r?\n([\s\S]*?)\r?\n---(?:\r?\n|$)/);
  if (!match) throw new Error(`${name}: missing frontmatter`);
  const data = parse(match[1]);
  if (!data || typeof data !== 'object' || Array.isArray(data)) throw new Error(`${name}: invalid frontmatter`);
  return {data, header:match[1], body:text.slice(match[0].length)};
}
export function articleStatus(text, name) {
  const status = frontmatter(text,name).data.status;
  if (!['planned','draft','published'].includes(status)) throw new Error(`${name}: explicit status must be planned, draft or published`);
  return status;
}
export function setPublication(text, status) {
  if (!['planned','draft','published'].includes(status)) throw new Error(`Invalid publication status ${status}`);
  const {header,body} = frontmatter(text);
  const doc = parseDocument(header);doc.set('status',status);
  if (status === 'published') doc.deleteIn(['sidebar','attrs','data-status']);
  else doc.setIn(['sidebar','attrs','data-status'],status);
  doc.set('pagefind',status === 'published');
  return `---\n${doc.toString()}---\n${body}`;
}
