# Contributing to ResoBoost

> Steady craft, clear words, and changes in small, well‑fitted pieces.

Thank you for your interest in improving **ResoBoost**. This project is a cross‑platform Tauri (Rust) + React/TypeScript app for DNS and download benchmarking. Contributions of all shapes are welcome—bug fixes, features, docs, tests, and triage.

---

## Ground Rules

* Be respectful and constructive; follow our **[Code of Conduct](./CODE_OF_CONDUCT.md)**.
* For **security issues**, never open a public issue. See **[SECURITY.md](./SECURITY.md)** or email **[ednoct@proton.me](mailto:ednoct@proton.me)**.
* Strive for small, focused pull requests with clear intent and tests where applicable.

---

## Ways to Contribute

* **Report bugs** → use **Issues » New issue » Bug report** (template provided).
* **Propose features** → use **Issues » New issue » Feature request**.
* **Improve docs** → README, guides, comments, and in‑app copy are all fair game.
* **Refactor/cleanup** → pay down tech debt in small, reviewable commits.

---

## Project Overview (at a glance)

* **Desktop app** via **Tauri 2** with a **Rust** backend
* **UI** with **React + TypeScript + Vite + Tailwind CSS**
* DNS benchmarking + per‑resolver download throughput; CSV export

*(See README for details and screenshots.)*

---

## Getting Set Up

### Prerequisites

* **Rust (stable)**
* **Node.js ≥ 18** (16+ may work, but we test with modern LTS)
* **bun** (or your preferred package manager)
* Platform-specific **Tauri** dependencies (e.g., Xcode CLT on macOS; MSVC on Windows; `libwebkit2gtk` and OpenSSL dev headers on Linux)

### Clone & Install

```bash
# clone and enter the project
git clone https://github.com/ednoct/ResoBoost.git
cd ResoBoost

# install JS deps
bun install
```

### Run the app (development)

```bash
bun run tauri dev
```

This starts the React dev server and the Tauri Rust backend.

---

## Branch, Commit, PR

1. **Fork** the repo (or create a branch if you have write access).
2. **Create a topic branch**: `feat/dnssec-reporting`, `fix/timeout-config`, etc.
3. **Commit style**: use **Conventional Commits** where possible, e.g.,

   * `feat(ui): add per-resolver Mbps chart`
   * `fix(rust): clamp jitter values to u32`
   * `docs: clarify server list import`
4. **Keep commits small and meaningful**; squash if needed before review.
5. **Open a Pull Request** and fill out the PR description (what/why/how, screenshots if UI).
6. **Link related issues** with `Fixes #123` or `Closes #123`.

### PR Checklist

* [ ] Code builds locally (`bun run build` or `bun run tauri build`) and the app runs.
* [ ] Rust code `cargo fmt` clean; no `clippy` warnings for changed lines.
* [ ] TypeScript passes type‑check for changed areas.
* [ ] Added/updated tests (where applicable) and docs.
* [ ] UI changes include before/after screenshots or brief video.
* [ ] No secrets, tokens, or private data in code or test fixtures.

> **Tip:** If the repo lacks a script you need (e.g., `typecheck`, `lint`), suggest one in your PR.

---

## Code Style & Quality

**Rust**

* Format with `cargo fmt`.
* Lint with `cargo clippy` (aim to address warnings; allow exceptions with justification).

**TypeScript/React**

* Prefer functional components and hooks.
* Keep components focused; extract helpers.
* Use explicit types where helpful; avoid `any` in new code.
* Follow project conventions for file/folder structure.

**General**

* Avoid needless allocations in hot paths; benchmark where performance is a concern.
* Log sparingly and meaningfully; prefer structured, consistent messages.
* Add comments for non‑obvious logic; keep functions small and testable.

---

## Testing

Testing is encouraged for core logic:

* **Rust**: add unit/integration tests under `src-tauri` where appropriate; consider property tests for parsers.
* **TypeScript**: add unit tests for utility functions and reducers; consider component tests for complex UI.

> If testing infrastructure is missing for a given area, propose it in a small enabling PR.

---

## Documentation

* Update **README** and in‑app help when behavior changes.
* Add usage examples for new flags, config keys, or pages.
* Keep changelogs in PR descriptions; maintainers will compile release notes.

---

## Issue Triage (maintainers)

* Label new issues with area and severity; request a minimal repro if missing.
* Close as **duplicate** with a link when appropriate; keep conversation friendly.
* Convert misfiled security issues to private channels and remove sensitive details.

---

## Security & Responsible Disclosure

Please read **[SECURITY.md](./SECURITY.md)**. Report vulnerabilities privately via the repo’s Security tab or email **[ednoct@proton.me](mailto:ednoct@proton.me)**. We follow coordinated disclosure and publish advisories with fixes or mitigations.

---

## License & Attribution

By contributing, you agree that your contributions are licensed under the project’s **MIT License**. If you add third‑party code, ensure licenses are compatible and include proper attribution.

---

## Questions?

Open a **discussion** or an **issue** (using the appropriate template). For sensitive matters, contact **[ednoct@proton.me](mailto:ednoct@proton.me)**.

> Thanks for helping ResoBoost stay sharp and sturdy. May your packets be swift, and your diffs small.
