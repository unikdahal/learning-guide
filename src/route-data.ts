// Splits one Starlight site into separate-feeling guides (src/data/sections.json):
// each page sees only its own section's sidebar, header title and title suffix,
// and prev/next links never cross from one guide into another.
import { defineRouteMiddleware } from '@astrojs/starlight/route-data';
import type { StarlightRouteData } from '@astrojs/starlight/route-data';
import { HUB_PATH, HUB_TITLE, sectionFor } from './lib/sections';

type Entry = StarlightRouteData['sidebar'][number];

const base = import.meta.env.BASE_URL.replace(/\/+$/, '') + '/';
const relative = (href: string) => (href.startsWith(base) ? href.slice(base.length) : href.replace(/^\/+/, ''));

/** Every link href inside a sidebar entry. */
function hrefs(entry: Entry): string[] {
  return entry.type === 'link' ? [entry.href] : entry.entries.flatMap(hrefs);
}

export const onRequest = defineRouteMiddleware((context) => {
  const route = context.locals.starlightRoute;
  const path = relative(context.url.pathname);
  const isHub = path === HUB_PATH;
  const section = sectionFor(path);
  const title = isHub ? HUB_TITLE : section.title;
  const inSection = (href: string) => sectionFor(relative(href)).id === section.id;

  // A top-level sidebar group belongs to the section that owns its first link.
  route.sidebar = route.sidebar.filter((entry) => {
    const first = hrefs(entry)[0];
    return first === undefined || inSection(first);
  });
  if (route.pagination.prev && !inSection(route.pagination.prev.href)) route.pagination.prev = undefined;
  if (route.pagination.next && !inSection(route.pagination.next.href)) route.pagination.next = undefined;

  // Starlight titles pages "<page> | <site title>" and sets og:site_name; swap in the section's name.
  const original = route.siteTitle;
  for (const tag of route.head) {
    if (tag.tag === 'title' && typeof tag.content === 'string' && tag.content.endsWith(original)) {
      tag.content = tag.content.slice(0, tag.content.length - original.length) + title;
    }
    if (tag.tag === 'meta' && tag.attrs?.property === 'og:site_name') tag.attrs.content = title;
  }
  route.siteTitle = title;
  route.siteTitleHref = base + (isHub ? '' : section.prefix);
});
