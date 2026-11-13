// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import { loadSyllabus } from './scripts/syllabus.mjs';

const { parts } = loadSyllabus();

// GitHub Pages serves project sites from /<repo>/. Override with SITE / BASE env vars
// if you move to a custom domain.
const site = process.env.SITE ?? 'https://unikdahal.github.io';
const base = process.env.BASE ?? '/learning-guide';

// Query Execution: the generated curriculum, at the site root.
const queryExecutionSidebar = [
  {
    label: 'Start here',
    items: [
      { label: 'Roadmap', link: '/roadmap/' },
      { label: 'Numbers from the lab', link: '/lab-numbers/' },
      { label: 'Visual style guide', link: '/style-guide/' },
    ],
  },
  ...parts.map((part) => ({
    label: `Part ${part.id}: ${part.title}`,
    collapsed: part.id > 1,
    items: part.modules.map((m) => ({
      label: `${m.id} ${m.title}`,
      collapsed: true,
      items: [{ autogenerate: { directory: m.slug } }],
    })),
  })),
];

// Other guides, each under its own prefix. src/route-data.ts shows each page only its own guide's
// groups, so the sidebars never mix.
const necSidebar = [
  {
    label: 'NEC Licence',
    items: [
      { label: 'Overview', link: '/nec/' },
      { label: 'Exam format and dates', link: '/nec/exam/' },
      { label: 'Syllabus and priorities', link: '/nec/syllabus/' },
      { label: 'Study plan to 1 Jan', link: '/nec/roadmap/' },
    ],
  },
  { label: 'Electronics from zero', items: [{ autogenerate: { directory: 'nec/electronics' } }] },
  { label: 'Chapters', items: [{ autogenerate: { directory: 'nec/chapters' } }] },
];

const lokSewaSidebar = [
  {
    label: 'Lok Sewa',
    items: [
      { label: 'Overview', link: '/lok-sewa/' },
      { label: 'Roadmap', link: '/lok-sewa/roadmap/' },
    ],
  },
  { label: 'Computer Engineer', items: [{ autogenerate: { directory: 'lok-sewa/computer-engineer' } }] },
  { label: 'Computer Officer', items: [{ autogenerate: { directory: 'lok-sewa/computer-officer' } }] },
];

const sidebar = [...queryExecutionSidebar, ...necSidebar, ...lokSewaSidebar];

export default defineConfig({
  site,
  base,
  trailingSlash: 'always',
  integrations: [
    starlight({
      title: 'Query Execution',
      description:
        'A deep, visual guide to how query engines execute queries, from first principles to engine internals.',
      logo: { src: './src/assets/logo.svg', replacesTitle: false },
      favicon: '/favicon.svg',
      social: [
        { icon: 'github', label: 'GitHub', href: 'https://github.com/unikdahal/learning-guide' },
      ],
      editLink: { baseUrl: 'https://github.com/unikdahal/learning-guide/edit/main/' },
      lastUpdated: true,
      customCss: [
        '@fontsource-variable/bricolage-grotesque/opsz.css',
        '@fontsource-variable/source-serif-4/opsz.css',
        '@fontsource-variable/source-serif-4/opsz-italic.css',
        '@fontsource-variable/jetbrains-mono/wght.css',
        './src/styles/theme.css',
        './src/styles/lab-visual.css',
      ],
      expressiveCode: {
        themes: ['github-dark-dimmed', 'github-light'],
        styleOverrides: { borderRadius: '6px', codeFontFamily: "'JetBrains Mono Variable', ui-monospace, monospace", codeFontSize: '0.875rem' },
      },
      sidebar,
      components: {
        Footer: './src/components/overrides/Footer.astro',
        SocialIcons: './src/components/overrides/SocialIcons.astro',
      },
      routeMiddleware: './src/route-data.ts',
    }),
  ],
});
