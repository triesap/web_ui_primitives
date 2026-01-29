# ui-primitives

Headless, accessible UI primitives for Rust web frameworks.

## Goals

- Provide framework-agnostic, headless UI primitives with strong accessibility defaults.
- Offer thin framework bindings that attach behavior to any markup.
- Stay unstyled by default so applications control their own design system.

## Crates

- `ui-primitives-core` (no_std): state machines and models (collapsible, dialog, tabs, roving focus, typeahead, ids).
- `ui-primitives-leptos`: Leptos bindings for attaching attributes/events and behavior (focus scope, dismissable layer, presence, portal, modal aria-hidden, scroll lock).

## How it works

Core models expose state. Framework bindings expose helpers that generate DOM attributes/events, plus primitives for behavior like focus or dismissable layers.

Example (Leptos):

```rust
use leptos::prelude::*;
use ui_primitives::core::collapsible::CollapsibleModel;
use ui_primitives::leptos::{use_primitive, builders::collapsible_trigger_attrs};

let model = RwSignal::new(CollapsibleModel::new(false));
let attrs = Signal::derive(move || collapsible_trigger_attrs(&model.get(), None));

view! {
    <button node_ref=use_primitive(attrs, vec![])>
        "Toggle"
    </button>
}
```

## Features

- `core` (default): core models and state machines
- `leptos`: Leptos bindings (depends on `ui-primitives-core`)

## Contributing

See `CONTRIBUTING.md`.

## License

Unlicense — see `LICENSE`.
