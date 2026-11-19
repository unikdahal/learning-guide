# Contributing to Query Execution

The YAML syllabus describes the intended course; MDX owns prose and publication status. Generated
module overviews and `src/data/syllabus.json` must match that contract. Existing article bodies are
never overwritten by generation. Keep explicit slugs when changing a title to preserve public URLs.

## Set up and verify

Use Node 22.23.4 or newer, Python 3 for static output checks, and the Rust toolchain in
`lab/rust-toolchain.toml`. Run `npm ci`, `npm run generate`, and `npm run verify`.
For browser checks, run `npx playwright install chromium` followed by `npm run test:browser`.

For the lab, run from `lab/`:

```sh
cargo test --locked -p m01-relational --no-default-features --lib # small pure-algebra check
cargo test --locked --workspace --lib --tests
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
```

The full DataFusion graph is large. Prefer `cargo check` during editing; build/run individual
examples rather than linking every example simultaneously on a small machine. Hardware and
performance experiments run separately, one at a time, in release mode. Do not turn noisy timing
ratios into CI pass criteria. Examples using the standard revenue fixture require generator scale 1.

## Write and publish an article

1. Edit the corresponding stub, following the website style guide. State prerequisites, exact
   input/output, supported types and NULL/duplicate/order assumptions, and any engine version.
2. Provide an executable example and correctness checks. Expected failures must be distinguished
   from successful queries; a printed verdict alone is not a regression check.
3. Test diagrams with combined controls, keyboard navigation, light/dark themes, 375px width,
   reduced motion, and JavaScript disabled. Keep rejected input values readable. Use `uniqueId`
   for repeated diagrams, and give every essential example a useful static state or explanation.
4. Include three to five retrieval/transfer questions with answers and primary references. Mark
   hypotheses separately from measurements. See `benchmarks/README.md` for evidence records.
5. Use `npm run publish -- 01 article-slug published` (or `draft`/`planned`). This updates the
   status, search flag and sidebar marker together, preserving the authored body. Run
   `npm run generate`, `npm run verify`, and browser checks before committing.

`npm run check:content` is read-only: it reports title/level/module/order drift, invalid publication
markers, stale generated data and orphan module directories. Repair metadata deliberately; do not
replace authored prose with generated stubs. Article descriptions may intentionally expand the
short YAML scope. Publication status drives labels, search exclusion and noindex for unfinished pages.

## Release without automatic build minutes

Pushing does not deploy or run Actions. Dispatch **Validate Rust lab** for changes to executable
claims, and check that its validated commit is the commit being released. Dispatch **Deploy to
GitHub Pages** when ready: it runs the Rust checks, website verification and browser checks before deployment.
The action revisions are pinned; update them deliberately with their upstream release notes.

## Access, reuse, and factual corrections

Reading a hosted site does not grant access to a private GitHub repository. If cloning gives an
authorization error, ask the maintainer for access through your normal GitHub account; an authorized
checkout can use `gh repo clone unikdahal/learning-guide`. Do not place access tokens in articles or URLs.
The maintainer must explicitly approve any public mirror or release distribution.

No license has been selected for the project's original content and code. This change does not grant
new reuse permissions or relicense contributions. The maintainer must choose and add appropriate
terms before inviting public reuse; third-party quotations and figures retain their own terms.

For a factual correction, include the article URL, engine/compiler version, minimal input/query,
expected result, actual result, and primary reference. For a visual issue, add browser, theme,
viewport, chosen controls, and whether it reproduces with keyboard or reduced motion.
