# Query Execution

A deep, visual guide to how query engines execute queries, from relational algebra to vectorized
joins, shuffles and lakehouse pruning. 324 planned articles across 7 parts and 22 modules,
from beginner to expert, with Rust for all code.

Live site: https://unikdahal.github.io/learning-guide/

## Structure

| Path | What it is |
| --- | --- |
| `syllabus/*.yaml` | The source of truth: parts, modules and every article with level, scope and planned visual |
| `scripts/generate.mjs` | Turns the syllabus into module overviews, article stubs and `src/data/syllabus.json` |
| `src/content/docs/<module>/` | One `.mdx` file per article |
| `src/components/` | Landing hero, roadmap, module and article components, `Figure` and `Stepper` for visuals |
| `src/styles/theme.css` | Theme tokens and the SVG diagram kit |

## Develop

```sh
npm install
npm run dev      # regenerates from the syllabus, then serves at localhost:4321/learning-guide/
npm run build    # production build into dist/
```

## Write an article

1. Open its stub in `src/content/docs/<module>/<article>.mdx`.
2. Replace the body with the article. Follow the structure and visual rules in the
   [style guide](https://unikdahal.github.io/learning-guide/style-guide/).
3. Change `status: planned` to `status: published`, and remove the `data-status: planned`
   attrs line so the sidebar shows it as written.

The generator never overwrites an existing article file.

## Add an article or module

Edit the YAML in `syllabus/` and run `npm run generate`. Order in the YAML is reading order.
See `syllabus/README.md` for the fields.

## Deploy

Pushing to `main` builds and deploys through GitHub Actions. In the repository settings,
under **Pages**, set the source to **GitHub Actions** once.
