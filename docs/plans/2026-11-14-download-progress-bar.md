---
title: Show yt-dlp download progress bar
date: 2026-11-14
status: done
---

# Show yt-dlp download progress bar

## Summary

Stream yt-dlp progress during URL download and show a live progress bar (percent, size, speed, ETA) on the job page.

## Reason

URL downloads show no feedback now. Long downloads look stuck.

## Scope

- In scope: parse yt-dlp progress lines, new `progress` job event, progress bar on job page during fetch stage.
- Out of scope: cancel button, whisper model download bar (same event can serve it later), playlist support.

## Files Affected

| File | Change |
| --- | --- |
| src/ingest.rs | `fetch` streams yt-dlp stdout with `--newline --progress --progress-template`, calls progress callback; still returns final file path |
| src/pipeline.rs | add `Event::Progress` (percent, downloaded, total, speed, eta); pass callback to `ingest::fetch` |
| src/jobs.rs | add `progress` field on `Job`; broadcast `{"type":"progress",...}`; clear on stage change |
| web/src/types.ts | `Progress` type, `progress` on `Job`, `progress` in `JobEvent` |
| web/src/pages/JobPage.tsx | handle `progress` event; bar under stepper while fetching |
| web/src/styles.css | reuse `.progress`; small text row for stats (only if needed) |

## Steps

1. [x] Backend: stream yt-dlp output in `ingest::fetch`, parse progress template
2. [x] Backend: `Event::Progress` + job field + SSE event (verified: direct mp4 and YouTube URL emit video then audio progress with percent, bytes, speed, ETA)
3. [x] UI: types, event handling, `DownloadBar` (label video/audio, percent, MB, speed, ETA) in JobPage, shown during fetch stage
4. [x] Run check: `cargo build --release` and `bun run build` clean. UI not checked in a browser by agent.
5. [-] Dropped by user: downloads stay in `~/.clipforge/cache/downloads/` (cache location approved as is). Added Scope section below not implemented. Disk clean-up handled in `2026-11-14-clear-cache-ui.md`.

## Added Scope (2026-11-14, awaiting confirm)

Request: keep downloaded videos in Movies, skip download when same URL used again, show downloaded path, folder named after the URL part where the video sits.

- Location: `~/Movies/clipforge/downloads/<id>/` (was `~/.clipforge/cache/downloads/`). `<id>` = unique part of URL: YouTube `v=` value or `youtu.be/<id>`, Twitch `videos/<id>`, else last path segment of URL, else short hash of URL. Sanitised to `[A-Za-z0-9_-]`.
- Reuse: before calling yt-dlp, if `<id>/` holds a finished video file (mp4/mkv/webm/mov, no `.part`), skip download, log "using existing download", emit no progress.
- Path shown: log line `saved: <path>` after download or reuse, plus `downloaded` path on the job (job page header shows it with copy button).
- Files: `src/ingest.rs` (id from URL, reuse check, new dir), `src/pipeline.rs` (log path, set job path), `src/jobs.rs` (optional `video_path` field), `web/src/types.ts`, `web/src/pages/JobPage.tsx`.
- Side effects: old downloads in `~/.clipforge/cache/downloads/` are not reused; yt-dlp `-o` template becomes `<id>/%(title).80B [%(id)s].%(ext)s`? Decision: keep `%(id)s.%(ext)s` inside the folder (simple, stable name).
- Open: output clip folder uses `slug(video stem)`, so it becomes the id (e.g. `dQw4w9WgXcQ`). Fine, or use the video title for the clips folder?

## Side Effects

- `fetch` signature gains a callback argument; only `pipeline.rs` calls it.
- `Job` JSON gains optional `progress`; old saved jobs load fine (serde default).
- Needs `yt-dlp` installed (`brew install yt-dlp`) to test; currently missing.

## Notes

- yt-dlp `--print` is quiet by default, so `--progress --newline` is required for progress lines.
- Template: `download:%(progress.downloaded_bytes)s|%(progress.total_bytes)s|%(progress.total_bytes_estimate)s|%(progress.speed)s|%(progress.eta)s`. Total falls back to estimate when exact total is `NA`.
- yt-dlp downloads video and audio as two streams, so percent resets once. UI shows "Downloading video" then "Downloading audio" if file extension hints allow; otherwise one bar that restarts. Decision needed below.
- Open: accept the restart between video and audio streams, or show combined progress (more code)?
