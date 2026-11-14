---
title: Brand kit zip (logos in SVG and PNG)
date: 2026-11-14
status: done
---

# Brand kit zip (logos in SVG and PNG)

## Summary

Produce a zip of shardreel logo files, one file per asset, in SVG and PNG, ready to drop into a GitHub repo.

## Reason

User wants the logos for GitHub (README header, social preview, avatar, favicon).

## Scope

- In scope: mark, wordmark, text-only, avatar, favicon, GitHub social previews; black and white variants; kit README.
- Out of scope: any change to the app or site code, licence choice for the logo files, trademark filing.

## Files Affected

| File | Change |
| --- | --- |
| /Users/rajdpatel/Downloads/shardreel-brand/ | new folder: `svg/` (9 files), `png/` (mark, wordmark, avatar, favicon, social: 28 files + favicon.ico), `README.md` |
| /Users/rajdpatel/Downloads/shardreel-brand.zip | new: the folder zipped (47 entries, about 300 KB) |

Nothing inside the clipforge repo changed. Generators lived in the scratchpad (a throwaway Python venv with fontTools and uharfbuzz, and bun scripts driving headless Chrome).

## Steps

1. [x] Outline "shardreel" from Geist SemiBold (variable font instanced at weight 600, kerned, tracking -0.04em), "reel" at 55% opacity
2. [x] Write SVGs (mark, wordmark, text, avatar, favicon), all validated as XML
3. [x] Render PNGs through Chrome DevTools (transparent for logos, opaque for social previews), build `favicon.ico`
4. [x] Visual check on light and dark backgrounds, then zip

## Side Effects

- Found and fixed a bug on the way: the first render showed missing-glyph boxes instead of the wordmark because the shaper could not read WOFF2. Fixed by saving plain TrueType, then all files were regenerated and re-checked.
- The favicon `.ico` and apple-touch icon are white-on-black (they cannot adapt to a theme). The `favicon.svg` adapts.

## Notes

- Wordmark font is Geist (SIL Open Font License 1.1); outlines are fine to ship.
- No licence is included for the logo files. User should add one.
- Trademark and `.app`, `.io`, `.gg` domain checks for shardreel are still manual.
