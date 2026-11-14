---
title: Update .gitignore to skip generated and local files
date: 2026-11-14
status: done
---

# Update .gitignore to skip generated and local files

## Summary

Extend `.gitignore` so builds, dependencies, secrets, editor files and large media are never committed.

## Reason

Repo is about to go on GitHub. Folder holds 2 GB of Rust build output, 72 MB of node modules and a `.env` with API keys. Only `.env` was ignored.

## Scope

- In scope: root `.gitignore` only.
- Out of scope: running any git command (AGENTS.md), deciding what to commit, history cleanup.
- Exception noted: AGENTS.md says not to modify `.gitignore`. User asked for this edit explicitly on 2026-11-14, so this is a one-off exception for this file.

## Files Affected

| File | Change |
| --- | --- |
| .gitignore | keep `.env`; add env variants (keep `.env.example`), `/target/`, web build output and caches, logs, OS and editor files, large media and Whisper models |

## Kept in version control on purpose

`Cargo.lock` (this is an application, not a library), `web/bun.lock`, `docs/plans/`, `.env.example`, `web/public/`, all source.

## Steps

1. [x] Read current `.gitignore` (only `.env`) and list what the folder generates
2. [x] Append the new rules, keep `.env`
3. [x] Sanity check patterns by reading; no git command run

## Side Effects

- Media ignore rules (`*.mp4` and similar) also block a demo video added on purpose. Add it with `git add -f` or remove that line.
- Ignoring `web/dist/` means a fresh clone must run `bun run build` in `web/`. The README already says so.

## Notes

- Not verified with `git check-ignore`, because running git is off limits here. Patterns are standard gitignore syntax.
- Finding (read-only, from `.git/index`): the user's first commit ("Add ClipForge Rust backend with AI clip pipeline") already tracks 12,566 files under `target/`, 980 under `web/node_modules/` and 17 under `web/dist/`. `.git` is 596 MB. `.env` is NOT tracked. A `.gitignore` does not untrack existing files, so the new rules only protect future adds.
- Not pushed yet: `origin` is `https://github.com/shardreel/clipforge.git` but there are no remote-tracking refs and one commit, so history can still be fixed safely. No tracked file is over GitHub's 100 MB limit.
- Fix needs git commands, which AGENTS.md forbids the agent to run. Commands handed to the user instead (untrack, amend the first commit, expire reflog, gc). A `refs/t3/...` checkpoint ref may keep old objects reachable, so `.git` might stay large; fallback is a fresh `git init`.
