---
title: Storage panel to clear disk space
date: 2026-11-14
status: done
---

# Storage panel to clear disk space

## Summary

Settings page gets a Storage card: sizes of cache categories and buttons to clear each. Backend adds storage routes.

## Reason

Cache grows fast (one 2.4 GB download already). User needs to free disk from the UI.

## Scope

- In scope: size report, clear downloads, clear analysis cache, clear uploads, delete a whisper model; refuse while a job runs.
- Out of scope: deleting rendered clips (output folders are never touched), per-file picking, auto-cleanup schedules.

## Files Affected

| File | Change |
| --- | --- |
| src/storage.rs | new: `report()` sizes per category, `clear(kind)`; guards path to stay inside `~/.clipforge` |
| src/api.rs | `GET /api/storage`, `DELETE /api/storage/{kind}` (kind: downloads, analysis, uploads, or model name); 409 if a job is queued or running |
| src/jobs.rs | `busy()` helper: any job queued or running |
| src/main.rs | `mod storage;` |
| web/src/types.ts | `Storage` types |
| web/src/api.ts | `storage()`, `clearStorage(kind)` |
| web/src/pages/SettingsPage.tsx | Storage card: row per category with size, Clear button, confirm step, total |

## Categories

| Kind | Path under `~/.clipforge` | Effect of clearing |
| --- | --- | --- |
| downloads | `cache/downloads/` | URL jobs re-download on next run |
| analysis | `cache/<hash>/` (audio.wav, transcript, signals, picks) | next run re-transcribes and re-analyses |
| uploads | `uploads/` | jobs made from uploads cannot be re-rendered |
| model | `models/<file>` | re-downloaded on next use (turbo ~1.6 GB) |

## Steps

1. [x] Backend: `storage.rs` size report and clear with path guard
2. [x] Backend: routes + `busy()` check (409 path not exercised)
3. [x] UI: types, api calls, Storage card with confirm and refresh (whisper models included, one row each)
4. [x] Run check: `cargo build --release`, `bun run build` clean. API tested against a fake HOME in scratch: report sizes, clear each kind, traversal and unknown kinds return 400, `jobs/` untouched. UI not checked in a browser by agent.

## Side Effects

- Clearing downloads or uploads breaks re-render for jobs whose source file lived there (job page shows error). UI text warns before clearing.
- Local-path jobs (video in place) are never deleted.
- Saved job records in `~/.clipforge/jobs/` are kept.

## Notes

- Confirm step in UI: first click turns button into "Confirm clear", second click runs it.
- Sizes computed by walking directories; fine at this scale.
- Open: include whisper models in the list (recommended), or hide them to avoid accidental 1.6 GB re-download?
