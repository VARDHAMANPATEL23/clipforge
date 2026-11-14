# Security Policy

## Supported versions

Only the latest commit on `main` and the latest release receive security fixes.

## Reporting a vulnerability

Please do **not** open a public issue for security problems.

Report privately through GitHub:
**Security tab > Report a vulnerability** at
<https://github.com/shardreel/clipforge/security/advisories/new>.

Include:

- what the issue is and its impact
- steps or a proof of concept to reproduce it
- affected version or commit, and your macOS version

You can expect an acknowledgement within 7 days and a status update within 30 days.
Please give us reasonable time to fix the issue before disclosing it publicly. We will credit
you in the release notes unless you prefer to stay anonymous.

## Scope

shardreel is a local app. The web UI and API listen on `127.0.0.1` only. Things we care about:

- the local server being reachable from other machines or from malicious web pages
  (for example CSRF or DNS rebinding against the local API)
- path traversal or arbitrary file read/write through uploads, local paths or URLs
- command injection through filenames, URLs or settings passed to ffmpeg or yt-dlp
- leaking API keys from `~/.clipforge/.env` in logs, responses or clip metadata

Out of scope: vulnerabilities in ffmpeg, yt-dlp, whisper.cpp or other dependencies
(report those upstream, but do tell us so we can update), and issues that need an attacker
who already has full access to your user account.

## Keeping your keys safe

Store API keys only in `~/.clipforge/.env` with `chmod 600`, never in the repo.
