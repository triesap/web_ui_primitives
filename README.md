# ui-primitives

Headless, accessible UI primitives for Rust web frameworks.

## Goals

- Provide framework-agnostic, headless UI primitives with strong accessibility defaults.
- Offer thin framework bindings that attach behavior to any markup.
- Stay unstyled by default so applications control their own design system.

## How it works

Builders return **elements**, **states**, and **options**. Elements can be attached to any DOM node in user markup, while states/options are signals for reactive control.

```rust
let collapsible = create_collapsible(CollapsibleProps::default());
let root = collapsible.elements.root;
let trigger = collapsible.elements.trigger;
let content = collapsible.elements.content;

view! {
    <div node_ref=use_primitive(root)>
        <button node_ref=use_primitive(trigger)>"Toggle"</button>
        <div node_ref=use_primitive(content)>"Content"</div>
    </div>
}
```

## Contributing

See `CONTRIBUTING.md`.

## License

Unlicense — see `LICENSE`.
