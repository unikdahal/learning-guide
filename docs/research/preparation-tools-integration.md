# Preparation tools integration

This increment adds technical mock rounds, IPv4 worked calculations, a PSC workstream/mistake notebook, and a syllabus-tagged emerging-technology lesson. It complements the shared NEC checklist and original-question bank; it does not establish full concept coverage or a complete authenticated past-paper archive.

## Review corrections

- Canonicalize multi-answer mock journals before comparing saved answer history, including interrupted final writes.
- Retain active in-memory work after quota failures and expose an active save retry. Reject stale finalisation against a different saved round, and reject changed durable state while a tab holds unsaved work.
- Use one Web Lock for browser practice-history writers: quiz submissions/bookmarks, mock finalisation and preparation backup restore. Browsers without Web Locks fall back to synchronous local writes; cross-tab serialization is unavailable there.
- Preserve pending journals and matching history in backups without replaying answers. Legacy backups clear the mock namespace; absent notebook data leaves existing notes alone. Conflicting journals remain exportable and stop automatic recording.
- Validate raw prefix and borrowed-bit inputs before updating controls or last-valid calculations. Blank prefixes are invalid, not /0.
- Scope labels cover a teaching subset of special-use IPv4 space. The fallback is “unclassified,” which makes no routability claim. RFC 6890 documents 0/8 and 192.0.0/24 special-purpose space: https://www.rfc-editor.org/rfc/rfc6890.html
- The .177 subnet example is labelled as an unverified historical transcription. Other numerical examples are original/adapted preparation exercises. No frequency or guaranteed-mark claims are made.
- Correct the block-size formula at octet boundaries and distinguish exclusion of subnet zero alone from exclusion of both zero/all-one subnet IDs.

## Coverage and limits

The notebook's 36 editorial workstreams complement shared-core preparation. Completion is a personal self-check, not measured readiness or exhaustive coverage of every PSC bullet. Mistake notes never change scored history. A later attempt makes an older checked correction stale.

Mocks use the 120 original NEC questions and balanced chapter quotas. They omit PSC general awareness and post-specific extras. Only one current round/report is retained. Backup deadlines are absolute and are not paused during export. Browser storage is user-editable and can be unavailable or cleared.

The new emerging-technology cases and their primary-source and numerical evidence are recorded separately in `emerging-technology-evidence.md`.

## Validation

- 57 unit tests passed, including shuffled multi-answer interrupted-save recovery, stale callers, optional backup namespaces, notebook sanitization and special-use scope boundaries.
- Astro check: 0 errors, warnings or hints across 156 files.
- 503 static pages built successfully.
- 59 distinct focused browser scenarios passed across the integration and correction runs. The final affected subnet suite passed all 11 tests, including /32-to-/24 control recovery. Coverage includes keyboard/mobile flows, both-theme accessibility checks, no-JavaScript fallbacks, quota failures, cross-tab history locking, and backup restoration without answer replay.
- `git diff --check` passed. Generated-site checks cover links, anchors, duplicate IDs, unpublished indexing and byte budgets.
