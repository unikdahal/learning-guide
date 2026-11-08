# Query Execution

A deep, visual guide to how query engines execute queries, from relational algebra to vectorized
joins, shuffles and lakehouse pruning. A blueprint of 324 articles across 7 parts and 22 modules,
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
| `src/scripts/progress.ts` | Reading progress, kept in the reader's browser (localStorage) |
| `lab/` | The Rust lab workspace |

## Lab

`lab/` is a Rust workspace with runnable examples for the published foundations, hardware and
representation lessons: one crate per implemented module, plus `labkit` for shared data generation and timing. See the article
[Setting up the Rust lab](https://unikdahal.github.io/learning-guide/orientation/setting-up-the-rust-lab/).

```sh
cd lab
cargo run --release -p labkit --bin gen-data
cargo run --release -p m00-orientation --example query_stages
```

## Develop

```sh
npm ci
npm run dev      # regenerates from the syllabus, then serves at localhost:4321/learning-guide/
npm run build    # production build into dist/
```

## Write an article

1. Open its stub in `src/content/docs/<module>/<article>.mdx`.
2. Replace the body with the article. Follow the structure and visual rules in the
   [style guide](https://unikdahal.github.io/learning-guide/style-guide/).
3. Run `npm run publish -- <module-id> <article-slug> published`, then `npm run generate`
   and the quality checks. The command updates publication flags together without changing the body.

The generator never overwrites an existing article file.

## Add an article or module

Edit the YAML in `syllabus/` and run `npm run generate`. Order in the YAML is reading order.
See `syllabus/README.md` for the fields.

## Deploy

Deploys are manual, so pushing to `main` uses no Actions minutes.

1. Once: in **Settings → Pages**, set **Source** to **GitHub Actions**.
2. Each time you want to publish: **Actions → Deploy to GitHub Pages → Run workflow**.

Deployment runs the Rust, site and browser checks first; the first Rust build can take several minutes.

## Quality gates and publication

Run `npm run verify` for unit tests, curriculum consistency, static diagnostics, the production build,
internal links/anchors, indexing policy and raw-byte budgets. Run `npm run test:browser` after installing
Chromium with `npx playwright install chromium`. See [CONTRIBUTING.md](CONTRIBUTING.md) for the manual
release workflow, publication command, Rust checks and source-access requirements.

The full syllabus is a blueprint; the site derives available/planned counts from article metadata.
Planned/draft pages remain reachable in the roadmap but are excluded from reading-oriented search
and carry noindex. See [capstone acceptance](docs/CAPSTONE.md), [benchmark evidence](benchmarks/README.md)
and [review implementation status](docs/IMPROVEMENTS.md) for remaining milestones and owner decisions.
