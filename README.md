# headless-primitives

Headless, accessible UI primitives for Rust web frameworks.

## Goals

- Provide framework-agnostic, headless UI primitives with strong accessibility defaults.
- Offer thin framework bindings that attach behavior to any markup.
- Stay unstyled by default so applications control their own design system.

## Crates

- `headless-primitives-core` (no_std): state machines and models (collapsible, dialog, tabs, roving focus, typeahead, ids).
- `headless-primitives-leptos`: Leptos bindings for attaching attributes/events and behavior (focus scope, dismissible layer, presence, portal, modal aria-hidden, scroll lock).
- `ui-primitives*`: compatibility shims that re-export the renamed crates during the transition.

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
use headless_primitives::leptos::{builders::collapsible_trigger_attrs, use_dom_bindings};

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

- `core` (default): core models and state machines
- `leptos`: Leptos bindings (depends on `headless-primitives-core`)

## Migration

- `Primitive*` names remain as deprecated aliases for the new `Dom*` and `BoundElement` surface.
- `use_primitive` remains as a deprecated alias for `use_dom_bindings`.
- `Dismissable*` names remain as deprecated aliases for `Dismissible*`.

## Contributing

See `CONTRIBUTING.md`.

## License

Unlicense — see `LICENSE`.
