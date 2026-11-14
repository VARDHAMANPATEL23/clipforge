# Contributing to shardreel

Thanks for helping. Bug reports, fixes, docs and small features are all welcome.

## Before you start

- Search [existing issues](https://github.com/shardreel/clipforge/issues) first.
- For anything larger than a small fix, open an issue to discuss it before writing code.
- shardreel targets Apple Silicon Macs. Say which macOS version and chip you tested on.

## Set up

Follow **Requirements** and **Build and run** in the [README](README.md). In short:

```bash
brew install cmake yt-dlp ffmpeg-full
cd web && bun install && bun run build && cd ..
cargo run --release
```

For UI work, run `cargo run --release` and `cd web && bun run dev`.
Use `cargo check` while iterating; it is much faster than a full build.

## Making a change

1. Fork the repo and create a branch from `main` (`fix/short-name` or `feat/short-name`).
2. Keep the change focused. Unrelated cleanups belong in a separate pull request.
3. Before pushing, make sure these pass:
   ```bash
   cargo fmt --all
   cargo clippy --all-targets
   cargo test
   cd web && bun run build
   ```
4. Add or update tests when you change behavior, and update the README if usage changes.
5. Open a pull request that explains what changed and why, and how you tested it.
   Include a screenshot or short recording for UI changes.

## Commit messages

Short imperative summary, for example `Fix caption timing on HEVC output`.
Add a body when the reason is not obvious.

## Rules of the road

- Never commit API keys, `.env` files, videos or model files.
- Do not add dependencies lightly. Mention new ones in the pull request and check their
  license is MIT-compatible.
- Be respectful. Assume good intent and keep feedback about the code.

## License

By contributing, you agree that your contributions are licensed under the [MIT License](LICENSE).
