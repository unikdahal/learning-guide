# Paper I Part I: source and verification record

Checked: 2026-11-17. Scope: the new `05-paper-one-reasoning.mdx` lesson only.

## Primary source inspected

[PSC federal Computer Engineer syllabus, Paper I and examination scheme](https://psc.gov.np/site_uploads/course/20231110070747654dd6c388682.pdf), printed pages 1–2 of 6. This is the Nepal Miscellaneous Service, Gazetted Third Class, Computer Engineer open-competition syllabus. Page 1 prints the effective date as 2080/07/20 BS. The file was fetched successfully, text extracted by the web reader, and both pages rendered locally and visually read. Legacy-font extraction of page 1 alone is unsuitable evidence for its Nepali notes.

Downloaded PDF SHA-256:

```text
d0aac16b9e912d89dabdfddedbb4fdb28ad1e08613c7222a974ca0cb8dec2498
```

| Lesson claim | Precise primary location | Boundary |
| --- | --- | --- |
| Part I is 50 marks; awareness 25 and reasoning 25 | p. 2, Part I heading and §§1–2 | Awareness is not included in reasoning's 25 marks. |
| Logical 9, numerical 8, spatial 8; one mark per question | p. 2, §§2.1–2.3 | The 15 practice questions do not reproduce this distribution. |
| Paper I totals 100 marks, has 50 Part I and 50 Part II MCQs, and lasts 90 minutes | p. 1, examination scheme, first-paper row | One time cell spans the whole paper; no reasoning-specific allocation. |
| Wrong MCQ deducts 20% of its mark; unanswered earns no mark and no deduction | p. 1, note 3 | For these one-mark questions that is a 0.2-mark deduction. |
| Calculators prohibited in MCQ examinations | p. 1, note 4 | Practice examples can all be solved manually. |
| Laws, acts, rules and policies use versions in force three months before the exam | p. 1, note 7 | Learner must establish the actual examination date and official applicable versions. |
| Sixteen awareness checklist items, in official order | p. 2, §§1.1–1.16 | Condensed revision headings, not a collection of verified current answers. |
| Logical, numerical and spatial study-map topics | p. 2, §§2.1–2.3 | Teaching methods and examples are original, not official instructions. |

The page-1 scheme calls the first section general awareness and general reasoning; the page-2 Part I heading uses general awareness and general ability. The lesson refers to Part I and then the explicitly named reasoning subsection to avoid conflating these labels. No assertion is made that this PDF alone establishes another post's syllabus or rules for a later advertisement.

## Original material and coverage

All Q1–Q15, answer options, worked explanations, distractor diagnoses, drill timings, and SVGs were created for this lesson. None is represented as an official sample, recalled paper, past-year question, frequency estimate, or forecast.

| Area | Worked coverage | Other methods introduced |
| --- | --- | --- |
| Logical | Q1 set inclusion/conclusions/converse; Q2 alphanumeric series and coding; Q3 ranks; Q4 routes; Q5 ordered input/output | Verbal analogy, explicit classification properties, assertion/reason distinction, Venn interpretation |
| Numerical | Q6 percentage/profit; Q7 work rates; Q8 fractions/ratio/average/table checks; Q9 sufficiency; Q10 calendar; Q11 arithmetic sum/analogy | Units, denominator changes, weighted averages, uniqueness and counterexamples |
| Spatial | Q12 cube net; Q13 exact mirror; Q14 matrix/rotation analogy; Q15 folds and holes | Figure-series/completion rules, classification, embedded-image tracing, water-image distinction |

Spatial SVGs have responsive view boxes, `role="img"`, unique title/description IDs referenced by `aria-labelledby`, current-colour geometry, and full textual specifications. Neither requires colour discrimination. The net is four squares A–B–C–D in a row, with E above B and F below B. The mirror has exact source coordinates and a stated axis; image choices use exact coordinates rather than subtly different thumbnails. Q14 uses text arrows plus words; Q15 specifies folds and coordinates in text.

## Independent answer audit

An inline Python script checked every answer, using exact fractions for arithmetic, exhaustive Boolean models for Q1, and propagated 3D face frames for Q12. These checks passed:

| Question | Key | Independent result |
| --- | --- | --- |
| Q1 | B | Enumerated two-person set memberships satisfying premises; only B holds in every model; no auditor existence assumed. |
| Q2 | B | Fifth term I32; subsequent encoding J35. |
| Q3 | C | Original 18 people; new length 15 and front rank 4. |
| Q4 | D | Displacement (3,4), squared magnitude 25, positive east/north signs. |
| Q5 | B | Actual sort then swap produces [5,2,9]. |
| Q6 | C | Exact selling price 900 and profit-on-cost 25/2 percent. |
| Q7 | A | Combined rate 5/36; remainder 7/12; B needs 21/2 additional days. |
| Q8 | B | Correct 30, wrong 10, accuracy 3/4, ratio 3:1, session mean 15. |
| Q9 | C | Separate counterexamples show nonuniqueness; combined dimensions 14/3 and 28/3 give area 392/9. |
| Q10 | B | Python Gregorian dates independently give 60 elapsed days and Friday; the Monday starting weekday is supplied in the question. |
| Q11 | D | Direct summation of 1 through 7 equals 28 and agrees with n(n+1)/2. |
| Q12 | D | Six unique normals: B +z, D −z; C +x, A −x; E +y, F −y. Thus C is opposite A (option D). |
| Q13 | A | Applied (x,y) → (200−x,y) to all three original vertices. |
| Q14 | D | In page coordinates (y downward), clockwise map (x,y) → (−y,x) sends up to right and down to left. |
| Q15 | B | Successive reflected sets give exactly (6,5), (2,5), (6,3), (2,3). |

The source of a distractor was also manually checked against each final option. Calendar assumptions are Gregorian; no Gregorian-to-BS conversion or BS month length is claimed. The work-rate example assumes constant independent rates. The data-sufficiency question permits positive noninteger lengths. Cube-face letters are labels, not a claimed standard numbered-die convention.

## Content validation

- Direct compilation with installed `@mdx-js/mdx` passed after stripping YAML frontmatter. A discovered `<details><summary>` paragraph-boundary issue was fixed by placing the opening tags on separate lines; compilation was rerun successfully.
- Python XML parsing passed for both inline SVGs; referenced accessible title/description IDs exist.
- Structural checks passed: 15 numbered worked questions, 15 balanced answer blocks, 16 checklist rows. Final lesson length: 325 lines.
- No full build, generator, sidebar/config edit, commit, or subagent was used. Only the two assigned new repository files were written by this task. Other work already present in the shared checkout was left alone.

## Awareness source-filling limits and remaining gaps

The awareness table recommends primary publishers by subject. Only the PSC syllabus was fetched for this task; no claim is made to have verified the current national plan, officeholders, statistics, legislation text, agency charter, or current events. Learners must fill dated cards from specific official documents, including page/table/section and check date. A publisher homepage is a starting point, not a supporting citation.

All sixteen official headings have a checklist row. Follow-up text preserves the detailed civil-service and management scopes that would otherwise disappear in a short heading. Modern-history prompts retain people and social/cultural/economic conditions; the official §1.4 remains the scope authority.

This compact lesson does not provide a deep drill bank for every verbal subtype, assertion/reason question format, embedded-image puzzle, figure series/classification, or cutting variant. It does not replace all 25 awareness marks with a current-facts notebook. Those are explicit next-stage practice/source-filling needs, not evidence of absent syllabus topics. Parent integration with the separate question bank and revision studio is outside these two files; no dependency on an unfinished integration was introduced.
