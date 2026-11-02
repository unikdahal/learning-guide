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
      customCss: [
        '@fontsource-variable/bricolage-grotesque/opsz.css',
        '@fontsource-variable/source-serif-4/opsz.css',
        '@fontsource-variable/source-serif-4/opsz-italic.css',
        '@fontsource-variable/jetbrains-mono/wght.css',
        './src/styles/theme.css',
      ],
      expressiveCode: {
        themes: ['github-dark-dimmed', 'github-light'],
        styleOverrides: { borderRadius: '6px', codeFontFamily: "'JetBrains Mono Variable', ui-monospace, monospace", codeFontSize: '0.875rem' },
      },
      sidebar,
      components: { Footer: './src/components/overrides/Footer.astro' },
    }),
  ],
});
