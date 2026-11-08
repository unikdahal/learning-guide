import { defineCollection } from 'astro:content';
import { z } from 'astro/zod';
import { docsLoader } from '@astrojs/starlight/loaders';
import { docsSchema } from '@astrojs/starlight/schema';

export const collections = {
  docs: defineCollection({
    loader: docsLoader(),
    schema: (context) => docsSchema({
      extend: z.object({
        // B beginner, I intermediate, A advanced, E expert
        level: z.enum(['B', 'I', 'A', 'E']).optional(),
        status: z.enum(['planned', 'draft', 'published']).optional(),
        module: z.string().optional(),
        generated: z.boolean().optional(),
      }),
    })(context).transform(entry => {
      if (!entry.status) return entry;
      const status=entry.status;
      const attrs={...entry.sidebar.attrs,'data-status':status};
      return {...entry,
        pagefind:status==='published' && entry.pagefind,
        sidebar:{...entry.sidebar,attrs},
        head:status==='published' ? entry.head : [...entry.head,{tag:'meta' as const,attrs:{name:'robots',content:'noindex'}}],
      };
    }),
  }),
};
