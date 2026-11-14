---
title: Video metadata tag, in-app logo, remove setup.sh (Mac only)
date: 2026-11-14
status: done
---

# Video metadata tag, in-app logo, remove setup.sh (Mac only)

## Summary

Write a "built with shardreel" tag into rendered video metadata. Use the main logo mark in the app UI. Delete `setup.sh`. Refresh README and `.env.example`. Project stays Apple Silicon (macOS) only.

## Reason

Light attribution with no watermark. In-app mark should match the site. `setup.sh` belongs to the old Python tool. README must stop pointing at it.

## Scope

- In scope: metadata tag and an off switch, in-app logo swap, delete `setup.sh`, README rewrite for shardreel on Apple Silicon, `.env.example` refresh, small docs site additions for the tag.
- Out of scope: Windows and Linux support (dropped by user 2026-11-14, was in the first draft of this plan), CI workflow, hardware-encoder work, watermark overlay, removing the old Python package (needs a separate yes).

## Decisions

| Item | Choice |
| --- | --- |
| Video tags | `comment`="Built with shardreel", `encoder`="shardreel", `title`=clip title, `description`=caption |
| Switch | "Embed shardreel tag", option `embed_tag`, default on |
| In-app logo | same two-polygon mark as the site (no ids), header size 22, plus empty and loading states |
| README | shardreel, Apple Silicon, build and run, link to docs site name only (no URL known) |

## Files Affected

| File | Change |
| --- | --- |
| setup.sh | delete (user asked) |
| src/pipeline.rs | `Options.embed_tag` (default true); pass to `RenderOpts` |
| src/render.rs | `RenderOpts.embed_tag`; add `-metadata` args when on |
| web/src/types.ts | `embed_tag` on `Options` |
| web/src/OptionsForm.tsx | "Embed shardreel tag" switch in Overlays card |
| web/src/ui.tsx | `Logo` becomes the two-polygon mark |
| web/src/App.tsx | header mark size 22 |
| web/src/pages/JobsPage.tsx, JobPage.tsx | mark in empty and loading states |
| README.md | rewrite |
| .env.example | refresh keys, note `~/.clipforge/.env` |
| shardreel-site `content/docs/reference/options.mdx`, `concepts/how-it-works.mdx` | document `embed_tag` and the tags |

## Steps

1. [x] Delete `setup.sh`
2. [x] Metadata tag in render, `embed_tag` option, UI switch. Verified with ffprobe: `comment`, `title`, `description` at container level, `encoder=shardreel` at video stream level; off switch writes none. Needed `-fflags +bitexact` (otherwise ffmpeg overwrites encoder); MP4 muxer then drops a container-level encoder tag, so it is stream-level.
3. [x] In-app logo: two-polygon mark, header size 22, empty and loading states. Checked in headless Chrome (header and Jobs empty state).
4. [x] `cargo build --release` and `bun run build` clean; clips rendered and tags verified (scratch HOME, user data untouched)
5. [x] README rewritten for shardreel on Apple Silicon (no Python, no setup.sh); `.env.example` refreshed (removed unused `CLIPFORGE_LLM`)
6. [x] Docs site: `embed_tag` in options reference, Video metadata section in captions guide, note in how-it-works; site rebuilt clean
7. [x] Removed old Python package `clipforge/` (12 .py files) and `pyproject.toml` on user's go (2026-11-14). Checked first: no references from `src/`, `web/src/`, README or `.env.example`. `cargo build --release` and `bun run build` still pass. No `uv tool` install of the Python CLI found on this Mac.

## Side Effects

- Saved jobs keep working. `embed_tag` defaults to true when missing.
- Tags can be stripped by upload platforms or re-encodes. They are attribution, not protection.
- Old Python package stays until step 7 is approved, so README must not tell people to use it.

## Notes

- Deviation 2026-11-14: user dropped cross-platform work. Earlier draft covered Windows and Linux (paths, `which`, software encoding, fonts, whisper features, CI). Not implemented.
- `encoder` tag: ffmpeg overwrote the container-level value; fixed with bitexact plus a stream-level tag (verified).
- Step 7 done: Python files removed.
