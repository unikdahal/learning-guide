"""Deterministic output checks: links, anchors, IDs, publication indexing, and byte budgets."""
from html.parser import HTMLParser
from pathlib import Path
from urllib.parse import urljoin,urlsplit,unquote
import json,os
root=Path(__file__).resolve().parents[2]
dist=root/'dist';base=os.environ.get('BASE','/learning-guide').rstrip('/')+'/'
site=os.environ.get('SITE','https://unikdahal.github.io');host=urlsplit(site).netloc
class Page(HTMLParser):
    def __init__(self,text):
        super().__init__();self.ids=set();self.duplicates=[];self.links=[];self.noindex=False;self.search_body=False;self.feed(text)
    def handle_starttag(self,tag,attrs):
        a=dict(attrs)
        if 'data-pagefind-body' in a:self.search_body=True
        if 'id' in a:
            if a['id'] in self.ids:self.duplicates.append(a['id'])
            self.ids.add(a['id'])
        if tag=='a' and a.get('href'):self.links.append(a['href'])
        if tag=='meta' and a.get('name')=='robots' and 'noindex' in a.get('content',''):self.noindex=True
pages={};errors=[]
for file in dist.rglob('*.html'):
    path=base+file.relative_to(dist).as_posix().removesuffix('index.html');pages[path]=Page(file.read_text())
    limit=550_000 if path==base+'roadmap/' else 400_000
    if file.stat().st_size>limit:errors.append(f'{path}: HTML exceeds {limit} raw bytes; investigate before raising budget')
for path,page in pages.items():
    for id in page.duplicates:errors.append(f'{path}: duplicate id {id}')
    for href in page.links:
        u=urlsplit(urljoin(site+path,href))
        if u.scheme not in ('http','https') or u.netloc!=host:continue
        if not u.path.startswith(base):errors.append(f'{path}: internal link outside base: {href}');continue
        target=dist/unquote(u.path[len(base):]);target=target/'index.html' if u.path.endswith('/') else target
        if not target.exists():errors.append(f'{path}: missing {href}')
        elif u.fragment and u.path in pages and unquote(u.fragment) not in pages[u.path].ids:errors.append(f'{path}: missing anchor {href}')
data=json.loads((root/'src/data/syllabus.json').read_text())
for m in data['modules'].values():
    for a in m['articles']:
        page=pages.get(base+a['path']+'/')
        if not page:errors.append(f"Missing article {a['path']}")
        else:
            if a['status']!='published' and not page.noindex:errors.append(f"Unpublished article lacks noindex: {a['path']}")
            if a['status']!='published' and page.search_body:errors.append(f"Unpublished article is searchable: {a['path']}")
fonts=sum(f.stat().st_size for f in dist.rglob('*.woff2'))
js=sum(f.stat().st_size for f in dist.rglob('*.js'))
if fonts>1_500_000:errors.append('Emitted font assets exceed 1.5 MB')
if js>1_500_000:errors.append('Emitted JavaScript exceeds 1.5 MB')
if errors:raise SystemExit('\n'.join(errors))
print(f'{len(pages)} pages: links, anchors, unique IDs, unpublished indexing and byte budgets passed. Fonts={fonts}, JavaScript={js} raw bytes.')
