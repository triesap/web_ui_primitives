# web_ui_primitives

Headless, accessible UI primitives for Rust web frameworks.

## Crates

- `web_ui_primitives_core`: framework-neutral interaction models and utilities.
- `web_ui_primitives_leptos`: Leptos bindings for DOM attributes, events, focus, overlays, presence, modal hiding, and scroll lock.
- `web_ui_primitives`: umbrella crate with feature-gated adapters.

## Install

Use the umbrella crate when you want one dependency with feature-gated adapters:

```toml
[dependencies]
web_ui_primitives = { version = "0.2.0", features = ["csr"] }
```

Use the adapter crate directly when you want explicit control over the dependency graph:

```toml
[dependencies]
web_ui_primitives_core = "0.2.0"
web_ui_primitives_leptos = { version = "0.2.0", features = ["csr"] }
```

## Features

- `core` (default): core interaction models and low-level interaction utilities
- `leptos`: Leptos bindings (depends on `web_ui_primitives_core`)
- `csr`, `hydrate`, `ssr`: mutually exclusive final-consumer render modes that
  enable and forward the Leptos adapter mode

The Leptos adapter exposes Presence ABI v2. Exit lifecycles account for all
root transition and animation tracks, cancel events, zero/reduced motion,
reopen races, and computed-style changes while an exit is in flight.

## Leptos render modes

The Leptos adapter is render-mode neutral. Final applications select CSR,
hydration, or SSR; the adapter must not force a delivery mode into a shared
dependency graph.

Browser effects remain browser-owned. Native SSR rendering must not create
process-global modal, scroll-lock, focus, dismissal, portal, or placement
state shared between requests. Portal markup, generated IDs, and placement
bindings used during SSR must hydrate without duplication or DOM identity
drift. A strict placement adapter supports CSP deployments that prohibit
inline style attributes while the existing element-style adapter remains
available to qualified CSR consumers.

## Testing

Browser-only Leptos behavior is covered in
`crates/web_ui_primitives_leptos/tests/browser.rs`. Run host tests with
`cargo test`. Run the browser harness with:

```bash
wasm-pack test --headless --chrome crates/web_ui_primitives_leptos
```

## Contributing

See `CONTRIBUTING.md`.

## License

MIT OR Apache-2.0. See `LICENSE-MIT` and `LICENSE-APACHE`.
