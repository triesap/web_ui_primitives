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
web_ui_primitives = { version = "0.1.0", features = ["leptos"] }
leptos = { version = "0.9.0-alpha", features = ["csr"] }
```

Use the adapter crate directly when you want explicit control over the dependency graph:

```toml
[dependencies]
web_ui_primitives_core = "0.1.0"
web_ui_primitives_leptos = "0.1.0"
leptos = { version = "0.9.0-alpha", features = ["csr"] }
```

Use adapter crates directly when you want a narrower dependency graph:

```toml
[dependencies]
web_ui_primitives_core = "0.1.0"
web_ui_primitives_leptos = "0.1.0"
leptos = { version = "0.9.0-alpha", features = ["csr"] }
```

## Features

- `core` (default): core interaction models and low-level interaction utilities
- `leptos`: Leptos bindings (depends on `web_ui_primitives_core`)

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
