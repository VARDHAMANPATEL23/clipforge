---
title: Fix OpenRouter default model, mark facecam as beta
date: 2026-11-14
status: done
---

# Fix OpenRouter default model, mark facecam as beta

## Summary

Change the built-in OpenRouter model to a valid name. Label the facecam layout as beta in the UI, docs and README. Fix the facecam region number inputs that wipe themselves.

## Reason

- OpenRouter default `google/gemini-flash-lite-latest` is not a real OpenRouter model, so an empty model fails and falls back to signal ranking. Verified earlier against OpenRouter's public model list.
- Facecam is not reliable in the UI. Findings from testing: the engine renders the facecam layout correctly (webcam crop on top, gameplay below, checked with a real render). UI problems: (1) typing the region numbers by hand resets the fields after the first digit, because the region is stored only when all four values are valid; (2) "Pick on frame" needs a local file or upload, so URL jobs cannot set a region; (3) the drag picker was never exercised in a real browser by the agent.

## Scope

- In scope: `llm.rs` default, a Beta badge and hint on the facecam tile and region section, docs and README notes, fix for the number-input reset (small, local draft state), clearer facecam region UX (user could not tell what to do, 2026-11-14 screenshot).
- Out of scope: reworking the picker, region picking for URL sources, new layout features, any change to the render filter graph.

## Decisions

| Item | Choice |
| --- | --- |
| OpenRouter default model | `google/gemini-3.8-flash` (valid on OpenRouter, accepts images, so it also covers vision). Cheaper alternative `google/gemini-3.5-flash-lite` stays a manual choice |
| Beta marking | small outlined "Beta" badge on the Facecam tile and next to "Webcam region", plus one hint line. Layout still selectable |
| Number inputs | keep four text drafts in the form; commit the region to the job only when all four are valid numbers in 0 to 1, clear it otherwise |
| Region UX | show a short numbered guide instead of bare x/y/w/h: 1) choose a video in Source, 2) press Pick on frame, 3) drag a box around your webcam. When no local video is chosen, show the reason next to the disabled button ("Choose an uploaded or local video first"). When a region is set, show a plain summary (for example "Webcam: bottom right, 26% wide, 32% tall") with a Change button. Move the four numbers into a collapsed "Enter numbers manually" section |

## Files Affected

| File | Change |
| --- | --- |
| src/llm.rs | OpenRouter default model `google/gemini-3.8-flash` |
| web/src/OptionsForm.tsx | Beta badge on tile and region header, hint line, draft state for region inputs, guided region section with plain summary and collapsed manual numbers |
| web/src/styles.css | `.beta` badge style |
| README.md | facecam marked beta, OpenRouter default note |
| shardreel-site `content/docs/guides/ai-ranking.mdx` | OpenRouter callout: default model is now valid; still recommend setting it |
| shardreel-site `content/docs/troubleshooting.mdx` | adjust the model-name bullet |
| shardreel-site `content/docs/guides/layouts.mdx` | Beta callout on facecam, note that region picking needs a local file |
| shardreel-site `content/docs/reference/options.mdx` | mark `layout.facecam` beta |
| shardreel-site `components/landing.tsx` | "Facecam (beta)" in the layouts section |

## Steps

1. [x] OpenRouter default in `llm.rs` is now `google/gemini-3.8-flash`. Verified with the mock server: log shows `ranking 4 candidates with google/gemini-3.8-flash` and the provider received that model name
2. [x] Beta badge on the Facecam tile and the region section, plus a hint line
3. [x] New `FacecamRegion` component: numbered guide, disabled reason, plain-English summary with Change and Clear, manual numbers collapsed with their own draft state (typing bug fixed)
4. [x] README, docs (ai-ranking, layouts, options reference) and landing page updated; UI and site rebuilt clean
5. [x] Driven through Chrome DevTools: typed four numbers one at a time and they stayed; summary read "bottom right, 26% wide, 32% tall"; Pick on frame enabled with a local video; dragged a box on the picker and Apply gave "bottom right, 25% wide, 31% tall"; screenshots checked

## Side Effects

- Anyone relying on the old (broken) OpenRouter default now gets `google/gemini-3.8-flash`, which costs more than a Lite model. Explicit model choices are unchanged.
- Saved jobs and settings are unaffected.
- Facecam stays usable. Beta is a label, not a restriction.

## Notes

- The Gemini direct defaults (`gemini-flash-lite-latest`, `gemini-flash-latest`) are aliases on Google's API and were not changed.
- Update: the drag picker was driven through Chrome DevTools from a scratch bun script and works. Beta stays because it was only tested on a synthetic video, not real webcam footage.
