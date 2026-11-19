---
title: Pass YouTube cookies to yt-dlp
date: 2026-11-19
status: done
---

# Pass YouTube cookies to yt-dlp

## Summary

yt-dlp gets `--cookies <file>` when cookie file found. Add `--socket-timeout 30 --retries 5` so stalled download fails or retries.

## Reason

Downloads stall on YouTube without logged-in cookies.

## Scope

- In scope: cookie lookup in `src/ingest.rs`, gitignore `cookies.txt`.
- Out of scope: UI setting for cookies, browser cookie import.

## Files Affected

| File | Change |
| ---- | ------ |
| src/ingest.rs | add cookie path lookup + yt-dlp args |
| .gitignore | ignore cookies.txt |
| cookies.txt (project root, untracked) | copy of user's attached file |

## Steps

1. [x] Add `cookies.txt` to `.gitignore`
2. [x] Copy attached cookies file to project root as `cookies.txt`
3. [x] In `fetch`, use `CLIPFORGE_COOKIES` env path, else `./cookies.txt` if exists; add `--cookies`
4. [x] Add `--socket-timeout 30 --retries 5`

## Side Effects

Cookies are session secrets. File is gitignored; never commit. Rotate by re-exporting if leaked. yt-dlp may rewrite the file.

## Notes

No tests run; user runs `cargo build` and a download.
