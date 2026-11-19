# Lok Sewa Computer Engineer / Computer Officer: evidence log

Checked on 2026-11-17 (2083-08-01 BS, approx.). This file records what each claim in
`src/content/docs/lok-sewa/preparation/` rests on, so that every exam fact can be traced and
re-checked. Tiers used everywhere:

| Tier | Meaning | May be used for |
| --- | --- | --- |
| **O** Official | Published by PSC (psc.gov.np) or NEC (nec.gov.np) and read directly | marks, timing, rules, syllabus wording, official sample questions |
| **O−** Official negative | An official index searched and found empty | only a dated statement that no match was found in that index; never proof of global absence |
| **T** Third-party transcription | Typed copy on a non-official site, no scan, no provenance | practice, topic-frequency hints; never as "the" question text |
| **R** Repo-reported | Appears in this repo's earlier pages ("reported CO 2083", from the user's draft); no source found | flagged as unverified recall only |
| **P** Original practice | Written for this guide | practice only; labelled per question |

## 1. Official syllabus PDFs (tier O)

The PSC course index (`GET https://psc.gov.np/front/course?page=1&title=कम्प्युटर`) lists exactly one
course each for the two posts under Nepal Miscellaneous Service, Gazetted Third (रा०प० तृतीय):

| PSC course id | Designation (designation_id) | Files |
| --- | --- | --- |
| 569 | कम्प्युटर इञ्जिनियर (200) | Paper I + scheme: `https://psc.gov.np/site_uploads/course/20231110070747654dd6c388682.pdf` (6 pp.) · Paper II: `https://psc.gov.np/site_uploads/course/20231110070747654dd6c3886821.pdf` (6 pp.) |
| 571 | कम्प्युटर अधिकृत (135) | Paper I + scheme: `https://psc.gov.np/site_uploads/course/20231110070338654dd5cad7373.pdf` (7 pp.) · Paper II + samples: `https://psc.gov.np/site_uploads/course/20231110070338654dd5cad73731.pdf` (7 pp.) |

All four downloaded with HTTP 200 on 2026-11-17. Text is in legacy Preeti encoding, so page 1 of the
CE scheme and pp. 5–6 of CO Paper II were rendered to images and read visually.

**Link fix for the parent:** `src/content/docs/lok-sewa/computer-engineer/index.mdx` "Sources" links
`https://psc.gov.np/uploads/course/20231110070747654dd6c3886821.pdf`; the path the PSC API returns is
`/site_uploads/course/…` (the Aside on the same page already uses `site_uploads`).

### 1.1 Computer Engineer scheme (CE Paper I PDF, p. 1, read from the rendered image)

- Stage 1 written 200; stage 2 group test 10 + interview 30.
- Paper I General Subject: 100 marks, pass 40, objective MCQ. Part I general awareness & reasoning
  50 Q × 1; Part II general technical 50 Q × 1. One time cell for the whole paper: **1 h 30 min**.
- Paper II Technical Subject: 100 marks, pass 40, subjective, **2 Q × 5 + 7 Q × 10 + 1 Q × 20**, **3 h**.
- Group test: group discussion, 10 marks, 30 min. Interview: board interview, 30 marks.
- Notes (द्रष्टव्य): (1) medium Nepali or English or both; (2) papers held separately;
  (3) **20% of the mark deducted per wrong MCQ answer**, no deduction for unanswered;
  (4) **no calculator** in MCQ papers; (5) a subjective question may be one long question, two or more
  parts, or two or more short notes; (6) **separate answer book per section** of Paper II;
  (7) laws, acts, rules and policies are examined **as in force 3 months before the exam date**;
  (8) only stage-1 qualifiers sit stage 2; (9) earlier syllabus repealed;
  (10) **effective date 2080/07/20 BS**.
- Paper I Part I sub-weights: general awareness 25 × 1; reasoning 25 × 1 = logical 9, numerical 8,
  spatial 8.
- Paper I Part II weights: Section A 34%, B 36%, C 30% (units 1–15, same unit list as Paper II A–C).
- Paper II weights: Section A 25%, B 25%, C 20%, D 30%. Section D = 16.1 technical writing (1 Q × 10)
  + 16.2 case study (1 Q × 20). Wording of 16.1/16.2 quoted in the workshop page.
- Group discussion (last page): leaderless group discussion, each candidate also gives an individual
  presentation within the set time; panel of 3 (PSC chair/member as chair, a psychologist, one
  expert). Sample topics listed: energy crisis (ऊर्जा संकट), poverty alleviation (गरिबी निवारण),
  health insurance (स्वास्थ्य बीमा), food security (खाद्य सुरक्षा), brain drain (प्रतिभा पलायन); only
  one topic is given.

Inference (not stated by PSC): Paper II has 10 questions; Section D uses one 10-marker and the
20-marker, so A+B+C = 70 = 2 × 5 + 6 × 10. How those split across A (25), B (25) and C (20) is not
stated; 25 = 5 + 10 + 10 or 5 + 2×10 with sub-parts are both consistent. Pages say "not specified".

### 1.2 Computer Officer scheme (CO Paper I PDF, pp. 1–2)

- Stage 1 written 200; stage 2 **practical test 50** + group test 10 + interview 30 (= 90).
- Paper I identical in format to CE (50 + 50 MCQ, 100 marks, pass 40, 1 h 30 min).
- Paper II: 100 marks, pass 40, subjective, **10 Q × 10**, 3 h.
- Practical: 50 marks, pass 25, **1 h 30 min**, **5 Q × 10**. Question count by unit:
  Operating Systems 1, Database Management System and Design 2, Programming Language 1, Networking 1.
- Note 10: final merit list built on stage 1 + stage 2 totals, but **only candidates who pass the
  practical** are placed on it.
- Same notes on medium, 20% negative marking, calculator ban, sections, 3-month law rule.
  Effective date 2080/07/20.
- Paper I Part II unit weights (all printed in the official PDF, sum 100): Computer fundamentals 10,
  DSA 8, SAD 10, OS 10, DBMS 14, Programming 8, Networking 10, Architecture 4, Compiler 2,
  E-commerce 4, MIS & web engineering 10, IT in Nepal 10.
- Paper II sections: A 30 marks (computer fundamentals, DSA, programming language); B 20 (SAD);
  C 20 (OS, DBMS); D 30 (networking, architecture, compiler, e-commerce, MIS & web, IT in Nepal).
  The existing CO page marks some weights "†/from your draft" — all are now confirmed.

### 1.3 Official sample questions (CO Paper II PDF, pp. 5–6) — "sample", not past paper

Subjective samples (विषयगत नमूना प्रश्नहरु):
1. Merge operation between two heap trees — H1 drawn as "a max heap": 9; children 4, 6; 4's children
   3, 5. H2 drawn as "a min heap": 1; children 19, 80; 19's children 9, 9; 80's child 96.
   **Both drawings violate their heap property** (5 > 4 under a max heap; 9 < 19 under a min heap).
2. Preorder u1 u2 u3 u4 u10 u8 u5 u9 u6 u11 u7; postorder u4 u10 u3 u8 u2 u9 u11 u7 u6 u5 u1;
   "obtain a binary tree".
3. Java nested do-while to print a 3×3 multiplication table.
4. "Using C program draw result" — two columns 1..4 and 2..5.
5. Suppliers/Parts/Catalog: names of suppliers who supply a red part, in relational algebra and SQL
   (solution printed in the PDF).
6. Develop top-level DFD from a context diagram (Budget process; entities Department, Management,
   Suppliers; flows spending request, rejected request, delivery advice, budget allocation, request
   for special approval, response to special approval, spending summaries, part order, supplier
   delivery advice).

Practical samples (प्रयोगात्मक परीक्षाका नमूना प्रश्नहरु):
1. Get IP addresses of networked computers; (a) give privileged share only to database
   administration; (b) are Node1 192.255.16.8 and Node2 192.225.14.8 with mask 255.255.255.0 on the
   local or a remote network?
2. Create tables Student(Sid, Sname, subject, level), Teacher(Tid, Tname, subject), Course(Cid,
   Cnumber, Tname, level) in SQL; insert 10 rows; delete the first two.
3. C++: given string "123456789", print the pyramid 1 / 232 / 34543 / 4567654 / 567898765.

The CE syllabus contains no sample questions.

## 2. Official question bank (tier O−)

`GET https://psc.gov.np/front/questionbank/?page=1&per_page=100` (the API behind
psc.gov.np/category/question-bank) returned **89 entries** on 2026-11-17, all uploaded from
2026-10-08 onward, covering joint-secretary, under-secretary, section-officer, nayab subba and kharidar
papers (administration, justice, foreign, audit, parliament, plus a few gazetted-first technical
groups). Filtering by `designation_id` 200 (CE), 336 (CE, विविध), 955, 135 (CO), 275 (CO, विविध)
returned 0; `search=कम्प्युटर`/`computer` returned 0. **No CE or CO question paper is published by
PSC** in that index as of this date. Re-check before the exam; the index is new and growing.

## 3. NEC (tier O)

- NEC Computer Engineering model question:
  `https://nec.gov.np/uploads/Model%20Question%20Registration/1763714765897_Model%20question%20of%20Computer%20Engineering.pdf`
  (already cited in `nec/syllabus.mdx`). Not re-read here.

## 4. Third-party transcriptions (tier T)

| Label used in pages | URL | What it is |
| --- | --- | --- |
| CE 2078 (transcription) | https://nextgenacademyyy.blogspot.com/p/subject-computer-engineer-subjective.html | Typed "Computer Engineer Subjective 2078", 10 questions, sections A 25/B 25/C 20/D 30. No scan, no date, no ad number, no statement of origin. |
| CE 2079 (transcription) | https://nextgenacademyyy.blogspot.com/p/computer-engineer-subjective-2079.html | Typed "Computer Engineer Subjective 2079", 10 questions, same section totals; Q3 and Q9 have no marks printed. Same caveats. |

Both pages were fetched with curl and the post text extracted on 2026-11-17 (the repo's earlier
"reported CE 2078/2079" items match these texts).

Caveats recorded:
- Both sittings (2078, 2079 BS) **predate the current syllabus (effective 2080/07/20)**, so they were
  set under the earlier, repealed syllabus. Topic hints still useful; section placement may differ.
- Internal oddities: 2078 puts pseudo-random numbers / gap test (current unit 15.1, Section C) in
  Section A; 2079 puts knowledge representation (AI, current Section C) in Section A, and asks
  "OOP over structured" twice (Q4 and Q6). Either the older syllabus sectioned differently or the
  transcription is imperfect; cannot tell which.
- **Discrepancy with the repo:** the 2078 transcription's host is **200.168.10.177**; the NEC page
  `nec/networks/04-ip-subnetting.mdx` works the question with host **.75**. The preparation pages use
  .177 and say so. (Not editing the NEC page — outside ownership; flag to parent.)

## 5. Repo-reported items without a source (tier R)

"CO 2083" items in NEC pages (JAD roles; use case for online shopping; DNS and DHCP; tree traversals;
virtual memory/paging vs segmentation; DBMS concurrency control; Java access modifiers; role of
language translators; CPU and its components). Searched 2026-11-17; no PSC copy (question bank
empty for CO) and no third-party copy found. All of these topics are in the official CO syllabus, so
they are good practice either way, but they are **not** cited as past questions.

## 6. IT in Nepal (CO unit 12) — what is and is not verified

Not verified in this session from lawcommission.gov.np / mocit.gov.np: exact current titles and
years of the IT policy, Electronic Transactions Act and Copyright Act, and any amendments. The CO page
gives the syllabus headings only and asks the reader to pull the current texts, because the official
note 7 makes the law as in force 3 months before the exam the examinable version.

## 7. Worked-example verification

Numerical and code examples in the pages were checked by script on 2026-11-17 (Python `ipaddress`
for subnetting; Python for FIRST/FOLLOW, LL(1) table, LCG and gap test, Shannon and AM power,
tree reconstruction, heap build; g++/javac/gcc for the code samples where the toolchain was present).
See section 8 for results.

## 8. Script results (2026-11-17)

| Example | Result |
| --- | --- |
| 200.168.10.0/24 → 7 subnets, host .177 | /27, mask 255.255.255.224; host in 200.168.10.160/27 (6th subnet), broadcast .191, hosts .161–.190 |
| CO practical P1 local/remote | 192.255.16.0/24 vs 192.225.14.0/24 → different networks (remote) |
| CO sample S2 tree | unique full binary tree: u1(u2(u3(u4,u10),u8), u5(u9,u6(u11,u7))); re-traversal matches both sequences; inorder u4 u3 u10 u2 u8 u1 u9 u5 u11 u6 u7 |
| CO sample S1 heaps | printed H1 not a valid max heap, printed H2 not a valid min heap; Floyd build of all 11 keys as max heap = [96, 80, 19, 9, 9, 1, 6, 3, 9, 4, 5]; as min heap = [1, 3, 6, 4, 5, 9, 19, 80, 9, 9, 96] |
| CO sample S5 SQL (SQLite) | PSC's printed SQL returns duplicates when a supplier supplies 2 red parts (Acme, Acme, Chet); with DISTINCT: Acme, Chet |
| CO practical P2 (SQLite) | after deleting lowest two Sids: 8 rows, min Sid 3 |
| CO practical P3 C++ (g++) | prints 1 / 232 / 34543 / 4567654 / 567898765 with centring spaces |
| CO samples S3 Java (javac/java), S4 C (gcc) | 3×3 table; 1 2 / 2 3 / 3 4 / 4 5 (tab-separated) |
| LL(1) expression grammar | FIRST/FOLLOW as in page; table has no conflicts |
| LCG a=5, c=3, m=16, X0=7 | 6 1 8 11 10 5 12 15 14 9 0 3 2 13 4 7 (period 16). Practice: a=3, c=5, m=16, X0=1 → period 8 (not full) |
| Gap test, 20 numbers, [0, 0.3) | marked positions 3,5,9,11,14,17,20; gaps 1,3,1,2,2,2; descriptive D = 0.300; initial continuous K–S threshold comparison removed in integration review because the geometric null is discrete. No significance or independence conclusion retained. |
| Shannon 3100 Hz, 30 dB | 30.9 kbit/s |
| AM 100 W, m = 0.8 | Pt 132 W, sidebands 32 W, η 24.2%; practice 50 W, m = 0.6 → 59 W |
| CRC 1101011011 / 10011 | remainder 1110; receiver remainder 0000. Practice 101110 / 1001 → 011 |
| Hamming(7,4) data 1011 | codeword 0110011; flipping bit 6 → syndrome 6 |

## 9. IT in Nepal sources consulted (secondary indexes, 2026-11-17)

- Electronic Transactions Act, 2063 — English text on UNODC SHERLOC (authentication 22 Mangsir 2063):
  https://sherloc.unodc.org/cld/uploads/res/document/the-electronic-transactions-act-_html/Nepal_Electronic_Transactions_Act.pdf
- Copyright Act, 2059 — WIPO Lex (authentication 30 Shrawan 2059 / 15 Aug 2002):
  https://www.wipo.int/wipolex/en/legislation/details/7230
- National ICT Policy, 2072 — OPMCM-hosted PDF:
  https://www.opmcm.gov.np/wp-content/uploads/npolicy/Communication/ict_policy_2072.pdf
- Digital Nepal Framework (2019) and draft DNF 2.0 (March 2025) — news reports only; a "National IT
  Policy 2082" is mentioned by a secondary site and is **unconfirmed**.
Law Commission pages were not read directly. The CO page marks each of these "verify".

## 10. Items for the parent (outside this task's file ownership)

1. `lok-sewa/computer-engineer/index.mdx`: Paper I Part II is officially MCQ 50 × 1 (drop the
   "provincial copy shows written — verify" caveat); add 20% negative marking (rate confirmed);
   fix the `psc.gov.np/uploads/course/...` Sources link to `/site_uploads/course/...`.
2. `lok-sewa/computer-officer/index.mdx`: all 12 weights and the Paper II section split are official
   (remove "†"/"per your draft" caveats); add practical 50 (pass 25), 1 h 30, 5 × 10 with unit split,
   and the "must pass practical to be on final list" rule.
3. `nec/networks/04-ip-subnetting.mdx`: worked "reported CE 2078" question uses host .75; the
   transcription says .177.
4. "reported CO 2083" labels across NEC pages have no findable source; consider "unverified".
5. Sidebar: add the new `lok-sewa/preparation` directory (index + 4 pages, `sidebar.order` 0–4).
6. The preparation hub links this evidence log on GitHub `main`; it 404s until committed.

## Integration corrections

Parent review narrowed archive-absence claims to the checked index, qualified CRC guarantees and sampling assumptions, removed invalid continuous K–S calibration for discrete gaps, corrected the SQL names-versus-identities distinction, and marked the existing .75 subnet lesson as adapted practice rather than the transcribed .177 question.

## Additional historical-paper lead search (second pass)

The mbhandari.com.np index is search-indexed as listing a Government of Nepal Computer Officer 2081 item. Its page could not be read in this pass (web fetch 403; direct request 502). No scan/header or question text was verified and no item was promoted to an authenticated past paper. This is a lead only. The issuing authority, exact post, date and complete paper must be inspected before adding question-level provenance.
