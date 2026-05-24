---
name: jiq-release
description: Drive the full jiq release flow — branch, PR, CI, squash-merge, version bump, tag, cargo publish, and Homebrew formula update. Use when the user says "release jiq", "ship jiq", "publish a new version of jiq", "do the jiq release", "release patch/minor/major", "tag a new release", or has a TUI-validated change ready to ship.
---

# jiq-release — End-to-End Release Flow

Use this skill **only after the user has validated the change in the TUI**. The skill assumes the working tree on `main` (or the feature branch) compiles and the user is satisfied with the behavior.

## Argument

The skill accepts one optional argument indicating the version bump:

| Argument | Meaning |
|---|---|
| `patch` (default) | `X.Y.Z+1` — bug fixes, internal refactors, polish, docs |
| `minor` | `X.Y+1.0` — new user-facing features, additive changes |
| `major` | `X+1.0.0` — **breaking** changes, requires explicit user authorization |

If no argument is passed, infer from the change set per step 6. If `major` is requested without prior explicit user authorization, stop and ask before proceeding.

## Rules

- Never `--no-verify`, never force-push, never `git reset --hard`, never destructive ops.
- Commit style: lowercase Conventional Commits, single line, no body, no SIM/issue refs (`feat(...):`, `fix(...):`, `release vX.Y.Z`).
- Pre-PR gate (must all be clean): `cargo test` (full suite — unit, integration in `tests/`, doc tests), `cargo build --release`, `cargo clippy --all-targets --all-features` (zero warnings), `cargo fmt --all --check`. Do not use `--lib` here — it skips the integration tests in `tests/`.
- Major version bump: **never** without explicit user authorization. Reserved for breaking changes.
- The user deletes remote feature branches manually — do **not** pass `--delete-branch=true` on merge, and remind the user at the end.
- README is kept compact: edit it only for user-visible feature/shortcut changes. Bug fixes, refactors, performance, polish → README untouched.
- **Docs site (`docs/`) is the canonical user-facing reference and must stay in sync with every release.** For any user-visible feature, shortcut, or config change, update the relevant `docs/features/*.md` page **and** `docs/quick-reference.md` **and** `docs/configuration.md` (when applicable). The changelog page `docs/changelog.md` is mirrored from `CHANGELOG.md` on every release (step 7a). Bug fixes / internal refactors → docs untouched (same rule as README).

---

## 1. Branch + commit

```sh
git checkout -b <feat|fix>/<short-topic>
git add <files>
git commit -m "<feat|fix>(<scope>): <imperative summary>"
```

## 2. Push + open PR

```sh
git push -u origin <branch>
gh pr create --title "<same as commit>" --body "$(cat <<'EOF'
## Summary
- bullet 1
- bullet 2

## Test plan
- [x] cargo test
- [x] cargo build --release
- [x] cargo clippy --all-targets --all-features
- [x] cargo fmt --all --check
- [x] Manual TUI validation
EOF
)"
```

## 3. Wait for CI

```sh
gh pr checks <N> --watch
```

Required jobs that **must pass**: `Lint`, `Test`, `Coverage`, `plan`.

The `Release` workflow on PRs has these jobs that **skip** (expected, not failures): `announce`, `build-global-artifacts`, `build-local-artifacts`, `host`, `publish-homebrew-formula`. Those run on tag push, not on PRs.

## 4. Squash-merge

```sh
gh pr merge <N> --squash --delete-branch=false
```

`--delete-branch=false` is intentional. The user deletes the remote branch manually.

## 5. Sync main

```sh
git checkout main
git pull origin main
git branch -d <feature-branch>
```

## 6. Pick the version bump

If the user passed an argument (`patch`, `minor`, or `major`), use it. Otherwise read the current `version` in `Cargo.toml` and infer:

| Bump | When |
|---|---|
| **Patch** (`X.Y.Z+1`) | Bug fixes, internal refactors, polish, docs |
| **Minor** (`X.Y+1.0`) | New user-facing features, additive changes |
| **Major** (`X+1.0.0`) | Breaking changes — **only** with explicit user authorization |

## 7. Update `CHANGELOG.md` (always)

Under `## [Unreleased]`, add:

```markdown
## [<new version>] - <YYYY-MM-DD>

### Added | Fixed | Changed
- **<short title>** ([#NNN](https://github.com/bellicose100xp/jiq/pull/NNN)) — <one paragraph of concrete user-visible behavior, not implementation detail>.
```

Match the prose voice of recent entries — concrete, specific, no hedging. Link the PR.

## 7a. Mirror `CHANGELOG.md` to `docs/changelog.md` (always)

The docs site has a `docs/changelog.md` page that mirrors the repo's `CHANGELOG.md`. Refresh it on every release so the site stays current:

```sh
{
  printf -- '---\ntitle: Changelog\nnav_order: 7\ndescription: Release history, mirrored from CHANGELOG.md in the repo.\n---\n\n# Changelog\n{: .no_toc }\n\nThis page mirrors [`CHANGELOG.md`](https://github.com/bellicose100xp/jiq/blob/main/CHANGELOG.md) in the repository. The release skill keeps the two in sync — every release writes its entry here as well.\n\n'
  tail -n +2 CHANGELOG.md
} > docs/changelog.md
```

(`tail -n +2` strips the top `# Changelog` heading from the source so the docs page heading isn't duplicated.)

## 8. Update `README.md` and `docs/` (conditional)

Only when the change adds, removes, or modifies a **user-visible feature, keyboard shortcut, or config option**. Bug fixes / internal refactors / perf / polish → leave both untouched.

When updating, both surfaces must move together:

| Change kind | README | `docs/features/<page>.md` | `docs/quick-reference.md` | `docs/configuration.md` |
|---|---|---|---|---|
| New feature | yes | yes (new or existing page) | yes | only if it adds a config key |
| New shortcut on existing feature | yes | yes (the page that owns it) | yes | no |
| Config-only change | no | only if it changes a feature's behavior | no | yes |
| Behavior tweak users can hit | conditional | yes | only if a key changed | no |

Style: match the prose voice of existing pages — concrete, specific, no hedging. For any new feature page, use the same scaffolding as existing ones (front matter with `parent: Features`, `.io-pair` / `.tui-mockup` / `.shortcuts` helpers from `_sass/custom/custom.scss`, and a closing shortcut table).

## 9. Bump `Cargo.toml` and rebuild

```sh
# edit Cargo.toml: version = "X.Y.Z"
cargo build --release
```

Confirm `Cargo.lock` shows the new version:

```sh
grep -A1 '^name = "jiq"$' Cargo.lock
```

## 10. Commit + push to main

```sh
git add CHANGELOG.md Cargo.toml Cargo.lock docs/changelog.md
# add README.md only if you changed it
# add any docs/features/*.md, docs/quick-reference.md, docs/configuration.md you changed
git commit -m "release vX.Y.Z"
git push origin main
```

## 11. Tag + push tag

Existing tags are **lightweight** (verify once with `git cat-file -t v3.23.1` → `commit`). Match that:

```sh
git tag vX.Y.Z
git push origin vX.Y.Z
```

## 12. Publish to crates.io

```sh
cargo publish
```

Wait for `Published jiq vX.Y.Z at registry crates-io`.

## 13. Watch the GitHub Release workflow

The tag push triggers the `Release` workflow.

```sh
gh run list --limit 5            # find the new Release run for tag vX.Y.Z
gh run watch <RUN_ID> --exit-status
```

Confirm `publish-homebrew-formula` finishes successfully.

## 14. Verify the Homebrew tap

```sh
gh api repos/bellicose100xp/homebrew-tap/contents/Formula/jiq.rb
```

Decode the base64 `content` field and confirm `version "X.Y.Z"` appears. Quick alternative: convert the version to base64 (e.g. `3.23.3` → `MyAyMy4z`) and grep the response.

## 15. Final summary

Report to the user:

- PR merge commit SHA
- Release commit SHA on `main`
- Tag pushed
- crates.io published
- GitHub Release published with assets
- Homebrew tap formula updated to the new version
- **Reminder**: ask the user to delete the remote feature branch on GitHub manually (`git push origin --delete <branch>` or via the GitHub UI).
