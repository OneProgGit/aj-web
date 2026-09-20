# aj-web

Web client for `ada-judge` — port of `aj-app` (Slint) to the browser.
Stack: [Dioxus](https://dioxuslabs.com) 0.7 (WASM) + Tailwind CSS v4 + DaisyUI v5.
Shared data models come from the [`aj-models`](https://crates.io/crates/aj-models) crate.

## Prerequisites

- Rust 1.85+ with the `wasm32-unknown-unknown` target (`rustup target add wasm32-unknown-unknown`)
- [Dioxus CLI](https://dioxuslabs.com/learn/0.7/getting_started/) (`cargo install dioxus-cli`)
- Node.js (for Tailwind CSS)

## Development

```bash
npm install          # tailwindcss + daisyui
npm run watch:css    # rebuild public/tailwind.css on change (terminal 1)
dx serve             # dev server with hot reload (terminal 2)
```

> CSS classes used in Rust sources are picked up by the Tailwind scanner.
> After changing classes always rebuild CSS **and** the frontend
> (`npm run build:css` + `dx build`); `dx build` alone only copies the CSS.

## Production build

```bash
npm run build:css
dx build --release
```

## Configuration

Backend base URL is baked in at compile time (defaults to `https://aj-host.oneprog.org`):

```bash
API_BASE=https://aj-host.oneprog.org dx build --release
```

WebSocket realtime (`/contests/{id}/ws?token=`) uses the stored JWT — no cookies involved.

## Layout

- `src/pages/` — routes (home, contest + tabs, problems, users, profiles, auth)
- `src/components/` — UI cards, forms, modals, icons, markdown renderer
- `src/api/` — REST client (`reqwest`), auth, contests, problems, users
- `src/models/` — re-exports of `aj-models` + local helpers (`describe_error`, `DeletionRequest`)
- `src/state.rs` — global `STATE` signal (data, token, language)
