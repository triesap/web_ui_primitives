# headless-primitives

Headless, accessible UI primitives for Rust web frameworks.

## Goals

- Provide framework-agnostic, headless UI primitives with strong accessibility defaults.
- Offer thin framework bindings that attach behavior to any markup.
- Stay unstyled by default so applications control their own design system.

## Crates

- `headless-primitives-core` (no_std): primary interaction models (`collapsible`, `dialog`, `tabs`) plus low-level interaction utilities (`roving_focus`, `typeahead`).
- `headless-primitives-leptos`: Leptos bindings for attaching attributes/events and behavior (focus scope, dismissible layer, presence, portal, modal aria-hidden, scroll lock).

## How it works

Core models expose state. Framework bindings expose helpers that generate DOM attributes and event bindings, plus behavior primitives such as focus scopes and dismissible layers.

## Using the crates

Use the umbrella crate when you want one dependency with feature-gated adapters:

```toml
[dependencies]
headless-primitives = { version = "0.1.0", features = ["leptos"] }
leptos = { version = "0.8.16", features = ["csr"] }
```

Use the adapter crate directly when you want explicit control over the dependency graph:

```toml
[dependencies]
headless-primitives-core = "0.1.0"
headless-primitives-leptos = "0.1.0"
leptos = { version = "0.8.16", features = ["csr"] }
```

Example (Leptos):

```rust
use leptos::html;
use leptos::prelude::*;
use headless_primitives::core::collapsible::CollapsibleModel;
use headless_primitives::leptos::{attrs::collapsible_trigger_attrs, use_dom_bindings};

let model = RwSignal::new(CollapsibleModel::new(false));
let attrs = Signal::derive(move || collapsible_trigger_attrs(&model.get(), Some("details")));
let bindings = use_dom_bindings::<html::Button>(attrs, vec![]);

view! {
    <button node_ref=bindings.node_ref()>
        "Toggle"
    </button>
}
```

## Features

- `core` (default): core interaction models and low-level interaction utilities
- `leptos`: Leptos bindings (depends on `headless-primitives-core`)

## Browser testing

Browser-only Leptos behavior is covered in
`crates/headless-primitives-leptos/tests/browser.rs`.

`cargo test` compiles that test target on host builds, but it only executes in
a browser runtime. Run the browser harness with:

```bash
wasm-pack test --headless --chrome crates/headless-primitives-leptos
```

If ChromeDriver needs explicit browser capabilities, use
`webdriver.json.example` as the template for a local `webdriver.json` in the
workspace root, or point `WASM_BINDGEN_TEST_WEBDRIVER_JSON` at a custom file.
Set `goog:chromeOptions.binary` when Chrome or Chromium is installed outside
the default discovery path.

If your OS rejects a local browser build or ChromeDriver fails during browser
startup or teardown, verify that the browser binary and driver are compatible
and point ChromeDriver at a known-good Chrome or Chromium install.

## Contributing

See `CONTRIBUTING.md`.

## License

Unlicense — see `LICENSE`.
