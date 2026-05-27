---
name: jiq-release
description: Drive the full jiq release flow — branch, PR, CI, squash-merge, version bump, tag, cargo publish, and Homebrew formula update. Use when the user says "release jiq", "ship jiq", "publish a new version of jiq", "do the jiq release", "release patch/minor/major", "tag a new release", or has a TUI-validated change ready to ship.
---

# jiq-release

Run only after the user has validated the change in the TUI.

## Argument: `patch` | `minor` | `major`

- `patch` (default) — bug fixes, refactors, polish, docs
- `minor` — new user-visible features, additive changes
- `major` — breaking changes, **explicit user authorization required**

If unspecified, infer from the change set (step 6).

## Rules

- No `--no-verify`, no force-push, no `git reset --hard`, no destructive ops.
- Commit style: lowercase Conventional Commits, single line, no body, no issue refs.
- **Pre-PR gate** — must already be green from the project's pre-commit flow (`CLAUDE.md` → Pre-Commit Requirements). The skill assumes all eight steps already passed and the change is committed locally:
  1. Implementation-detail comments stripped
  2. 100% test coverage on new logic
  3. `cargo build --release` (zero warnings)
  4. User TUI validation (explicit steps; STOP-and-wait gate)
  5. `cargo clippy --all-targets --all-features -- -D warnings`
  6. `cargo fmt --all --check`
  7. `cargo build` (debug; zero warnings)
  8. `cargo test` (full suite, never `--lib`)

  If any of these are unverified, **stop and run them before branching**. Don't paper over a skipped TUI validation by jumping ahead to step 1 of this skill.
- Don't pass `--delete-branch=true` on merge; the user deletes remote branches manually. Remind them at the end.
- **User-visible feature/shortcut/config change** → update README (one line per item, no emoji, only real top-level features), `docs/features/*.md`, `docs/quick-reference.md`, `docs/configuration.md` (if config). Bug fixes / refactors / perf → leave both untouched.
- `docs/changelog.md` is mirrored from `CHANGELOG.md` every release (step 7a).

---

## 1. Sync with remote, then branch + commit

```sh
# Pull latest main first to minimize merge conflicts in the PR.
git fetch origin
git checkout main
git pull --ff-only origin main

# Branch and commit on top of fresh main.
git checkout -b <feat|fix>/<short-topic>
git add <files>
git commit -m "<feat|fix>(<scope>): <imperative summary>"
```

If the user already committed on `main` (per CLAUDE.md's pre-commit flow), reset main to origin and move the local commits to the new branch instead — never force-push `main`:

```sh
git fetch origin
git checkout -b <feat|fix>/<short-topic>      # carry local commits to feature branch
git checkout main
git reset --hard origin/main                  # OK: only resets local main, no remote impact
git checkout <feat|fix>/<short-topic>
git rebase origin/main                        # resolve conflicts locally, before PR
```

## 1a. Re-run the gate after rebase

If the rebase moved any commits, the eight pre-commit checks may no longer be green. Re-run them (CLAUDE.md → Pre-Commit Requirements). Do not push until they're all green again.

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

## 7a. Mirror `CHANGELOG.md` → `docs/changelog.md`

```sh
{
  printf -- '---\ntitle: Changelog\nnav_order: 7\ndescription: Release history, mirrored from CHANGELOG.md in the repo.\n---\n\n# Changelog\n{: .no_toc }\n\nThis page mirrors [`CHANGELOG.md`](https://github.com/bellicose100xp/jiq/blob/main/CHANGELOG.md). The release skill keeps the two in sync.\n\n'
  tail -n +2 CHANGELOG.md
} > docs/changelog.md
```

## 8. Update README + docs (conditional)

Per the Rules section: user-visible feature/shortcut/config change only. New feature pages use the same scaffolding (front matter with `parent: Features`, `.io-pair` / `.tui-mockup` / `.shortcuts` helpers from `_sass/custom/custom.scss`, closing shortcut table).

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

## 10a. Verify GitHub Pages build (block tagging until green)

```sh
gh run list --workflow=pages-build-deployment --limit 1
gh run watch <RUN_ID> --exit-status
```

On failure, extract the error:

```sh
gh run view <RUN_ID> --log 2>&1 | grep -E "Conversion error|SyntaxError|Liquid|Incompatible units" | head -20
```

Common: `Incompatible units 'rem' and 'px'` → values in `_sass/color_schemes/jiq.scss` must be `rem`. `Liquid syntax error` → wrap `{{ }}` / `{% %}` in code blocks with `{% raw %}…{% endraw %}`.

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
