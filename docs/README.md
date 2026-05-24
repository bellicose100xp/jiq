# jiq Documentation Site

This directory is the source for [the jiq documentation site](https://bellicose100xp.github.io/jiq/), built with [Jekyll](https://jekyllrb.com/) and the [just-the-docs](https://just-the-docs.com/) theme.

GitHub Pages is configured to serve from this folder on the `main` branch.

## Local preview

```bash
cd docs
bundle install
bundle exec jekyll serve --livereload
# open http://localhost:4000/jiq/
```

## Structure

- `_config.yml` — Jekyll + theme configuration.
- `index.md` — landing page.
- `getting-started.md`, `quick-reference.md`, `configuration.md`, `troubleshooting.md`, `changelog.md` — top-level pages.
- `features/` — one page per feature (results pane, autocomplete, AI, snippets, search, history, mouse, clipboard, VIM editing, tooltip).
- `_sass/custom/custom.scss` — site-specific styling (TUI mockups, IO blocks, keybind chips, drill chains).

## Editing rules

- **Always update the docs alongside any user-visible feature or shortcut change.** The `jiq-release` skill has an explicit step for this.
- Keep the **quick reference** in sync with the per-feature pages — it is a cheat sheet, not a separate source of truth.
- Visuals: prefer inline SVG or HTML mockups over external images so the site stays self-contained.
