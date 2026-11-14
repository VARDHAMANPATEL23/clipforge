---
title: Fix logo mark rendering in docs layout
date: 2026-11-14
status: done
---

# Fix logo mark rendering in docs layout

## Summary

Draw the logo mark as two plain polygons, with no `clipPath` ids, so it renders the same everywhere.

## Reason

Docs pages show a plain triangle instead of the sliced peak. The nav title element is rendered twice in the docs layout (sidebar and mobile header), so the `clipPath` ids are duplicated. The first copy sits in a hidden container, the `url(#id)` reference breaks, and the clip is dropped. Verified: `/docs/getting-started` has 4 `clipPath` elements with 2 unique ids; home has 10 unique ids.

## Scope

- In scope: `Logo` component in `shardreel-site`.
- Out of scope: favicon and standalone `logo*.svg` files (separate documents, no collision), the app UI `Logo` (renders once per page), any other styling.

## Files Affected

| File | Change |
| --- | --- |
| `/Users/rajdpatel/RAJ/git/shardreel-site/components/logo.tsx` | replace clipPath markup with two precomputed polygons; drop `useId` |

## Geometry

Peak triangle (6,56) (32,8) (58,56), cut along (0,46)-(64,34), top piece moved (+3.5,-3.5):

- Bottom piece: (6,56) (58,56) (47.735,37.050) (12.707,43.617)
- Top piece, already shifted: (35.5,4.5) (51.235,33.550) (16.207,40.117)

Same shape as the current mark, expressed without clipping.

## Steps

1. [x] Rewrite `Logo` with the two polygons
2. [x] `bun run build` clean; `/docs/getting-started` and `/` now have 0 `clipPath` elements
3. [x] Screenshots: docs (mobile header and desktop sidebar) and home nav all show the sliced mark

## Side Effects

- None visible. Mark looks the same on the home page.

## Notes

- The app UI (`clipforge/web`) keeps its `useId` version. It has one Logo in the header. Switch it to polygons too if it ever renders twice.
