# Design — Koma (Ink & Amber)

Written from the built world (M4, 2026-11-24). The direction contract lives in
`.impeccable/surfaces/web-src-routes-page-svelte.md`; this records what shipped.

## Tokens

- Ground ink `#0b0f1a`; panel `#141a2a`; panel-2 `#1a2138`; line `#232c44`
- Text `#e7ebf3`; secondary `#8b93a7`
- Amber accent `#f59e0b` (dim: 14% alpha wash) — active/selected states, the progress
  strip, primary actions
- State colors: ok `#34d399` (downloaded), queued `#60a5fa`, danger `#ef4444`
  (failed, purge)
- Shape: 14px card radius, 8px control radius; 12px gutters, 16px padding

## Type

- Outfit 500/700, self-hosted latin subsets (woff2) for display/headings; -0.02em tracking
- system-ui stack for body; tabular numerals in counters and chapter numbers

## Components

- App shell: top bar (panel mark + wordmark, desktop nav ≥768px), mobile bottom tabs with
  the lit amber panel-dot on the active tab
- Cover panels: 2:3 aspect cards, inset line frame, hover lift with amber frame, the
  panel-mark placeholder when no cover exists
- Category chips: pills; active state is the amber wash with an amber inset ring
- Chapter rows: tabular number, state badge (color + text, never color alone), page count,
  read (amber ring link) / download / working states
- Category cards: inline-editable name, member count, consent-gated destructive flag,
  per-card purge with confirm; membership assign
- Reader chrome: centered sticky pill (strip marker + page counter), amber progress strip
  along the bottom edge
- Messages: amber wash notice, red wash alert

## Signature move

The manga panel: covers as panels in a page grid, and the reading position drawn as the
amber strip. The frame and the progression carry the identity — no other decoration.

## Motion

Hover lift (2px translate + shadow, 160ms ease) on cover panels; strip width 240ms ease.
Nothing else moves.

## Browser surfaces

Selection in the amber wash; focus-visible 2px amber outline; thin dark scrollbars;
inputs on the panel-2 ground with line frame, amber border on focus.

## Gaps recorded

- The desktop shell has no first-run token prompt (localStorage-only credentials;
  `.dojo/findings.md` 2026-11-24).
- Library covers depend on the source providing a cover URL (search-added titles have
  them; `.tachibk` imports render the panel placeholder).
