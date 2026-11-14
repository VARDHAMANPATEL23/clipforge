---
title: Peakcut UI branding and logo
date: 2026-11-14
status: done
---

# Peakcut UI branding and logo

## Summary

Show the name "peakcut" and a new logo in the web UI. Rust package, binary, API, and `~/.clipforge` data folder stay named clipforge.

## Reason

User dislikes the name clipforge for the product. Chosen UI name: peakcut. Internal tool name stays to avoid renames, data migration, and API churn.

## Scope

- In scope: logo mark (peak sliced diagonally, top piece shifted), header brand, browser tab title, favicon (dark and light aware), standalone logo SVG files.
- Out of scope: Cargo package name, binary name, `~/.clipforge` folder, env var names (`CLIPFORGE_*`), API routes, web `package.json` name, README rewrite (done in the later Python-removal step).

## Files Affected

| File | Change |
| --- | --- |
| web/src/ui.tsx | add `Logo` component (inline SVG, `currentColor`, unique clip ids) |
| web/src/App.tsx | header brand uses `Logo` + text "peakcut" in place of triangle + "clipforge" |
| web/index.html | title "peakcut"; favicon link points to `/favicon.svg` |
| web/public/favicon.svg | new: mark with `prefers-color-scheme` fill (white on dark tabs, black on light) |
| web/public/logo.svg | new: mark, black, for reuse |
| web/public/logo-wordmark.svg | new: mark + "peakcut" text as paths-free SVG using system font stack |

## Steps

1. [x] Add `Logo` component and use it in header
2. [x] Update `index.html` title and favicon, add `web/public` SVG files (favicon.svg, logo.svg, logo-wordmark.svg)
3. [x] `bun run build` clean; backend serves `/favicon.svg` (200, image/svg+xml) and new title. Header not checked in a live browser by agent.

## Side Effects

- UI text mentioning `~/.clipforge/uploads` stays (real path, still accurate).
- Browser may cache the old favicon; hard refresh needed.
- Product name (peakcut) and tool name (clipforge) differ; README should say "peakcut, powered by the clipforge engine" when rewritten.

## Notes

- 2026-11-14: name later changed from peakcut to shardreel (peakcut already in use). Same mark kept. See `2026-11-14-docs-site-nextjs-fumadocs.md`.

- Mark geometry: triangle (6,56) (32,8) (58,56) in a 64 box, cut along (0,46)-(64,34), top piece moved (+3.5,-3.5).
- Wordmark: lowercase, weight 600, tight tracking, matches Geist look of the UI.
- Trademark and domain checks for "peakcut" not done.
