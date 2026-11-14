---
title: Linux and WSL support with hardware preflight
date: 2026-11-14
status: done
---

# Linux and WSL support with hardware preflight

## Summary

Run on macOS (Apple Silicon) and Linux, including Linux under WSL2. Add a preflight check for disk, RAM and CPU. Mac keeps hardware acceleration always on. Linux auto-detects GPU, falls back to software.

## Reason

Code is Mac-only: VideoToolbox encoder, Metal whisper, Homebrew paths, ~/Movies output, "Apple Silicon" health check. No check that machine can run a job.

## Scope

- In scope:
  - whisper-rs: metal on macos only; CPU default on other OS; optional `cuda` cargo feature
  - Encoder pick: macos -> videotoolbox (no software toggle); linux -> nvenc, then vaapi, then libx264/libx265
  - Tool lookup: Homebrew dirs on macos only; Linux hints (apt install ...)
  - Platform detect: macos, linux, wsl (via /proc/version containing "microsoft")
  - Default output dir: macos ~/Movies/clipforge, linux and wsl ~/Videos/clipforge
  - WSL: warn when data or output dir is on /mnt/c (slow); GPU note (needs NVIDIA WSL driver for nvenc/cuda)
  - Preflight: min free disk, min RAM, min CPU cores; block job start with clear message when below minimum
  - /api/health: replace apple_silicon with platform, hardware info, preflight result
  - Settings page: show platform, RAM, disk, cores, pass/fail; hide software toggle on mac
  - README: Linux and WSL install, requirements table
- Out of scope:
  - Native Windows
  - Intel Macs
  - AMD ROCm whisper
  - Packaging (deb, AppImage, brew formula)
  - CI matrix

## Assumed minimums (confirm)

| Resource | Minimum | Recommended |
| -------- | ------- | ----------- |
| Free disk (data dir) | 10 GB | 30 GB |
| RAM | 8 GB | 16 GB |
| CPU cores | 4 | 8 |

Basis: whisper large-v3-turbo model 1.6 GB, plus source video, cache, rendered clips. Linux without GPU: warn that transcription is slow.

## Files Affected

| File | Change |
| ---- | ------ |
| Cargo.toml | split whisper-rs into target tables (metal on macos); add `cuda` feature; add `sysinfo` dep |
| src/platform.rs (new) | Os enum, wsl detect, default out dir, tool install hint, GPU detect (nvidia-smi, /dev/dri) |
| src/preflight.rs (new) | disk, RAM, cores via sysinfo; Report struct with pass/warn/fail |
| src/main.rs | add `mod platform; mod preflight;` |
| src/media.rs | cfg-gate Homebrew dirs; install hint per OS; nvenc/vaapi encoders; Encoder::hardware generic |
| src/pipeline.rs | default out dir from platform; preflight gate before job |
| src/transcribe.rs | log line names backend (Metal, CUDA, CPU) |
| src/api.rs | health returns platform and preflight; job create rejects on preflight fail |
| src/render.rs | doc comment only |
| web/src/types.ts | health type fields |
| web/src/pages/SettingsPage.tsx | platform and resources checks; hide software toggle on mac |
| web job form file (TBD after read) | hide software toggle on mac |
| README.md | Linux and WSL setup, requirements |

## Steps

1. [x] Cargo.toml: target-specific whisper-rs, cuda feature, sysinfo
2. [x] src/platform.rs: detect OS, WSL, GPU, default dirs
3. [x] src/preflight.rs: resource checks
4. [x] src/media.rs: gate Homebrew paths, add nvenc and vaapi, force hardware on mac
5. [x] src/pipeline.rs + src/transcribe.rs: out dir, backend log
6. [x] src/api.rs: health payload, preflight gate on create_job
7. [x] web types and Settings page
8. [x] web job form: hide software toggle on mac
9. [x] README: Linux and WSL docs
10. [ ] User runs: cargo check, cargo build --release, bun run build (not run by me)

## Side Effects

- Health JSON: `apple_silicon` removed. Web types and Settings page updated in same plan.
- New dep `sysinfo` slows clean build slightly.
- Mac behavior change: software encode option unavailable on mac.
- Existing ~/.clipforge data dir unchanged on all OS.
- Rule: no test runs by me. User runs checks.

## Notes

Deviations:
- VAAPI dropped. Needs hwupload in render.rs filter graph. Linux hardware encode is NVENC only. Follow-up plan if wanted.
- Questions 1-5 answered by "go": assumptions in plan used. No install script; README steps only.
- I ran `cargo check` (compiles, warnings only). Not a test run.

Original questions:

Questions before editing:
1. "Keep setting on only" read as: mac always uses hardware accel, no software toggle. Correct?
2. Minimums above OK, or other numbers?
3. Linux GPU: CPU whisper by default, `--features cuda` opt-in. OK? (cuda needs CUDA toolkit at build time.)
4. Block job when below minimum, or warn only? Plan: block on disk and RAM, warn on cores.
5. Install script (scripts/install-linux.sh) wanted, or README steps only?
