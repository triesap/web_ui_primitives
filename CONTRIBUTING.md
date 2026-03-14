# Contributing

Thanks for your interest in contributing to headless-primitives.

## Ways to help

- Report bugs and regressions
- Improve documentation and examples
- Add new attribute helpers or framework bindings
- Expand accessibility and keyboard coverage

## Development setup

This repository is a Rust workspace. Typical tasks:

- `cargo fmt`
- `cargo test`
- `wasm-pack test --headless --chrome crates/headless-primitives-leptos`

## Browser tests

Browser-targeted Leptos tests live in
`crates/headless-primitives-leptos/tests/browser.rs`.

Use the `wasm-pack test --headless --chrome crates/headless-primitives-leptos`
command when you change dismissible, portal, focus, modal, presence, or other
browser-only behavior.

`cargo test` is still the main host-side check, but it does not execute the
wasm browser harness.

If ChromeDriver cannot find the browser you want to use, start from
`webdriver.json.example` and point `WASM_BINDGEN_TEST_WEBDRIVER_JSON` at a
local override file. Set `goog:chromeOptions.binary` when your Chrome or
Chromium binary is not discoverable through the default webdriver setup.

## Pull request checklist

- Keep changes focused and well-scoped
- Add or update tests when behavior changes
- Keep public APIs documented
- Avoid introducing new unsafe code

## Code style

- Use idiomatic Rust
- Prefer small, composable helpers
- Favor clear, explicit APIs over cleverness

## Accessibility

All components should follow WAI-ARIA APG patterns where applicable.
If behavior changes affect keyboard interaction or focus, include tests.

## License

By contributing, you agree that your contributions are released under the
project license (Unlicense).
