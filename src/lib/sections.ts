// The guides hosted on this site. Each section owns a URL prefix, its own sidebar and its own
// header title, so it reads as a separate site while sharing the theme and components.
// Query Execution keeps the site root and every existing URL; other sections live under a prefix.

import sectionData from '../data/sections.json';

export type Section = {
  id: string;
  title: string;
  /** One line for the guides hub. */
  blurb: string;
  /** Path under the site base, with a trailing slash; also the section's landing page. '' is the root. */
  prefix: string;
};

// Order matters: the root section ('' prefix) comes last because it matches every path.
// scripts/generate.mjs reads the same file so section directories are not flagged as orphan modules.
export const SECTIONS: Section[] = sectionData;

/** The hub page that lists every guide. It belongs to no section. */
export const HUB_PATH = 'guides/';
export const HUB_TITLE = 'Learning Guide';

/** Find the section for a path relative to the site base (no leading slash). */
export function sectionFor(relativePath: string): Section {
  const p = relativePath.replace(/^\/+/, '');
  return SECTIONS.find((s) => p.startsWith(s.prefix))!;
}
