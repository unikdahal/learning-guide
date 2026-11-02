// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import { loadSyllabus } from './scripts/syllabus.mjs';

const { parts } = loadSyllabus();

// GitHub Pages serves project sites from /<repo>/. Override with SITE / BASE env vars
// if you move to a custom domain.
const site = process.env.SITE ?? 'https://unikdahal.github.io';
const base = process.env.BASE ?? '/learning-guide';

const sidebar = [
  {
    label: 'Start here',
    items: [
      { label: 'Roadmap', link: '/roadmap/' },
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
      customCss: ['./src/styles/theme.css'],
      head: [
        { tag: 'link', attrs: { rel: 'preconnect', href: 'https://fonts.googleapis.com' } },
        { tag: 'link', attrs: { rel: 'preconnect', href: 'https://fonts.gstatic.com', crossorigin: '' } },
        {
          tag: 'link',
          attrs: {
            rel: 'stylesheet',
            href: 'https://fonts.googleapis.com/css2?family=Bricolage+Grotesque:opsz,wght@12..96,400..800&family=JetBrains+Mono:wght@400;600&family=Source+Serif+4:ital,opsz,wght@0,8..60,400..700;1,8..60,400..700&display=swap',
          },
        },
      ],
      expressiveCode: {
        themes: ['github-dark-dimmed', 'github-light'],
        styleOverrides: { borderRadius: '6px', codeFontFamily: "'JetBrains Mono', ui-monospace, monospace" },
      },
      sidebar,
    }),
  ],
});
