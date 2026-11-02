import { defineCollection } from 'astro:content';
import { z } from 'astro/zod';
import { docsLoader } from '@astrojs/starlight/loaders';
import { docsSchema } from '@astrojs/starlight/schema';

export const collections = {
  docs: defineCollection({
    loader: docsLoader(),
    schema: docsSchema({
      extend: z.object({
        // B beginner, I intermediate, A advanced, E expert
        level: z.enum(['B', 'I', 'A', 'E']).optional(),
        status: z.enum(['planned', 'draft', 'published']).optional(),
        module: z.string().optional(),
        generated: z.boolean().optional(),
      }),
    }),
  }),
};
