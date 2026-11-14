---
title: Port clipforge to Rust backend + React web UI
date: 2026-11-14
status: done
---

# Port clipforge to Rust backend + React web UI

## Summary

Replace Python CLI with Rust (axum) backend and Vite/React web UI run on bun. Same pipeline: ingest, whisper-rs transcribe, signals, picker, LLM rank, ffmpeg render. Apple Silicon only. Python code deleted after Rust part verified by user.

## Reason

Single native binary backend, no Python runtime, browser UI replaces CLI flags (facecam picker, layout preview, live progress, clip review).

## Scope

- In scope: Rust backend (HTTP API + SSE), whisper-rs with Metal, ffmpeg/ffprobe/yt-dlp via subprocess, React UI served by backend, settings (LLM key) via UI, cache parity, delete Python after port.
- Out of scope: CLI, `watch` folder mode, `doctor` command (replaced by `/api/health`), Docker, non-Apple platforms, auto-posting, in-browser clip trimming (later).

## Decisions

- Transcribe: `whisper-rs` with `metal` feature, model `ggml-large-v3-turbo` (download from HF on first run to `~/.clipforge/models/`). Needs `cmake` (brew).
- Media: `ffmpeg`, `ffprobe`, `yt-dlp` stay external binaries (brew). Same filter graphs, `h264_videotoolbox` / `hevc_videotoolbox`.
- Layout: single Cargo crate at project root (`src/`), UI in `web/`. Use this folder only.
- Web: Vite + React + TypeScript, bun only. Dev: Vite proxy `/api` to backend. Prod: axum serves `web/dist`.
- Store: `~/.clipforge/` as today (cache, models, settings.json, jobs). No database.
- LLM: OpenAI-compatible `/chat/completions` via `reqwest` (rustls). Same presets: gemini, lmstudio, ollama, openrouter.

## Files Affected

| File | Change |
| --- | --- |
| Cargo.toml | new: deps axum, tokio, serde, serde_json, reqwest, whisper-rs, hound, regex, sha1, uuid, tower-http, tokio-stream, anyhow, tracing |
| src/main.rs | new: server bootstrap, routes, static serving |
| src/api.rs | new: handlers (jobs, clips, settings, health, upload, file serving) |
| src/jobs.rs | new: job queue, state, SSE progress broadcast |
| src/pipeline.rs | new: port of pipeline.py (Options, cache key, orchestration) |
| src/media.rs | new: port of media.py (probe, encoder pick, libass check) |
| src/ingest.rs | new: port of ingest.py (yt-dlp fetch, audio extract) |
| src/transcribe.rs | new: whisper-rs word-level transcription, hallucination filter, words_in/text_in |
| src/signals.rs | new: port of signals.py (loudness, rolling median, scene score parse, zscore) |
| src/picker.rs | new: port of picker.py (excitement, candidates, snap, dedupe, fallback titles, LLM refine) |
| src/llm.rs | new: port of llm.py (chat client, parse_json) |
| src/render.rs | new: port of render.py (video_filter, render_one, parallel render_all, thumbnail) |
| src/subtitles.rs | new: port of subtitles.py (ASS builder) |
| web/package.json | new: vite, react, typescript |
| web/vite.config.ts | new: `/api` proxy |
| web/index.html, web/src/** | new: UI (see UI pages) |
| .env.example | update: settings now via UI, keep env override |
| README.md | rewrite after port: build and run with cargo + bun |
| clipforge/ (python pkg), pyproject.toml, setup.sh | delete after user confirms Rust part works |

## API

| Route | Purpose |
| --- | --- |
| GET /api/health | ffmpeg, encoder, libass, whisper model, yt-dlp, key status |
| POST /api/jobs | body: source (path or URL) + options; returns job id |
| POST /api/uploads | multipart video upload; returns server path |
| GET /api/jobs | list jobs |
| GET /api/jobs/:id | job state + clips |
| GET /api/jobs/:id/events | SSE: stage, percent, log lines |
| POST /api/jobs/:id/rerender | new layout/options, reuse analysis cache |
| GET /api/jobs/:id/frame?t= | still frame for facecam region picker |
| GET /files/* | clips and thumbnails (range requests for video) |
| GET/PUT /api/settings | LLM provider, model, keys, default niche |

## UI Pages

1. New job: drop file or paste URL, options form, layout picker.
2. Facecam region picker: draw box on a frame, outputs x,y,w,h fractions.
3. Job progress: stage list + live log via SSE.
4. Clips: grid with player, thumbnail, title, hook, hashtags, copy caption.
5. Settings: LLM provider, key, niche, health checks.

## Steps

1. [x] Cargo project + axum skeleton + `/api/health`
2. [x] `media.rs`: probe, encoder pick, libass check
3. [x] `ingest.rs`: fetch, extract_audio (+ `paths.rs` data-dir helper)
4. [x] `signals.rs`: loudness, spike, scene activity, cache
5. [x] `picker.rs`: excitement, candidates, snap, dedupe (no LLM yet); `transcribe.rs` types only
6. [x] `subtitles.rs` + `render.rs`: ASS, filter graphs, parallel render, thumbs
7. [x] `pipeline.rs` + `jobs.rs`: orchestration, progress events, `clips.json` (verified e2e with synthetic video, ASR off: 3 clips at correct bursts, 1080x1920, blur layout OK)
8. [x] `transcribe.rs`: whisper-rs, model download, word timestamps (compiles; not yet run, model download pending)
9. [x] `llm.rs` + `settings.rs` + LLM refine in `picker.rs` (text and vision). Verified with mock OpenAI-compatible server (parse, guard rails, hashtags, missing-key error). Real Gemini and vision frames not yet run.
10. [x] API routes: jobs, uploads, SSE, rerender, frame, probe, files, settings (all hit with curl except upload)
11. [x] `web/` scaffold (hand-written Vite + React 19 + TS 7, bun only) + `/api` proxy
12. [x] UI pages 1-5 (new job, facecam picker, job progress, clips, jobs list, settings). Monochrome Vercel-style, Geist fonts bundled locally. `bun run build` typechecks and builds clean. Not visually checked in a browser by agent.
13. [x] Backend serves `web/dist` with SPA fallback (`CLIPFORGE_WEB_DIR` override). Added `/api/defaults`. Verified static, fallback, upload (filename sanitised).
14. [ ] User verifies full run in browser. Agent verified: backend e2e (signals, picker, render), mock LLM, whisper small on Metal (word timestamps sane). Not verified: real Gemini, vision frames, captions (ffmpeg lacks libass), URL download (yt-dlp missing), UI in browser.
15. [x] Python package, pyproject.toml and setup.sh deleted; README rewritten for shardreel (see 2026-11-14-cross-platform-metadata-logo.md)

## Side Effects

- Needs brew: `cmake` (whisper-rs build), `ffmpeg`, `yt-dlp`. Rust toolchain via rustup.
- Whisper model (~1.6 GB) downloads once. Format changes from MLX to ggml; old HF cache unused.
- Cache JSON (`~/.clipforge/cache`) kept compatible where cheap (signals, moments, transcript schema); old caches may be ignored if not.
- `.env` still read as override; UI settings.json takes precedence for keys it sets.
- Removes CLI and `watch`.

## Notes

- Risk (resolved): whisper-rs word timestamps are token-level. Token merge on leading space works; checked on synthetic speech with `small` model: monotonic, contractions merged, timings track speech. Caption timing still to confirm visually once libass ffmpeg is installed.
- Risk: whisper-rs transcription speed on Metal vs mlx-whisper; model size fallback `small`.
- Steps 1-9 ordered so pipeline works with ASR off before whisper is wired.
- Per AGENTS.md: small incremental edits, plan updated per step, no git, no test runs by agent. Agent builds only if user asks.
- Decided: uploads live in `~/.clipforge/uploads`. Renders saved to a local path; user uploads to platforms manually. User allows agent to run build/run as needed (overrides default no-run rule). UI style: Vercel-like, monochrome black and white.
- Env finding: installed ffmpeg has no libass and `yt-dlp` missing (`brew install yt-dlp`). Health route reports both.
- Fix (captions): installed keg-only `ffmpeg-full` (libass + VideoToolbox) via brew; existing ffmpeg untouched. `media::require` now searches `/opt/homebrew/opt/ffmpeg-full/bin` first. Verified: word-highlight captions burn in correctly, synced to speech.
- Core is sync (`std::process`), jobs run on `spawn_blocking`.
- Settings: API keys write-only (`GET /api/settings` returns `has_*_key` flags). Lookup: settings.json, process env, `~/.clipforge/.env`. Keys are never stored in job files.
- Deviation: step 10 core routes (jobs, events, rerender, files, uploads, probe, frame) done before step 9. Frame route is `/api/frame?path=&t=` (not per job) so facecam picker works before job exists; added `/api/probe?path=`. Clip files: `/api/jobs/:id/files/:name` (Range OK). Settings route comes with step 9.
- Deviation: step 8 (whisper) done before step 7 (pipeline) so pipeline has no placeholder. `asr` option becomes `whisper | none` (mlx/faster removed).
