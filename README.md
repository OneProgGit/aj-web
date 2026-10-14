# aj-web

Web client for `ada-judge` — a port of `aj-app` (Slint) to the browser.

**Stack:** [Dioxus](https://dioxuslabs.com) 0.7 (WASM) + Tailwind CSS v4 + daisyUI v5.
Shared data models come from the [`aj-models`](https://crates.io/crates/aj-models) crate.

## Prerequisites

- Rust 1.85+ with the `wasm32-unknown-unknown` target:
  ```bash
  rustup target add wasm32-unknown-unknown
  ```
- [Dioxus CLI](https://dioxuslabs.com/learn/0.7/getting_started/):
  ```bash
  cargo install dioxus-cli
  ```
- Node.js (used to build the CSS only)

## Development

```bash
npm install          # tailwindcss + daisyui
npm run watch:css    # rebuild public/tailwind.css on change (terminal 1)
dx serve             # dev server with hot reload (terminal 2)
```

Tailwind scans the Rust sources for class names, so after changing any class you
must rebuild **both** the CSS and the frontend:

```bash
npm run build:css && dx build
```

`dx build` on its own only copies the already-built `public/tailwind.css` into
the output directory.

## Production build

```bash
npm run build:css
dx build --release
```

## Configuration

The backend base URL is baked into the wasm at compile time and defaults to
`https://aj-host.oneprog.org`:

```bash
API_BASE=https://aj-host.oneprog.org dx build --release
```

Realtime updates use a WebSocket at `/contests/{id}/ws?token={JWT}` — the stored
JWT is passed as a query parameter, no cookies are involved.

## Styling

Plain daisyUI — no component library, no runtime style copying. Everything is
utility classes plus a few global rules in `src/input.css`:

- two custom themes are defined via `@plugin "daisyui/theme"`; built-in themes
  are disabled (`themes: false`):

  | Theme         | Applied by                        |
  | ------------- | --------------------------------- |
  | `violet`      | `:root` (default, light)          |
  | `violet-dark` | `prefers-color-scheme: dark`      |

  Both are also selectable explicitly with `data-theme="violet"` /
  `data-theme="violet-dark"`. The palette is violet-leaning (OKLCH hue
  300–320) with deliberately low chroma (0.004–0.15) so it stays calm on
  large surfaces. Every foreground/background pair clears WCAG AA (≥ 4.5:1)
  for body text — the lightest/darkest pairs sit around 12–15:1.
- `@theme` sets JetBrains Mono as the sans/mono font;
- `button.btn` gets a transparent border;
- result tables are centre-aligned with a sticky header;
- `.md-body` styles rendered markdown: headings, lists, quotes, tables, code
  blocks and the `.md-copy-btn` copy button.

To change the palette, edit the two `@plugin "daisyui/theme"` blocks. Keep the
foreground/background pairs above 4.5:1 — the values are in OKLCH, where `L` is
lightness and `C` is chroma.

## Routes

| Path                    | Page                |
| ----------------------- | ------------------- |
| `/`                     | welcome / redirect  |
| `/login`, `/register`   | auth                |
| `/home`                 | contest list        |
| `/contest/:contest_id`  | contest with tabs   |
| `/problems`, `/users`   | lists               |
| `/account`              | private profile     |
| `/user/:user_id`        | public profile      |
| `/user/:user_id/private`| private profile     |

Contest pages keep the selected tab in `localStorage`; the URL is left untouched,
so it resets on reload while the tab is restored.

## Layout

- `src/pages/` — one file per route (contest tabs, forms, modals)
- `src/components/` — reusable UI: cards, forms, markdown renderer, icons
- `src/api/` — REST client (`reqwest`): auth, contests, problems, users
- `src/models/` — re-exports of `aj-models` plus local types (`DeletionRequest`)
- `src/state.rs` — global `STATE` signal (data, token, language)
- `src/i18n.rs` — `tr(lang, "ru", "en")` translation helper
- `src/alerts.rs` — toast notifications; `state::show_error` swallows 403s,
  since "no access" is a normal state rather than an error

## Notes

- Do not run `dx fmt`: it mis-parses parts of this `rsx!` code and can corrupt
  files (dropped `;`, duplicated lines). Use `cargo fmt` for Rust code.
- `cargo clippy --all-targets` is expected to be clean; note that
  `cargo clippy --fix` will happily delete helpers it considers unused
  (e.g. `state::show_error`) if their call sites are temporarily gone.
