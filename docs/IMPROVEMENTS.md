# Review implementation record

Implemented against main `325b1209c28b35b2c8033f6ef5a74d9aec93736c` (8 November 2026),
preserving the newer hardware/representation lessons added after the original review. The current
course has 49 published articles within a 324-article blueprint. This change repairs the current
foundation and supplies gates for expansion; it does not mark unwritten articles or capstone engines complete.

## Findings

| Finding | Result and evidence boundary |
| --- | --- |
| F01 — Correct the conditions under which UNION ALL is equivalent to UNION | Corrected the uniqueness conditions and added the disjoint-but-duplicated counterexample; SQL regression distinguishes UNION and UNION ALL. |
| F02 — Use NULL-safe row equality in the numbered-copy set-operation rewrite | Numbered-copy rewrites use NULL-safe equality for every field and ordinary equality for the copy number; composite nullable regression covers both subtraction and intersection. |
| F03 — Make grouping explanations account for HAVING and active filters | Grouping output and explanatory text derive from the combined filters, grouping and HAVING controls; empty/all-NULL SQL regressions and browser checks cover the counterexample. |
| F04 — Fix candidate-key minimality for empty and singleton relations | Candidate-key search tests the empty subset and handles empty, singleton and zero-degree relations; unit regressions included. |
| F05 — Validate persisted progress before returning it to page code | Versioned persisted state is validated field by field, restricted to published paths and migrated defensively; malformed/null/array and invalid numeric inputs have unit coverage. |
| F06 — Merge same-article progress monotonically across tabs | Same-article merge preserves maximum progress/completion and the most recently visited section. Web Locks serialize writes where supported; two-tab browser regression included. Older browsers use best-effort merging. |
| F07 — Make clearing reading history durable across open tabs | A durable reset generation invalidates old tabs and legacy migration; reset, reload and cross-tab behavior are covered in the browser. |
| F08 — Repair contrast at the token and component-state levels | Adjusted theme tokens, meaningful text opacity and selected-state colors. Browser checks cover main content of all published lessons in both themes, including the newer lessons added after the review. |
| F09 — Use a valid table structure for the domain-product diagram | Domain product uses native table elements with valid headers and cells. |
| F10 — Complete the keyboard and semantic contract for tabs | Scenario tabs have unique associations, roving focus, Arrow/Home/End handling and readable panels; keyboard regression included. New window-frame controls use native buttons inside list items. |
| F11 — Make overflowing code and diagrams keyboard reachable | Overflowing examples gain keyboard focus and region labels; grid children can shrink so charts resize within narrow viewports. Native table semantics are preserved. |
| F12 — Add a real static diagnostic gate and repair the existing error | Astro diagnostics are a required gate; shared syllabus interfaces replace untyped component assumptions. Fixed the existing collection/component errors and new boolean ARIA attributes. |
| F13 — Reject invalid and internally inconsistent dataset scales | Invalid, tiny and unrepresentable scales fail before output creation. Generation stages its files and records the version and row counts; invalid-scale tests check no output side effects. |
| F14 — Make example correctness executable rather than print-only | Added executable SQL oracles for NULL/composite bag matching, empty aggregates/HAVING, truth tables, pinned engine bugs, natural/outer joins, ties/keyset predicates and an expected rejection. Revenue examples assert exact keyed answers. These checks cover the reviewed failure modes; they are not a claim that every printed demonstration in the growing lab has a complete golden-output suite. |
| F15 — Extend manual release validation to cover the course’s executable evidence | Manual release requires Node tests, curriculum validation, Astro diagnostics, build/link/index/payload checks, browser accessibility/interaction checks, Rust formatting/tests and all-target Clippy. Pushes remain free of Actions runs. |
| F16 — Show the difference between the available course and the planned course | Available and planned counts are derived from metadata on the home page, routes and module views; roadmap defaults to published with explicit full-blueprint controls. |
| F17 — Expose publication status in search and navigation | Publication status drives sidebar/list labels, search exclusion and noindex for unfinished articles. |
| F18 — Make suggested routes honest about prerequisite closure | Route definitions are shared. Beginner route closes prerequisites; experienced routes state assumed knowledge and show publication coverage. |
| F19 — Reconcile the capstone’s all-22-TPC-H promise with its operator plan | docs/CAPSTONE.md defines three milestones and the all-22-query capability matrix, including subqueries, correlation, semi/anti joins, scalar aggregates, sort/limit and exact Decimal/date semantics. The syllabus now states these staged requirements and links the matrix from capstone stubs. Future implementation remains a curriculum milestone. |
| F20 — Validate the complete normalized syllabus contract | Normalized syllabus validation checks shape, required text, safe/reserved slugs, IDs, duplicates, levels and dependency cycles. Generation validates everything before writing outputs. |
| F21 — Define one authoritative publication and ordering workflow | Frontmatter owns publication; YAML owns titles/levels/module/order. The publication command synchronizes flags without replacing bodies; read-only checks report drift and orphans. |
| F22 — Publish reproducible benchmark records and strengthen answer checks | Benchmark recorder captures command, revision/dirty state, host/compiler/data/cache details and raw timing samples where supported. Exact revenue assertions added. Missing original raw samples and hardware metadata cannot be reconstructed; historical claims are labeled accordingly in the evidence policy. |
| F23 — Maintain versioned references and repair the observed missing link | DataFusion references are pinned to 55.2.0; repaired the observed Spark reference. Versioned reference review is part of contribution guidance. |
| F24 — Qualify broad semantic claims before readers generalize them | Qualified aggregate NULL behavior and state size, finite-width overflow, range joins, keyset pagination under mutation, rank ties, engine-specific UNIQUE semantics and NULL-sensitive anti-join rewrites. |
| F25 — Specify the generator’s data model and test deterministic guarantees | Documented logical repeatability and generator version; corrected trips fare cents and added deterministic row/fare/reference tests. Encoded bytes are not promised across library upgrades. |
| F26 — Document the limits of the tiny relational and bag engines | Documented the tiny engine’s supported values and preconditions; no implication of full SQL domains/coercions/NULL support. |
| F27 — Provide useful static states when JavaScript is unavailable | Core reviewed visuals have useful static explanations/worked states and hidden interactive-only controls without JavaScript. A browser check verifies the NULL worked example. New lesson diagrams still require author review for equivalent no-script depth; prose and measured tables remain available. |
| F28 — Handle animation cancellation and respect motion preferences consistently | View-transition rejections are handled; query-journey timers are cancelled when superseded/disconnected and scrolling honors reduced motion. Hero reduced-motion regression retained. |
| F29 — Keep reusable visuals independent of document-global identifiers | Reusable controls and SVG references use request-local unique identifiers. Routes share one definition rather than divergent component copies. |
| F30 — Add concise contribution, ownership, and licensing guidance | CONTRIBUTING.md defines authoring, ownership, validation and correction procedures. Licensing is explicitly an owner decision; no license or reuse grant was invented. |
| F31 — Reduce unnecessary dependency coupling before adding many lab crates | Pure relational checks can disable SQL dependencies; labkit data support is optional and unused coupling removed. Existing example defaults remain usable. |
| F32 — Define timing-helper behavior for degenerate samples | Timing summaries and parallelism define zero-duration behavior and reject empty summaries; arithmetic avoids median overflow. Unit tests cover degenerate inputs. |
| F33 — Make the natural-join result heading match the operation shown | Natural-join visual renders shared keys once with matching headings; browser and SQL oracles verify the output schema. |
| F34 — Set payload budgets before growing the static navigation further | Static checks enforce raw-byte HTML, script and font budgets plus links, anchors, duplicate IDs and project-base URLs. Budgets are explicit starting limits, not a performance score. |
| F35 — Resolve the access model for a publicly readable guide with a private lab repository | Setup and contribution instructions explain private-repository access and the authorization-error path. Publishing a public mirror, distributing an archive, changing visibility and setting branch protections remain owner decisions. |

## Remaining delivery milestones

- Recover original performance evidence only if the original author still has it. Otherwise rerun
  experiments with `scripts/capture-benchmark.py` and publish new dated records; keep historical
  observations distinguishable. Do not transfer this environment’s host details onto old tables.
- Before each new lesson is published, add answer checks, retrieval/transfer questions, a useful
  static visual state, keyboard/combined-control coverage and versioned primary references. Extend
  the regression suite when a newly authored query or engine feature needs a new oracle.
- Deliver the capstone milestones and test all 22 TPC-H queries against a reference engine before
  advertising full coverage. See [CAPSTONE.md](CAPSTONE.md) for prerequisites and acceptance criteria.
- The maintainer chooses licensing, public lab distribution, repository visibility and branch rules.
  The source remains private and no access or legal policy has been changed.

## Validation

`npm run verify` runs unit tests, read-only curriculum consistency, Astro checks, a production build
and generated-site checks. `npm run test:browser` runs functional regressions and main-content WCAG
checks in both themes for home, roadmap and every published lesson, then checks mobile overflow.
These automated checks do not replace screen-reader testing, editorial review or benchmarking.

From `lab/`, run `cargo fmt --all -- --check`,
`cargo test --workspace --lib --tests --locked` and
`cargo clippy --workspace --all-targets --locked -- -D warnings`. The manual deployment workflow runs
these Rust checks alongside website validation and only deploys after both jobs succeed.
