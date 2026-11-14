---
title: Landing site and docs repo (Next.js + Fumadocs)
date: 2026-11-14
status: done
---

# Landing site and docs repo (Next.js + Fumadocs)

## Summary

New separate repo with a Next.js landing page and Fumadocs docs: what the tool is, setup, how-to guides, config and API reference. Plus rename UI brand from peakcut to the new name.

## Reason

Peakcut name is already used elsewhere. Project needs a home for details and guides, separate from the app code.

## Scope

- In scope: new folder outside `clipforge/`, Next.js App Router, Fumadocs, Tailwind, bun only, monochrome black and white design matching the app, landing page, docs content, new-name rename in UI (title, header text, wordmark svg).
- Out of scope: deploying, analytics, blog, i18n, git setup (version control is handled externally), renaming Rust package, `~/.clipforge`, env vars.

## Decisions (defaults, change if wrong)

| Item | Default |
| --- | --- |
| Product name | **shardreel** (user decision; .com free, 0 GitHub repos, npm and crates free; peakcut dropped: name already in use) |
| Location | **`/Users/rajdpatel/RAJ/git/shardreel-site/`** (user decision) |
| Logo | sliced-peak mark kept as is (user loves it); wordmark two-tone "shard" + "reel" at 55% |
| Stack | Next.js (App Router, TypeScript), fumadocs-ui + fumadocs-core + fumadocs-mdx, Tailwind v4, Geist fonts |
| Look | black and white, 1px borders, Vercel-like, logo mark from app; dark default with light mode |
| Screenshots | CSS mockups of the app UI on the landing page (no browser capture by agent) |

## Files Affected

Existing project (UI rename only):

| File | Change |
| --- | --- |
| web/src/App.tsx | header text "peakcut" to two-tone "shardreel" |
| web/src/styles.css | `.brand .dim` opacity rule |
| web/index.html | title "shardreel" |
| web/public/logo-wordmark.svg | wordmark text "shardreel" |
| docs/plans/2026-11-14-peakcut-ui-branding.md | note rename in Notes (plan file kept, not renamed) |

New repo (`spikecut-site/`):

| File | Change |
| --- | --- |
| package.json, bun.lock, tsconfig.json, next.config.mjs, source.config.ts, postcss config | scaffold |
| app/(home)/page.tsx | landing: hero, how it works, features, layouts, install CTA |
| app/docs/layout.tsx, app/docs/[[...slug]]/page.tsx | Fumadocs docs shell |
| app/api/search/route.ts | docs search |
| lib/source.ts, lib/layout.shared.tsx | Fumadocs source and shared layout (logo, nav) |
| app/global.css | monochrome theme tokens |
| components/logo.tsx, public/ | logo, favicon |
| content/docs/**.mdx, meta.json | docs content (below) |

## Docs Outline

1. Introduction: what it does, how it works at a glance, rights note
2. Getting started: requirements (Apple Silicon, brew deps: ffmpeg-full, yt-dlp, cmake, Rust, bun), build and run, first job
3. Guides: new job and sources (upload, path, URL), layouts and facecam picker, captions and hook banner, AI ranking (Gemini, OpenRouter, local), env file, re-render, download progress, storage and clearing disk space
4. How it works: pipeline stages, signals, moment picking and scoring, whisper word timestamps, LLM guard rails, caching
5. Reference: options, settings, env vars, files under `~/.clipforge`, HTTP API and SSE events
6. Troubleshooting: libass missing, yt-dlp 403, whisper model download, key not detected, port in use

## Steps

1. [x] Confirm name and location with user (shardreel, `/Users/rajdpatel/RAJ/git/shardreel-site/`; user said go ahead 2026-11-14)
2. [x] Rename UI brand in `clipforge/web` (two-tone shardreel header, title, wordmark svg) and rebuild; no `peakcut` left in `web/`
3. [x] Scaffold with `create-fumadocs-app` (template +next+fuma-docs-mdx, orama search, biome, `--no-git`, bun)
4. [x] Theme: black and white tokens, Geist via @fontsource-variable, logo, nav, favicon, dark default; GitHub links removed (no repo URL known)
5. [x] Landing page: hero, app mockup, pipeline, layouts, features, CTA, footer
6. [x] 19 docs pages (intro, getting started, 8 guides, 3 concepts, 4 reference, troubleshooting) written from the real code
7. [x] `bun run build` clean: landing + 19 docs pages + search, llms.txt, og images
8. [x] Visual check via local headless Chrome screenshots (not the preview tool): landing and a docs page look right in dark mode. Light mode and mobile widths not checked.

## Side Effects

- Second project to maintain; docs must follow app changes.
- Docs state facts from this repo; claims not verified (real Gemini run, captions on other machines) will be worded as such.

## Notes

- Domain and trademark for the chosen name still need a manual check; .app/.dev checks were rough.
- Fumadocs scaffolder (`create-fumadocs-app`) may be interactive; fall back to manual scaffold if flags do not cover it.
