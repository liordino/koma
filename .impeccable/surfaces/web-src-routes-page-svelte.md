---
version: 1
slug: "web-src-routes-page-svelte"
primary_target: "web/src/routes/+page.svelte"
related_targets: ["web/src/routes/search/+page.svelte","web/src/routes/manga/[id]/+page.svelte","web/src/routes/categories/+page.svelte","web/src/routes/reader/[chapterId]/+page.svelte"]
---

## Scope and visitor mode

Scope: the whole SPA — app shell, library, search, manga detail, categories, reader chrome.
Mode: Operate (the owner completes reading-management tasks; scanability and native
expectations outrank expression).

## Audience, job, action

The single operator, on a phone first: browse the library, find and add new titles,
download chapters, read, manage categories. Proof: the owner's real library data.
Constraints: dark-first, minimal clutter, strip-only reader, separate origins, bearer auth
everywhere, lean device budget.

## Direction contract

THESIS: The library is a manga page — covers as panels in a generous grid, one amber
signature; refuses the category default of text-row lists and dashboard chrome.

OWN-WORLD: Ink ground #0b0f1a, panel surfaces #141a2a, text #e7ebf3, secondary #8b93a7,
single amber accent #f59e0b. Self-hosted Outfit (500/700) display over system body. The
signature move: panel framing — cover cards as manga panels, and the reading position
drawn as an amber frame marker.

STORY: Open the app → the library reads as a page of covers, the continue-reading panel
framed amber; tap a cover → detail with chapter states; or search → add. Progress and
download states are visible at a glance, never color-alone.

FIRST VIEWPORT: 390px: top bar (panel mark + Koma), category chips, 2-up cover grid with
amber-framed continue-reading card first; bottom tab bar (Library / Search / Categories).
Desktop 1440px: same grid at 6-up, tabs in the top bar, page title left.

FORM: Ink & Amber (user-pinned, code-led build; Operate surface).
seed: none (user-pinned direction)

FINISH: unreviewed and undocumented is unfinished; this build ends with the finish review,
the verdict, DESIGN.md, and every shipping raster carrying its provenance.

## Unresolved decisions

None — direction and scope confirmed by the owner (covers-first, search in scope, Ink & Amber).
