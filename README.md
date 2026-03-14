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

Core models expose state. Framework bindings expose helpers that generate DOM attributes/events, plus primitives for behavior like focus or dismissable layers.

Example (Leptos):

```rust
use leptos::prelude::*;
use headless_primitives::core::collapsible::CollapsibleModel;
use headless_primitives::leptos::{builders::collapsible_trigger_attrs, use_dom_bindings};

let model = RwSignal::new(CollapsibleModel::new(false));
let attrs = Signal::derive(move || collapsible_trigger_attrs(&model.get(), None));

view! {
    <button node_ref=use_dom_bindings(attrs, vec![])>
        "Toggle"
    </button>
}
```

## Features

- `core` (default): core models and state machines
- `leptos`: Leptos bindings (depends on `headless-primitives-core`)

## Contributing

See `CONTRIBUTING.md`.

## License

Unlicense — see `LICENSE`.
