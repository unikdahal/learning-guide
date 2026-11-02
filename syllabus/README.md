# Syllabus source

Every module and article in the guide is defined here, one YAML file per Part.
`npm run generate` turns these files into the sidebar, the roadmap page and a stub
page for every article that hasn't been written yet. Written articles are never
overwritten.

## Article fields

| Field | Meaning |
| --- | --- |
| `t` | Title |
| `l` | Level: `B` beginner, `I` intermediate, `A` advanced, `E` expert |
| `d` | Scope: what the article covers, in one or two sentences |
| `v` | Planned visual (optional) |
| `slug` | Optional override; otherwise derived from the title |

## Writing an article

1. Find its stub in `src/content/docs/<module>/`.
2. Replace the stub body with the article, and change `status: planned` to
   `status: published` in the frontmatter.
3. Run `npm run dev` to preview.

To add an article, append it to the module's `articles` list and run
`npm run generate`. Article order in the YAML is the reading order.
