#![cfg(target_arch = "wasm32")]

use headless_primitives_leptos::{
    DismissibleLayer, DismissibleReason, FocusScope, Portal, Presence, modal_hide_siblings,
    scroll_lock_acquire, scroll_lock_release,
};
use gloo_timers::future::TimeoutFuture;
use leptos::mount::mount_to;
use leptos::prelude::*;
use std::sync::{Arc, Mutex};
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

fn document() -> web_sys::Document {
    window().document().expect("document")
}

fn window() -> web_sys::Window {
    web_sys::window().expect("window")
}

fn body() -> web_sys::HtmlElement {
    document().body().expect("body")
}

fn append_div(id: &str) -> web_sys::HtmlElement {
    let element = document()
        .create_element("div")
        .expect("create div")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("html element");
    element.set_id(id);
    body().append_child(&element).expect("append div");
    element
}

fn append_button(id: &str) -> web_sys::HtmlElement {
    let element = document()
        .create_element("button")
        .expect("create button")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("button html element");
    element.set_id(id);
    body().append_child(&element).expect("append button");
    element
}

fn append_child_div(parent: &web_sys::HtmlElement, id: &str) -> web_sys::HtmlElement {
    let element = document()
        .create_element("div")
        .expect("create child div")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("child html element");
    element.set_id(id);
    parent.append_child(&element).expect("append child div");
    element
}

fn remove_from_body(element: &web_sys::HtmlElement) {
    body().remove_child(element).expect("remove element");
}

fn dispatch_pointer_down(target: &web_sys::HtmlElement) {
    let init = web_sys::PointerEventInit::new();
    init.set_bubbles(true);
    init.set_composed(true);
    let event = web_sys::PointerEvent::new_with_event_init_dict("pointerdown", &init)
        .expect("pointer event");
    target.dispatch_event(&event).expect("dispatch pointerdown");
}

fn dispatch_keydown(
    target: &web_sys::HtmlElement,
    key: &str,
    shift: bool,
) -> web_sys::KeyboardEvent {
    let init = web_sys::KeyboardEventInit::new();
    init.set_bubbles(true);
    init.set_cancelable(true);
    init.set_key(key);
    init.set_shift_key(shift);
    let event = web_sys::KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init)
        .expect("keyboard event");
    target.dispatch_event(&event).expect("dispatch keydown");
    event
}

fn dispatch_tab_keydown(target: &web_sys::HtmlElement, shift: bool) -> web_sys::KeyboardEvent {
    dispatch_keydown(target, "Tab", shift)
}

fn dispatch_escape_keydown(target: &web_sys::HtmlElement) -> web_sys::KeyboardEvent {
    dispatch_keydown(target, "Escape", false)
}

fn dispatch_transition_end(target: &web_sys::HtmlElement, bubbles: bool) {
    let init = web_sys::TransitionEventInit::new();
    init.set_bubbles(bubbles);
    let event =
        web_sys::TransitionEvent::new_with_event_init_dict("transitionend", &init.into())
            .expect("transition event");
    target
        .dispatch_event(&event)
        .expect("dispatch transitionend");
}

fn dispatch_animation_end(target: &web_sys::HtmlElement, bubbles: bool) {
    let init = web_sys::AnimationEventInit::new();
    init.set_bubbles(bubbles);
    let event = web_sys::AnimationEvent::new_with_event_init_dict("animationend", &init.into())
        .expect("animation event");
    target
        .dispatch_event(&event)
        .expect("dispatch animationend");
}

fn attr(element: &web_sys::HtmlElement, name: &str) -> Option<String> {
    element.get_attribute(name)
}

fn active_id() -> Option<String> {
    document().active_element().map(|element| element.id())
}

fn active_element() -> Option<web_sys::Element> {
    document().active_element()
}

fn html_element_by_id(id: &str) -> web_sys::HtmlElement {
    document()
        .get_element_by_id(id)
        .unwrap_or_else(|| panic!("missing element: {id}"))
        .dyn_into::<web_sys::HtmlElement>()
        .expect("html element")
}

fn body_style(name: &str) -> String {
    body()
        .style()
        .get_property_value(name)
        .unwrap_or_else(|_| panic!("missing body style: {name}"))
}

fn set_body_style(name: &str, value: &str) {
    body()
        .style()
        .set_property(name, value)
        .unwrap_or_else(|_| panic!("set body style: {name}"));
}

fn scroll_y() -> f64 {
    window().scroll_y().expect("read scroll y")
}

#[wasm_bindgen_test]
fn dismissible_layer_ignores_inside_pointerdown_and_reports_outside_pointerdown() {
    let host = append_div("dismissible-host");
    let outside = append_div("dismissible-outside");
    let dismissals: Arc<Mutex<Vec<DismissibleReason>>> = Arc::new(Mutex::new(Vec::new()));
    let dismissals_handle = Arc::clone(&dismissals);

    let mount = mount_to(host.clone(), move || {
        let dismissals = Arc::clone(&dismissals_handle);
        view! {
            <DismissibleLayer on_dismiss=Callback::new(move |reason| {
                dismissals.lock().expect("dismissals lock").push(reason);
            })>
                <button id="dismissible-inside">"Inside"</button>
            </DismissibleLayer>
        }
    });

    let inside = document()
        .get_element_by_id("dismissible-inside")
        .expect("inside button")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("inside html element");

    dispatch_pointer_down(&inside);
    assert!(dismissals.lock().expect("dismissals lock").is_empty());

    dispatch_pointer_down(&outside);
    assert_eq!(
        dismissals.lock().expect("dismissals lock").as_slice(),
        &[DismissibleReason::PointerDownOutside]
    );

    drop(mount);
    remove_from_body(&host);
    remove_from_body(&outside);
}

#[wasm_bindgen_test]
fn dismissible_layer_reports_escape_to_callback_and_dismiss_reason() {
    let host = append_div("dismissible-escape-host");
    let dismissals: Arc<Mutex<Vec<DismissibleReason>>> = Arc::new(Mutex::new(Vec::new()));
    let dismissals_handle = Arc::clone(&dismissals);
    let escape_keys: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let escape_keys_handle = Arc::clone(&escape_keys);

    let mount = mount_to(host.clone(), move || {
        let dismissals = Arc::clone(&dismissals_handle);
        let escape_keys = Arc::clone(&escape_keys_handle);

        view! {
            <DismissibleLayer
                on_dismiss=Callback::new(move |reason| {
                    dismissals.lock().expect("dismissals lock").push(reason);
                })
                on_escape_key_down=Callback::new(move |event: leptos::ev::KeyboardEvent| {
                    escape_keys
                        .lock()
                        .expect("escape keys lock")
                        .push(event.key());
                })
            >
                <button id="dismissible-escape-inside">"Inside"</button>
            </DismissibleLayer>
        }
    });

    let inside = html_element_by_id("dismissible-escape-inside");
    let escape = dispatch_escape_keydown(&inside);

    assert!(!escape.default_prevented());
    assert_eq!(
        dismissals.lock().expect("dismissals lock").as_slice(),
        &[DismissibleReason::Escape]
    );
    assert_eq!(
        escape_keys.lock().expect("escape keys lock").as_slice(),
        &["Escape".to_string()]
    );

    drop(mount);
    remove_from_body(&host);
}

#[wasm_bindgen_test]
fn dismissible_layer_handles_escape_and_pointer_outside_in_live_dom() {
    let host = append_div("dismissible-combined-host");
    let outside = append_div("dismissible-combined-outside");
    let dismissals: Arc<Mutex<Vec<DismissibleReason>>> = Arc::new(Mutex::new(Vec::new()));
    let dismissals_handle = Arc::clone(&dismissals);

    let mount = mount_to(host.clone(), move || {
        let dismissals = Arc::clone(&dismissals_handle);

        view! {
            <DismissibleLayer on_dismiss=Callback::new(move |reason| {
                dismissals.lock().expect("dismissals lock").push(reason);
            })>
                <button id="dismissible-combined-inside">"Inside"</button>
            </DismissibleLayer>
        }
    });

    let inside = html_element_by_id("dismissible-combined-inside");

    dispatch_escape_keydown(&inside);
    dispatch_pointer_down(&outside);

    assert_eq!(
        dismissals.lock().expect("dismissals lock").as_slice(),
        &[
            DismissibleReason::Escape,
            DismissibleReason::PointerDownOutside
        ]
    );

    drop(mount);
    remove_from_body(&host);
    remove_from_body(&outside);
}

#[wasm_bindgen_test]
fn dismissible_layer_ignores_focus_moves_within_the_layer() {
    let host = append_div("dismissible-focus-inside-host");
    let dismissals: Arc<Mutex<Vec<DismissibleReason>>> = Arc::new(Mutex::new(Vec::new()));
    let dismissals_handle = Arc::clone(&dismissals);
    let focus_targets: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let focus_targets_handle = Arc::clone(&focus_targets);

    let mount = mount_to(host.clone(), move || {
        let dismissals = Arc::clone(&dismissals_handle);
        let focus_targets = Arc::clone(&focus_targets_handle);

        view! {
            <DismissibleLayer
                on_dismiss=Callback::new(move |reason| {
                    dismissals.lock().expect("dismissals lock").push(reason);
                })
                on_focus_outside=Callback::new(move |event: leptos::ev::FocusEvent| {
                    let target_id = event
                        .target()
                        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                        .map(|element| element.id())
                        .unwrap_or_default();
                    focus_targets
                        .lock()
                        .expect("focus targets lock")
                        .push(target_id);
                })
            >
                <button id="dismissible-focus-first">"First"</button>
                <button id="dismissible-focus-second">"Second"</button>
            </DismissibleLayer>
        }
    });

    let first = html_element_by_id("dismissible-focus-first");
    let second = html_element_by_id("dismissible-focus-second");

    first.focus().expect("focus first");
    second.focus().expect("focus second");

    assert!(dismissals.lock().expect("dismissals lock").is_empty());
    assert!(
        focus_targets
            .lock()
            .expect("focus targets lock")
            .is_empty()
    );

    drop(mount);
    remove_from_body(&host);
}

#[wasm_bindgen_test]
fn dismissible_layer_reports_focus_outside_via_callback_and_reason() {
    let host = append_div("dismissible-focus-outside-host");
    let outside = append_button("dismissible-focus-outside-target");
    let dismissals: Arc<Mutex<Vec<DismissibleReason>>> = Arc::new(Mutex::new(Vec::new()));
    let dismissals_handle = Arc::clone(&dismissals);
    let focus_targets: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let focus_targets_handle = Arc::clone(&focus_targets);

    let mount = mount_to(host.clone(), move || {
        let dismissals = Arc::clone(&dismissals_handle);
        let focus_targets = Arc::clone(&focus_targets_handle);

        view! {
            <DismissibleLayer
                on_dismiss=Callback::new(move |reason| {
                    dismissals.lock().expect("dismissals lock").push(reason);
                })
                on_focus_outside=Callback::new(move |event: leptos::ev::FocusEvent| {
                    let target_id = event
                        .target()
                        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                        .map(|element| element.id())
                        .unwrap_or_default();
                    focus_targets
                        .lock()
                        .expect("focus targets lock")
                        .push(target_id);
                })
            >
                <button id="dismissible-focus-inside">"Inside"</button>
            </DismissibleLayer>
        }
    });

    let inside = html_element_by_id("dismissible-focus-inside");

    inside.focus().expect("focus inside");
    assert!(dismissals.lock().expect("dismissals lock").is_empty());
    assert!(
        focus_targets
            .lock()
            .expect("focus targets lock")
            .is_empty()
    );

    outside.focus().expect("focus outside");

    assert_eq!(
        dismissals.lock().expect("dismissals lock").as_slice(),
        &[DismissibleReason::FocusOutside]
    );
    assert_eq!(
        focus_targets.lock().expect("focus targets lock").as_slice(),
        &["dismissible-focus-outside-target".to_string()]
    );

    drop(mount);
    remove_from_body(&host);
    remove_from_body(&outside);
}

#[wasm_bindgen_test]
fn dismissible_layer_suppresses_pointer_outside_when_disabled() {
    let host = append_div("dismissible-pointer-disabled-host");
    let outside = append_div("dismissible-pointer-disabled-outside");
    let dismissals: Arc<Mutex<Vec<DismissibleReason>>> = Arc::new(Mutex::new(Vec::new()));
    let dismissals_handle = Arc::clone(&dismissals);
    let pointer_targets: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let pointer_targets_handle = Arc::clone(&pointer_targets);

    let mount = mount_to(host.clone(), move || {
        let dismissals = Arc::clone(&dismissals_handle);
        let pointer_targets = Arc::clone(&pointer_targets_handle);

        view! {
            <DismissibleLayer
                disable_pointer_down_outside_dismiss=true
                on_dismiss=Callback::new(move |reason| {
                    dismissals.lock().expect("dismissals lock").push(reason);
                })
                on_pointer_down_outside=Callback::new(move |event: leptos::ev::PointerEvent| {
                    let target_id = event
                        .target()
                        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                        .map(|element| element.id())
                        .unwrap_or_default();
                    pointer_targets
                        .lock()
                        .expect("pointer targets lock")
                        .push(target_id);
                })
            >
                <button id="dismissible-pointer-disabled-inside">"Inside"</button>
            </DismissibleLayer>
        }
    });

    dispatch_pointer_down(&outside);

    assert!(dismissals.lock().expect("dismissals lock").is_empty());
    assert!(
        pointer_targets
            .lock()
            .expect("pointer targets lock")
            .is_empty()
    );

    drop(mount);
    remove_from_body(&host);
    remove_from_body(&outside);
}

#[wasm_bindgen_test]
fn dismissible_layer_keeps_escape_and_focus_outside_active_when_pointer_dismiss_is_disabled() {
    let host = append_div("dismissible-pointer-disabled-combined-host");
    let outside = append_button("dismissible-pointer-disabled-focus-target");
    let dismissals: Arc<Mutex<Vec<DismissibleReason>>> = Arc::new(Mutex::new(Vec::new()));
    let dismissals_handle = Arc::clone(&dismissals);
    let focus_targets: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let focus_targets_handle = Arc::clone(&focus_targets);

    let mount = mount_to(host.clone(), move || {
        let dismissals = Arc::clone(&dismissals_handle);
        let focus_targets = Arc::clone(&focus_targets_handle);

        view! {
            <DismissibleLayer
                disable_pointer_down_outside_dismiss=true
                on_dismiss=Callback::new(move |reason| {
                    dismissals.lock().expect("dismissals lock").push(reason);
                })
                on_focus_outside=Callback::new(move |event: leptos::ev::FocusEvent| {
                    let target_id = event
                        .target()
                        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                        .map(|element| element.id())
                        .unwrap_or_default();
                    focus_targets
                        .lock()
                        .expect("focus targets lock")
                        .push(target_id);
                })
            >
                <button id="dismissible-pointer-disabled-combined-inside">"Inside"</button>
            </DismissibleLayer>
        }
    });

    let inside = html_element_by_id("dismissible-pointer-disabled-combined-inside");

    inside.focus().expect("focus inside");
    dispatch_escape_keydown(&inside);
    dispatch_pointer_down(&outside);
    outside.focus().expect("focus outside");

    assert_eq!(
        dismissals.lock().expect("dismissals lock").as_slice(),
        &[DismissibleReason::Escape, DismissibleReason::FocusOutside]
    );
    assert_eq!(
        focus_targets.lock().expect("focus targets lock").as_slice(),
        &["dismissible-pointer-disabled-focus-target".to_string()]
    );

    drop(mount);
    remove_from_body(&host);
    remove_from_body(&outside);
}

#[wasm_bindgen_test]
fn dismissible_layer_routes_pointer_outside_only_to_matching_callbacks() {
    let host = append_div("dismissible-callback-pointer-host");
    let outside = append_div("dismissible-callback-pointer-outside");
    let dismissals: Arc<Mutex<Vec<DismissibleReason>>> = Arc::new(Mutex::new(Vec::new()));
    let dismissals_handle = Arc::clone(&dismissals);
    let pointer_targets: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let pointer_targets_handle = Arc::clone(&pointer_targets);
    let focus_targets: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let focus_targets_handle = Arc::clone(&focus_targets);
    let escape_keys: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let escape_keys_handle = Arc::clone(&escape_keys);

    let mount = mount_to(host.clone(), move || {
        let dismissals = Arc::clone(&dismissals_handle);
        let pointer_targets = Arc::clone(&pointer_targets_handle);
        let focus_targets = Arc::clone(&focus_targets_handle);
        let escape_keys = Arc::clone(&escape_keys_handle);

        view! {
            <DismissibleLayer
                on_dismiss=Callback::new(move |reason| {
                    dismissals.lock().expect("dismissals lock").push(reason);
                })
                on_pointer_down_outside=Callback::new(move |event: leptos::ev::PointerEvent| {
                    let target_id = event
                        .target()
                        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                        .map(|element| element.id())
                        .unwrap_or_default();
                    pointer_targets
                        .lock()
                        .expect("pointer targets lock")
                        .push(target_id);
                })
                on_focus_outside=Callback::new(move |event: leptos::ev::FocusEvent| {
                    let target_id = event
                        .target()
                        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                        .map(|element| element.id())
                        .unwrap_or_default();
                    focus_targets
                        .lock()
                        .expect("focus targets lock")
                        .push(target_id);
                })
                on_escape_key_down=Callback::new(move |event: leptos::ev::KeyboardEvent| {
                    escape_keys
                        .lock()
                        .expect("escape keys lock")
                        .push(event.key());
                })
            >
                <button id="dismissible-callback-pointer-inside">"Inside"</button>
            </DismissibleLayer>
        }
    });

    dispatch_pointer_down(&outside);

    assert_eq!(
        dismissals.lock().expect("dismissals lock").as_slice(),
        &[DismissibleReason::PointerDownOutside]
    );
    assert_eq!(
        pointer_targets
            .lock()
            .expect("pointer targets lock")
            .as_slice(),
        &["dismissible-callback-pointer-outside".to_string()]
    );
    assert!(focus_targets.lock().expect("focus targets lock").is_empty());
    assert!(escape_keys.lock().expect("escape keys lock").is_empty());

    drop(mount);
    remove_from_body(&host);
    remove_from_body(&outside);
}

#[wasm_bindgen_test]
fn dismissible_layer_routes_focus_and_escape_only_to_matching_callbacks() {
    let host = append_div("dismissible-callback-focus-host");
    let outside = append_button("dismissible-callback-focus-outside");
    let dismissals: Arc<Mutex<Vec<DismissibleReason>>> = Arc::new(Mutex::new(Vec::new()));
    let dismissals_handle = Arc::clone(&dismissals);
    let pointer_targets: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let pointer_targets_handle = Arc::clone(&pointer_targets);
    let focus_targets: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let focus_targets_handle = Arc::clone(&focus_targets);
    let escape_keys: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let escape_keys_handle = Arc::clone(&escape_keys);

    let mount = mount_to(host.clone(), move || {
        let dismissals = Arc::clone(&dismissals_handle);
        let pointer_targets = Arc::clone(&pointer_targets_handle);
        let focus_targets = Arc::clone(&focus_targets_handle);
        let escape_keys = Arc::clone(&escape_keys_handle);

        view! {
            <DismissibleLayer
                on_dismiss=Callback::new(move |reason| {
                    dismissals.lock().expect("dismissals lock").push(reason);
                })
                on_pointer_down_outside=Callback::new(move |event: leptos::ev::PointerEvent| {
                    let target_id = event
                        .target()
                        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                        .map(|element| element.id())
                        .unwrap_or_default();
                    pointer_targets
                        .lock()
                        .expect("pointer targets lock")
                        .push(target_id);
                })
                on_focus_outside=Callback::new(move |event: leptos::ev::FocusEvent| {
                    let target_id = event
                        .target()
                        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                        .map(|element| element.id())
                        .unwrap_or_default();
                    focus_targets
                        .lock()
                        .expect("focus targets lock")
                        .push(target_id);
                })
                on_escape_key_down=Callback::new(move |event: leptos::ev::KeyboardEvent| {
                    escape_keys
                        .lock()
                        .expect("escape keys lock")
                        .push(event.key());
                })
            >
                <button id="dismissible-callback-focus-inside">"Inside"</button>
            </DismissibleLayer>
        }
    });

    let inside = html_element_by_id("dismissible-callback-focus-inside");

    inside.focus().expect("focus inside");
    outside.focus().expect("focus outside");
    inside.focus().expect("restore focus inside");
    dispatch_escape_keydown(&inside);

    assert_eq!(
        dismissals.lock().expect("dismissals lock").as_slice(),
        &[DismissibleReason::FocusOutside, DismissibleReason::Escape]
    );
    assert!(pointer_targets.lock().expect("pointer targets lock").is_empty());
    assert_eq!(
        focus_targets.lock().expect("focus targets lock").as_slice(),
        &["dismissible-callback-focus-outside".to_string()]
    );
    assert_eq!(
        escape_keys.lock().expect("escape keys lock").as_slice(),
        &["Escape".to_string()]
    );

    drop(mount);
    remove_from_body(&host);
    remove_from_body(&outside);
}

#[wasm_bindgen_test]
fn nested_dismissible_layers_route_pointer_and_focus_outside_to_the_topmost_layer() {
    let host = append_div("dismissible-stack-host");
    let outer_dismissals: Arc<Mutex<Vec<DismissibleReason>>> = Arc::new(Mutex::new(Vec::new()));
    let outer_dismissals_handle = Arc::clone(&outer_dismissals);
    let inner_dismissals: Arc<Mutex<Vec<DismissibleReason>>> = Arc::new(Mutex::new(Vec::new()));
    let inner_dismissals_handle = Arc::clone(&inner_dismissals);

    let mount = mount_to(host.clone(), move || {
        let outer_dismissals = Arc::clone(&outer_dismissals_handle);
        let inner_dismissals = Arc::clone(&inner_dismissals_handle);
        let outer_on_dismiss = Callback::new(move |reason| {
            outer_dismissals
                .lock()
                .expect("outer dismissals lock")
                .push(reason);
        });
        let inner_on_dismiss = Callback::new(move |reason| {
            inner_dismissals
                .lock()
                .expect("inner dismissals lock")
                .push(reason);
        });

        view! {
            <DismissibleLayer on_dismiss=outer_on_dismiss>
                <button id="dismissible-stack-outer-only">"Outer"</button>
                <DismissibleLayer on_dismiss=inner_on_dismiss.clone()>
                    <button id="dismissible-stack-inner">"Inner"</button>
                </DismissibleLayer>
            </DismissibleLayer>
        }
    });

    let inner = html_element_by_id("dismissible-stack-inner");
    let outer_only = html_element_by_id("dismissible-stack-outer-only");

    dispatch_pointer_down(&outer_only);

    assert!(outer_dismissals.lock().expect("outer dismissals lock").is_empty());
    assert_eq!(
        inner_dismissals.lock().expect("inner dismissals lock").as_slice(),
        &[DismissibleReason::PointerDownOutside]
    );

    inner.focus().expect("focus inner");
    outer_only.focus().expect("focus outer only");

    assert!(outer_dismissals.lock().expect("outer dismissals lock").is_empty());
    assert_eq!(
        inner_dismissals.lock().expect("inner dismissals lock").as_slice(),
        &[
            DismissibleReason::PointerDownOutside,
            DismissibleReason::FocusOutside
        ]
    );

    drop(mount);
    remove_from_body(&host);
}

#[wasm_bindgen_test]
fn nested_dismissible_layers_route_escape_to_the_topmost_layer() {
    let host = append_div("dismissible-stack-escape-host");
    let outer_dismissals: Arc<Mutex<Vec<DismissibleReason>>> = Arc::new(Mutex::new(Vec::new()));
    let outer_dismissals_handle = Arc::clone(&outer_dismissals);
    let inner_dismissals: Arc<Mutex<Vec<DismissibleReason>>> = Arc::new(Mutex::new(Vec::new()));
    let inner_dismissals_handle = Arc::clone(&inner_dismissals);

    let mount = mount_to(host.clone(), move || {
        let outer_dismissals = Arc::clone(&outer_dismissals_handle);
        let inner_dismissals = Arc::clone(&inner_dismissals_handle);
        let outer_on_dismiss = Callback::new(move |reason| {
            outer_dismissals
                .lock()
                .expect("outer dismissals lock")
                .push(reason);
        });
        let inner_on_dismiss = Callback::new(move |reason| {
            inner_dismissals
                .lock()
                .expect("inner dismissals lock")
                .push(reason);
        });

        view! {
            <DismissibleLayer on_dismiss=outer_on_dismiss>
                <DismissibleLayer on_dismiss=inner_on_dismiss.clone()>
                    <button id="dismissible-stack-escape-inner">"Inner"</button>
                </DismissibleLayer>
            </DismissibleLayer>
        }
    });

    let inner = html_element_by_id("dismissible-stack-escape-inner");
    dispatch_escape_keydown(&inner);

    assert!(outer_dismissals.lock().expect("outer dismissals lock").is_empty());
    assert_eq!(
        inner_dismissals.lock().expect("inner dismissals lock").as_slice(),
        &[DismissibleReason::Escape]
    );

    drop(mount);
    remove_from_body(&host);
}

#[wasm_bindgen_test]
fn nested_dismissible_layers_restore_outer_pointer_and_focus_after_inner_unmount() {
    let host = append_div("dismissible-stack-restore-host");
    let outside = append_button("dismissible-stack-restore-outside");
    let inner_present = RwSignal::new(true);
    let outer_dismissals: Arc<Mutex<Vec<DismissibleReason>>> = Arc::new(Mutex::new(Vec::new()));
    let outer_dismissals_handle = Arc::clone(&outer_dismissals);
    let inner_dismissals: Arc<Mutex<Vec<DismissibleReason>>> = Arc::new(Mutex::new(Vec::new()));
    let inner_dismissals_handle = Arc::clone(&inner_dismissals);

    let mount = mount_to(host.clone(), move || {
        let outer_dismissals = Arc::clone(&outer_dismissals_handle);
        let inner_dismissals = Arc::clone(&inner_dismissals_handle);
        let outer_on_dismiss = Callback::new(move |reason| {
            outer_dismissals
                .lock()
                .expect("outer dismissals lock")
                .push(reason);
        });
        let inner_on_dismiss = Callback::new(move |reason| {
            inner_dismissals
                .lock()
                .expect("inner dismissals lock")
                .push(reason);
            inner_present.set(false);
        });

        view! {
            <DismissibleLayer on_dismiss=outer_on_dismiss>
                <button id="dismissible-stack-restore-outer">"Outer"</button>
                {move || {
                    inner_present.get().then(|| {
                        let on_dismiss = inner_on_dismiss.clone();
                        view! {
                            <DismissibleLayer on_dismiss=on_dismiss>
                                <button id="dismissible-stack-restore-inner">"Inner"</button>
                            </DismissibleLayer>
                        }
                    })
                }}
            </DismissibleLayer>
        }
    });

    let outer = html_element_by_id("dismissible-stack-restore-outer");

    dispatch_pointer_down(&outer);

    assert!(outer_dismissals.lock().expect("outer dismissals lock").is_empty());
    assert_eq!(
        inner_dismissals.lock().expect("inner dismissals lock").as_slice(),
        &[DismissibleReason::PointerDownOutside]
    );
    assert!(
        document()
            .get_element_by_id("dismissible-stack-restore-inner")
            .is_none()
    );

    dispatch_pointer_down(&outside);
    assert_eq!(
        outer_dismissals.lock().expect("outer dismissals lock").as_slice(),
        &[DismissibleReason::PointerDownOutside]
    );

    outer.focus().expect("focus outer");
    outside.focus().expect("focus outside");
    assert_eq!(
        outer_dismissals.lock().expect("outer dismissals lock").as_slice(),
        &[
            DismissibleReason::PointerDownOutside,
            DismissibleReason::FocusOutside
        ]
    );
    assert_eq!(
        inner_dismissals.lock().expect("inner dismissals lock").as_slice(),
        &[DismissibleReason::PointerDownOutside]
    );

    drop(mount);
    remove_from_body(&host);
    remove_from_body(&outside);
}

#[wasm_bindgen_test]
fn nested_dismissible_layers_restore_outer_escape_after_inner_unmount() {
    let host = append_div("dismissible-stack-restore-escape-host");
    let inner_present = RwSignal::new(true);
    let outer_dismissals: Arc<Mutex<Vec<DismissibleReason>>> = Arc::new(Mutex::new(Vec::new()));
    let outer_dismissals_handle = Arc::clone(&outer_dismissals);
    let inner_dismissals: Arc<Mutex<Vec<DismissibleReason>>> = Arc::new(Mutex::new(Vec::new()));
    let inner_dismissals_handle = Arc::clone(&inner_dismissals);

    let mount = mount_to(host.clone(), move || {
        let outer_dismissals = Arc::clone(&outer_dismissals_handle);
        let inner_dismissals = Arc::clone(&inner_dismissals_handle);
        let outer_on_dismiss = Callback::new(move |reason| {
            outer_dismissals
                .lock()
                .expect("outer dismissals lock")
                .push(reason);
        });
        let inner_on_dismiss = Callback::new(move |reason| {
            inner_dismissals
                .lock()
                .expect("inner dismissals lock")
                .push(reason);
            inner_present.set(false);
        });

        view! {
            <DismissibleLayer on_dismiss=outer_on_dismiss>
                <button id="dismissible-stack-restore-escape-outer">"Outer"</button>
                {move || {
                    inner_present.get().then(|| {
                        let on_dismiss = inner_on_dismiss.clone();
                        view! {
                            <DismissibleLayer on_dismiss=on_dismiss>
                                <button id="dismissible-stack-restore-escape-inner">"Inner"</button>
                            </DismissibleLayer>
                        }
                    })
                }}
            </DismissibleLayer>
        }
    });

    let inner = html_element_by_id("dismissible-stack-restore-escape-inner");
    dispatch_escape_keydown(&inner);

    assert!(outer_dismissals.lock().expect("outer dismissals lock").is_empty());
    assert_eq!(
        inner_dismissals.lock().expect("inner dismissals lock").as_slice(),
        &[DismissibleReason::Escape]
    );
    assert!(
        document()
            .get_element_by_id("dismissible-stack-restore-escape-inner")
            .is_none()
    );

    let outer = html_element_by_id("dismissible-stack-restore-escape-outer");
    outer.focus().expect("focus outer");
    dispatch_escape_keydown(&outer);

    assert_eq!(
        outer_dismissals.lock().expect("outer dismissals lock").as_slice(),
        &[DismissibleReason::Escape]
    );
    assert_eq!(
        inner_dismissals.lock().expect("inner dismissals lock").as_slice(),
        &[DismissibleReason::Escape]
    );

    drop(mount);
    remove_from_body(&host);
}

#[wasm_bindgen_test]
fn stacked_dismissible_layers_suppress_pointer_outside_for_all_layers_when_topmost_pointer_dismiss_is_disabled(
) {
    let host = append_div("dismissible-stack-suppressed-pointer-host");
    let outside = append_div("dismissible-stack-suppressed-pointer-outside");
    let outer_dismissals: Arc<Mutex<Vec<DismissibleReason>>> = Arc::new(Mutex::new(Vec::new()));
    let outer_dismissals_handle = Arc::clone(&outer_dismissals);
    let inner_dismissals: Arc<Mutex<Vec<DismissibleReason>>> = Arc::new(Mutex::new(Vec::new()));
    let inner_dismissals_handle = Arc::clone(&inner_dismissals);
    let outer_pointer_targets: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let outer_pointer_targets_handle = Arc::clone(&outer_pointer_targets);
    let inner_pointer_targets: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let inner_pointer_targets_handle = Arc::clone(&inner_pointer_targets);

    let mount = mount_to(host.clone(), move || {
        let outer_dismissals = Arc::clone(&outer_dismissals_handle);
        let inner_dismissals = Arc::clone(&inner_dismissals_handle);
        let outer_pointer_targets = Arc::clone(&outer_pointer_targets_handle);
        let inner_pointer_targets = Arc::clone(&inner_pointer_targets_handle);
        let outer_on_dismiss = Callback::new(move |reason| {
            outer_dismissals
                .lock()
                .expect("outer dismissals lock")
                .push(reason);
        });
        let inner_on_dismiss = Callback::new(move |reason| {
            inner_dismissals
                .lock()
                .expect("inner dismissals lock")
                .push(reason);
        });
        let outer_on_pointer_down_outside =
            Callback::new(move |event: leptos::ev::PointerEvent| {
                let target_id = event
                    .target()
                    .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                    .map(|element| element.id())
                    .unwrap_or_default();
                outer_pointer_targets
                    .lock()
                    .expect("outer pointer targets lock")
                    .push(target_id);
            });
        let inner_on_pointer_down_outside =
            Callback::new(move |event: leptos::ev::PointerEvent| {
                let target_id = event
                    .target()
                    .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                    .map(|element| element.id())
                    .unwrap_or_default();
                inner_pointer_targets
                    .lock()
                    .expect("inner pointer targets lock")
                    .push(target_id);
            });

        view! {
            <DismissibleLayer
                on_dismiss=outer_on_dismiss
                on_pointer_down_outside=outer_on_pointer_down_outside
            >
                <DismissibleLayer
                    disable_pointer_down_outside_dismiss=true
                    on_dismiss=inner_on_dismiss
                    on_pointer_down_outside=inner_on_pointer_down_outside
                >
                    <button id="dismissible-stack-suppressed-pointer-inner">"Inner"</button>
                </DismissibleLayer>
            </DismissibleLayer>
        }
    });

    dispatch_pointer_down(&outside);

    assert!(outer_dismissals.lock().expect("outer dismissals lock").is_empty());
    assert!(inner_dismissals.lock().expect("inner dismissals lock").is_empty());
    assert!(
        outer_pointer_targets
            .lock()
            .expect("outer pointer targets lock")
            .is_empty()
    );
    assert!(
        inner_pointer_targets
            .lock()
            .expect("inner pointer targets lock")
            .is_empty()
    );

    drop(mount);
    remove_from_body(&host);
    remove_from_body(&outside);
}

#[wasm_bindgen_test]
fn stacked_dismissible_layers_keep_focus_and_escape_owned_by_the_topmost_suppressed_layer() {
    let host = append_div("dismissible-stack-suppressed-ownership-host");
    let outside = append_button("dismissible-stack-suppressed-ownership-outside");
    let outer_dismissals: Arc<Mutex<Vec<DismissibleReason>>> = Arc::new(Mutex::new(Vec::new()));
    let outer_dismissals_handle = Arc::clone(&outer_dismissals);
    let inner_dismissals: Arc<Mutex<Vec<DismissibleReason>>> = Arc::new(Mutex::new(Vec::new()));
    let inner_dismissals_handle = Arc::clone(&inner_dismissals);
    let outer_focus_targets: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let outer_focus_targets_handle = Arc::clone(&outer_focus_targets);
    let inner_focus_targets: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let inner_focus_targets_handle = Arc::clone(&inner_focus_targets);
    let outer_escape_keys: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let outer_escape_keys_handle = Arc::clone(&outer_escape_keys);
    let inner_escape_keys: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let inner_escape_keys_handle = Arc::clone(&inner_escape_keys);

    let mount = mount_to(host.clone(), move || {
        let outer_dismissals = Arc::clone(&outer_dismissals_handle);
        let inner_dismissals = Arc::clone(&inner_dismissals_handle);
        let outer_focus_targets = Arc::clone(&outer_focus_targets_handle);
        let inner_focus_targets = Arc::clone(&inner_focus_targets_handle);
        let outer_escape_keys = Arc::clone(&outer_escape_keys_handle);
        let inner_escape_keys = Arc::clone(&inner_escape_keys_handle);
        let outer_on_dismiss = Callback::new(move |reason| {
            outer_dismissals
                .lock()
                .expect("outer dismissals lock")
                .push(reason);
        });
        let inner_on_dismiss = Callback::new(move |reason| {
            inner_dismissals
                .lock()
                .expect("inner dismissals lock")
                .push(reason);
        });
        let outer_on_focus_outside = Callback::new(move |event: leptos::ev::FocusEvent| {
            let target_id = event
                .target()
                .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                .map(|element| element.id())
                .unwrap_or_default();
            outer_focus_targets
                .lock()
                .expect("outer focus targets lock")
                .push(target_id);
        });
        let inner_on_focus_outside = Callback::new(move |event: leptos::ev::FocusEvent| {
            let target_id = event
                .target()
                .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                .map(|element| element.id())
                .unwrap_or_default();
            inner_focus_targets
                .lock()
                .expect("inner focus targets lock")
                .push(target_id);
        });
        let outer_on_escape_key_down = Callback::new(move |event: leptos::ev::KeyboardEvent| {
            outer_escape_keys
                .lock()
                .expect("outer escape keys lock")
                .push(event.key());
        });
        let inner_on_escape_key_down = Callback::new(move |event: leptos::ev::KeyboardEvent| {
            inner_escape_keys
                .lock()
                .expect("inner escape keys lock")
                .push(event.key());
        });

        view! {
            <DismissibleLayer
                on_dismiss=outer_on_dismiss
                on_focus_outside=outer_on_focus_outside
                on_escape_key_down=outer_on_escape_key_down
            >
                <DismissibleLayer
                    disable_pointer_down_outside_dismiss=true
                    on_dismiss=inner_on_dismiss
                    on_focus_outside=inner_on_focus_outside
                    on_escape_key_down=inner_on_escape_key_down
                >
                    <button id="dismissible-stack-suppressed-ownership-inner">"Inner"</button>
                </DismissibleLayer>
            </DismissibleLayer>
        }
    });

    let inner = html_element_by_id("dismissible-stack-suppressed-ownership-inner");

    inner.focus().expect("focus inner");
    outside.focus().expect("focus outside");
    inner.focus().expect("restore focus inner");
    dispatch_escape_keydown(&inner);

    assert!(outer_dismissals.lock().expect("outer dismissals lock").is_empty());
    assert!(
        outer_focus_targets
            .lock()
            .expect("outer focus targets lock")
            .is_empty()
    );
    assert!(
        outer_escape_keys
            .lock()
            .expect("outer escape keys lock")
            .is_empty()
    );
    assert_eq!(
        inner_dismissals.lock().expect("inner dismissals lock").as_slice(),
        &[DismissibleReason::FocusOutside, DismissibleReason::Escape]
    );
    assert_eq!(
        inner_focus_targets
            .lock()
            .expect("inner focus targets lock")
            .as_slice(),
        &["dismissible-stack-suppressed-ownership-outside".to_string()]
    );
    assert_eq!(
        inner_escape_keys
            .lock()
            .expect("inner escape keys lock")
            .as_slice(),
        &["Escape".to_string()]
    );

    drop(mount);
    remove_from_body(&host);
    remove_from_body(&outside);
}

#[wasm_bindgen_test]
fn stacked_suppressed_dismissible_layers_restore_outer_pointer_and_focus_after_inner_unmount() {
    let host = append_div("dismissible-stack-suppressed-restore-host");
    let outside = append_button("dismissible-stack-suppressed-restore-outside");
    let inner_present = RwSignal::new(true);
    let outer_dismissals: Arc<Mutex<Vec<DismissibleReason>>> = Arc::new(Mutex::new(Vec::new()));
    let outer_dismissals_handle = Arc::clone(&outer_dismissals);
    let inner_dismissals: Arc<Mutex<Vec<DismissibleReason>>> = Arc::new(Mutex::new(Vec::new()));
    let inner_dismissals_handle = Arc::clone(&inner_dismissals);
    let outer_focus_targets: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let outer_focus_targets_handle = Arc::clone(&outer_focus_targets);

    let mount = mount_to(host.clone(), move || {
        let outer_dismissals = Arc::clone(&outer_dismissals_handle);
        let inner_dismissals = Arc::clone(&inner_dismissals_handle);
        let outer_focus_targets = Arc::clone(&outer_focus_targets_handle);
        let outer_on_dismiss = Callback::new(move |reason| {
            outer_dismissals
                .lock()
                .expect("outer dismissals lock")
                .push(reason);
        });
        let inner_on_dismiss = Callback::new(move |reason| {
            inner_dismissals
                .lock()
                .expect("inner dismissals lock")
                .push(reason);
            inner_present.set(false);
        });
        let outer_on_focus_outside = Callback::new(move |event: leptos::ev::FocusEvent| {
            let target_id = event
                .target()
                .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                .map(|element| element.id())
                .unwrap_or_default();
            outer_focus_targets
                .lock()
                .expect("outer focus targets lock")
                .push(target_id);
        });

        view! {
            <DismissibleLayer
                on_dismiss=outer_on_dismiss
                on_focus_outside=outer_on_focus_outside
            >
                <button id="dismissible-stack-suppressed-restore-outer">"Outer"</button>
                {move || {
                    inner_present.get().then(|| {
                        let on_dismiss = inner_on_dismiss.clone();
                        view! {
                            <DismissibleLayer
                                disable_pointer_down_outside_dismiss=true
                                on_dismiss=on_dismiss
                            >
                                <button id="dismissible-stack-suppressed-restore-inner">"Inner"</button>
                            </DismissibleLayer>
                        }
                    })
                }}
            </DismissibleLayer>
        }
    });

    let inner = html_element_by_id("dismissible-stack-suppressed-restore-inner");
    inner.focus().expect("focus inner");
    outside.focus().expect("focus outside");

    assert!(outer_dismissals.lock().expect("outer dismissals lock").is_empty());
    assert_eq!(
        inner_dismissals.lock().expect("inner dismissals lock").as_slice(),
        &[DismissibleReason::FocusOutside]
    );
    assert!(
        document()
            .get_element_by_id("dismissible-stack-suppressed-restore-inner")
            .is_none()
    );

    dispatch_pointer_down(&outside);
    let outer = html_element_by_id("dismissible-stack-suppressed-restore-outer");
    outer.focus().expect("focus outer");
    outside.focus().expect("restore focus outside");

    assert_eq!(
        outer_dismissals.lock().expect("outer dismissals lock").as_slice(),
        &[
            DismissibleReason::PointerDownOutside,
            DismissibleReason::FocusOutside
        ]
    );
    assert_eq!(
        outer_focus_targets
            .lock()
            .expect("outer focus targets lock")
            .as_slice(),
        &["dismissible-stack-suppressed-restore-outside".to_string()]
    );
    assert_eq!(
        inner_dismissals.lock().expect("inner dismissals lock").as_slice(),
        &[DismissibleReason::FocusOutside]
    );

    drop(mount);
    remove_from_body(&host);
    remove_from_body(&outside);
}

#[wasm_bindgen_test]
fn stacked_suppressed_dismissible_layers_restore_outer_escape_after_inner_unmount() {
    let host = append_div("dismissible-stack-suppressed-restore-escape-host");
    let inner_present = RwSignal::new(true);
    let outer_dismissals: Arc<Mutex<Vec<DismissibleReason>>> = Arc::new(Mutex::new(Vec::new()));
    let outer_dismissals_handle = Arc::clone(&outer_dismissals);
    let inner_dismissals: Arc<Mutex<Vec<DismissibleReason>>> = Arc::new(Mutex::new(Vec::new()));
    let inner_dismissals_handle = Arc::clone(&inner_dismissals);

    let mount = mount_to(host.clone(), move || {
        let outer_dismissals = Arc::clone(&outer_dismissals_handle);
        let inner_dismissals = Arc::clone(&inner_dismissals_handle);
        let outer_on_dismiss = Callback::new(move |reason| {
            outer_dismissals
                .lock()
                .expect("outer dismissals lock")
                .push(reason);
        });
        let inner_on_dismiss = Callback::new(move |reason| {
            inner_dismissals
                .lock()
                .expect("inner dismissals lock")
                .push(reason);
            inner_present.set(false);
        });

        view! {
            <DismissibleLayer on_dismiss=outer_on_dismiss>
                <button id="dismissible-stack-suppressed-restore-escape-outer">"Outer"</button>
                {move || {
                    inner_present.get().then(|| {
                        let on_dismiss = inner_on_dismiss.clone();
                        view! {
                            <DismissibleLayer
                                disable_pointer_down_outside_dismiss=true
                                on_dismiss=on_dismiss
                            >
                                <button id="dismissible-stack-suppressed-restore-escape-inner">"Inner"</button>
                            </DismissibleLayer>
                        }
                    })
                }}
            </DismissibleLayer>
        }
    });

    let inner = html_element_by_id("dismissible-stack-suppressed-restore-escape-inner");
    dispatch_escape_keydown(&inner);

    assert!(outer_dismissals.lock().expect("outer dismissals lock").is_empty());
    assert_eq!(
        inner_dismissals.lock().expect("inner dismissals lock").as_slice(),
        &[DismissibleReason::Escape]
    );
    assert!(
        document()
            .get_element_by_id("dismissible-stack-suppressed-restore-escape-inner")
            .is_none()
    );

    let outer = html_element_by_id("dismissible-stack-suppressed-restore-escape-outer");
    outer.focus().expect("focus outer");
    dispatch_escape_keydown(&outer);

    assert_eq!(
        outer_dismissals.lock().expect("outer dismissals lock").as_slice(),
        &[DismissibleReason::Escape]
    );
    assert_eq!(
        inner_dismissals.lock().expect("inner dismissals lock").as_slice(),
        &[DismissibleReason::Escape]
    );

    drop(mount);
    remove_from_body(&host);
}

#[wasm_bindgen_test]
fn dismissible_stack_cleanup_handles_middle_sibling_removal_for_pointer_and_focus() {
    let host = append_div("dismissible-nonlifo-doc-host");
    let outside = append_button("dismissible-nonlifo-doc-outside");
    let middle_present = RwSignal::new(true);
    let inner_present = RwSignal::new(true);
    let outer_dismissals: Arc<Mutex<Vec<DismissibleReason>>> = Arc::new(Mutex::new(Vec::new()));
    let outer_dismissals_handle = Arc::clone(&outer_dismissals);
    let inner_dismissals: Arc<Mutex<Vec<DismissibleReason>>> = Arc::new(Mutex::new(Vec::new()));
    let inner_dismissals_handle = Arc::clone(&inner_dismissals);
    let outer_focus_targets: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let outer_focus_targets_handle = Arc::clone(&outer_focus_targets);
    let inner_focus_targets: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let inner_focus_targets_handle = Arc::clone(&inner_focus_targets);

    let mount = mount_to(host.clone(), move || {
        let outer_dismissals = Arc::clone(&outer_dismissals_handle);
        let inner_dismissals = Arc::clone(&inner_dismissals_handle);
        let outer_focus_targets = Arc::clone(&outer_focus_targets_handle);
        let inner_focus_targets = Arc::clone(&inner_focus_targets_handle);
        let outer_on_dismiss = Callback::new(move |reason| {
            outer_dismissals
                .lock()
                .expect("outer dismissals lock")
                .push(reason);
        });
        let inner_on_dismiss = Callback::new(move |reason| {
            inner_dismissals
                .lock()
                .expect("inner dismissals lock")
                .push(reason);
        });
        let outer_on_focus_outside = Callback::new(move |event: leptos::ev::FocusEvent| {
            let target_id = event
                .target()
                .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                .map(|element| element.id())
                .unwrap_or_default();
            outer_focus_targets
                .lock()
                .expect("outer focus targets lock")
                .push(target_id);
        });
        let inner_on_focus_outside = Callback::new(move |event: leptos::ev::FocusEvent| {
            let target_id = event
                .target()
                .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                .map(|element| element.id())
                .unwrap_or_default();
            inner_focus_targets
                .lock()
                .expect("inner focus targets lock")
                .push(target_id);
        });

        view! {
            <DismissibleLayer
                on_dismiss=outer_on_dismiss
                on_focus_outside=outer_on_focus_outside
            >
                <button id="dismissible-nonlifo-doc-outer">"Outer"</button>
                {move || {
                    middle_present.get().then(|| {
                        view! {
                            <DismissibleLayer>
                                <button id="dismissible-nonlifo-doc-middle">"Middle"</button>
                            </DismissibleLayer>
                        }
                    })
                }}
                {move || {
                    inner_present.get().then(|| {
                        let on_dismiss = inner_on_dismiss.clone();
                        let on_focus_outside = inner_on_focus_outside.clone();
                        view! {
                            <DismissibleLayer
                                on_dismiss=on_dismiss
                                on_focus_outside=on_focus_outside
                            >
                                <button id="dismissible-nonlifo-doc-inner">"Inner"</button>
                            </DismissibleLayer>
                        }
                    })
                }}
            </DismissibleLayer>
        }
    });

    middle_present.set(false);
    assert!(
        document()
            .get_element_by_id("dismissible-nonlifo-doc-middle")
            .is_none()
    );

    let inner = html_element_by_id("dismissible-nonlifo-doc-inner");
    let outer = html_element_by_id("dismissible-nonlifo-doc-outer");

    inner.focus().expect("focus inner");
    outer.focus().expect("focus outer");
    dispatch_pointer_down(&outside);

    assert!(outer_dismissals.lock().expect("outer dismissals lock").is_empty());
    assert!(
        outer_focus_targets
            .lock()
            .expect("outer focus targets lock")
            .is_empty()
    );
    assert_eq!(
        inner_dismissals.lock().expect("inner dismissals lock").as_slice(),
        &[
            DismissibleReason::FocusOutside,
            DismissibleReason::PointerDownOutside
        ]
    );
    assert_eq!(
        inner_focus_targets
            .lock()
            .expect("inner focus targets lock")
            .as_slice(),
        &["dismissible-nonlifo-doc-outer".to_string()]
    );

    inner_present.set(false);
    assert!(
        document()
            .get_element_by_id("dismissible-nonlifo-doc-inner")
            .is_none()
    );

    dispatch_pointer_down(&outside);
    outer.focus().expect("refocus outer");
    outside.focus().expect("focus outside");

    assert_eq!(
        outer_dismissals.lock().expect("outer dismissals lock").as_slice(),
        &[
            DismissibleReason::PointerDownOutside,
            DismissibleReason::FocusOutside
        ]
    );
    assert_eq!(
        outer_focus_targets
            .lock()
            .expect("outer focus targets lock")
            .as_slice(),
        &["dismissible-nonlifo-doc-outside".to_string()]
    );
    assert_eq!(
        inner_dismissals.lock().expect("inner dismissals lock").as_slice(),
        &[
            DismissibleReason::FocusOutside,
            DismissibleReason::PointerDownOutside
        ]
    );

    drop(mount);
    remove_from_body(&host);
    remove_from_body(&outside);
}

#[wasm_bindgen_test]
fn dismissible_stack_cleanup_handles_middle_sibling_removal_for_escape() {
    let host = append_div("dismissible-nonlifo-escape-host");
    let middle_present = RwSignal::new(true);
    let inner_present = RwSignal::new(true);
    let outer_dismissals: Arc<Mutex<Vec<DismissibleReason>>> = Arc::new(Mutex::new(Vec::new()));
    let outer_dismissals_handle = Arc::clone(&outer_dismissals);
    let inner_dismissals: Arc<Mutex<Vec<DismissibleReason>>> = Arc::new(Mutex::new(Vec::new()));
    let inner_dismissals_handle = Arc::clone(&inner_dismissals);

    let mount = mount_to(host.clone(), move || {
        let outer_dismissals = Arc::clone(&outer_dismissals_handle);
        let inner_dismissals = Arc::clone(&inner_dismissals_handle);
        let outer_on_dismiss = Callback::new(move |reason| {
            outer_dismissals
                .lock()
                .expect("outer dismissals lock")
                .push(reason);
        });
        let inner_on_dismiss = Callback::new(move |reason| {
            inner_dismissals
                .lock()
                .expect("inner dismissals lock")
                .push(reason);
        });

        view! {
            <DismissibleLayer on_dismiss=outer_on_dismiss>
                <button id="dismissible-nonlifo-escape-outer">"Outer"</button>
                {move || {
                    middle_present.get().then(|| {
                        view! {
                            <DismissibleLayer>
                                <button id="dismissible-nonlifo-escape-middle">"Middle"</button>
                            </DismissibleLayer>
                        }
                    })
                }}
                {move || {
                    inner_present.get().then(|| {
                        let on_dismiss = inner_on_dismiss.clone();
                        view! {
                            <DismissibleLayer on_dismiss=on_dismiss>
                                <button id="dismissible-nonlifo-escape-inner">"Inner"</button>
                            </DismissibleLayer>
                        }
                    })
                }}
            </DismissibleLayer>
        }
    });

    middle_present.set(false);
    assert!(
        document()
            .get_element_by_id("dismissible-nonlifo-escape-middle")
            .is_none()
    );

    let inner = html_element_by_id("dismissible-nonlifo-escape-inner");
    dispatch_escape_keydown(&inner);

    assert!(outer_dismissals.lock().expect("outer dismissals lock").is_empty());
    assert_eq!(
        inner_dismissals.lock().expect("inner dismissals lock").as_slice(),
        &[DismissibleReason::Escape]
    );

    inner_present.set(false);
    assert!(
        document()
            .get_element_by_id("dismissible-nonlifo-escape-inner")
            .is_none()
    );

    let outer = html_element_by_id("dismissible-nonlifo-escape-outer");
    outer.focus().expect("focus outer");
    dispatch_escape_keydown(&outer);

    assert_eq!(
        outer_dismissals.lock().expect("outer dismissals lock").as_slice(),
        &[DismissibleReason::Escape]
    );
    assert_eq!(
        inner_dismissals.lock().expect("inner dismissals lock").as_slice(),
        &[DismissibleReason::Escape]
    );

    drop(mount);
    remove_from_body(&host);
}

#[wasm_bindgen_test]
fn dismissible_cleanup_reuse_cycles_restore_outer_pointer_and_focus_each_time() {
    let host = append_div("dismissible-reuse-doc-host");
    let outside = append_button("dismissible-reuse-doc-outside");
    let inner_present = RwSignal::new(true);
    let outer_dismissals: Arc<Mutex<Vec<DismissibleReason>>> = Arc::new(Mutex::new(Vec::new()));
    let outer_dismissals_handle = Arc::clone(&outer_dismissals);
    let inner_dismissals: Arc<Mutex<Vec<DismissibleReason>>> = Arc::new(Mutex::new(Vec::new()));
    let inner_dismissals_handle = Arc::clone(&inner_dismissals);
    let outer_focus_targets: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let outer_focus_targets_handle = Arc::clone(&outer_focus_targets);

    let mount = mount_to(host.clone(), move || {
        let outer_dismissals = Arc::clone(&outer_dismissals_handle);
        let inner_dismissals = Arc::clone(&inner_dismissals_handle);
        let outer_focus_targets = Arc::clone(&outer_focus_targets_handle);
        let outer_on_dismiss = Callback::new(move |reason| {
            outer_dismissals
                .lock()
                .expect("outer dismissals lock")
                .push(reason);
        });
        let inner_on_dismiss = Callback::new(move |reason| {
            inner_dismissals
                .lock()
                .expect("inner dismissals lock")
                .push(reason);
            inner_present.set(false);
        });
        let outer_on_focus_outside = Callback::new(move |event: leptos::ev::FocusEvent| {
            let target_id = event
                .target()
                .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                .map(|element| element.id())
                .unwrap_or_default();
            outer_focus_targets
                .lock()
                .expect("outer focus targets lock")
                .push(target_id);
        });

        view! {
            <DismissibleLayer
                on_dismiss=outer_on_dismiss
                on_focus_outside=outer_on_focus_outside
            >
                <button id="dismissible-reuse-doc-outer">"Outer"</button>
                {move || {
                    inner_present.get().then(|| {
                        let on_dismiss = inner_on_dismiss.clone();
                        view! {
                            <DismissibleLayer on_dismiss=on_dismiss>
                                <button id="dismissible-reuse-doc-inner">"Inner"</button>
                            </DismissibleLayer>
                        }
                    })
                }}
            </DismissibleLayer>
        }
    });

    for cycle in 0..2 {
        if cycle > 0 {
            inner_present.set(true);
        }

        let outer = html_element_by_id("dismissible-reuse-doc-outer");
        dispatch_pointer_down(&outer);

        assert_eq!(
            inner_dismissals.lock().expect("inner dismissals lock").len(),
            cycle + 1
        );
        assert_eq!(
            outer_dismissals.lock().expect("outer dismissals lock").len(),
            cycle * 2
        );
        assert!(
            document()
                .get_element_by_id("dismissible-reuse-doc-inner")
                .is_none()
        );

        dispatch_pointer_down(&outside);
        outer.focus().expect("focus outer");
        outside.focus().expect("focus outside");
    }

    assert_eq!(
        inner_dismissals.lock().expect("inner dismissals lock").as_slice(),
        &[
            DismissibleReason::PointerDownOutside,
            DismissibleReason::PointerDownOutside
        ]
    );
    assert_eq!(
        outer_dismissals.lock().expect("outer dismissals lock").as_slice(),
        &[
            DismissibleReason::PointerDownOutside,
            DismissibleReason::FocusOutside,
            DismissibleReason::PointerDownOutside,
            DismissibleReason::FocusOutside
        ]
    );
    assert_eq!(
        outer_focus_targets
            .lock()
            .expect("outer focus targets lock")
            .as_slice(),
        &[
            "dismissible-reuse-doc-outside".to_string(),
            "dismissible-reuse-doc-outside".to_string()
        ]
    );

    drop(mount);
    remove_from_body(&host);
    remove_from_body(&outside);
}

#[wasm_bindgen_test]
fn dismissible_cleanup_reuse_cycles_restore_outer_escape_each_time() {
    let host = append_div("dismissible-reuse-escape-host");
    let inner_present = RwSignal::new(true);
    let outer_dismissals: Arc<Mutex<Vec<DismissibleReason>>> = Arc::new(Mutex::new(Vec::new()));
    let outer_dismissals_handle = Arc::clone(&outer_dismissals);
    let inner_dismissals: Arc<Mutex<Vec<DismissibleReason>>> = Arc::new(Mutex::new(Vec::new()));
    let inner_dismissals_handle = Arc::clone(&inner_dismissals);

    let mount = mount_to(host.clone(), move || {
        let outer_dismissals = Arc::clone(&outer_dismissals_handle);
        let inner_dismissals = Arc::clone(&inner_dismissals_handle);
        let outer_on_dismiss = Callback::new(move |reason| {
            outer_dismissals
                .lock()
                .expect("outer dismissals lock")
                .push(reason);
        });
        let inner_on_dismiss = Callback::new(move |reason| {
            inner_dismissals
                .lock()
                .expect("inner dismissals lock")
                .push(reason);
            inner_present.set(false);
        });

        view! {
            <DismissibleLayer on_dismiss=outer_on_dismiss>
                <button id="dismissible-reuse-escape-outer">"Outer"</button>
                {move || {
                    inner_present.get().then(|| {
                        let on_dismiss = inner_on_dismiss.clone();
                        view! {
                            <DismissibleLayer on_dismiss=on_dismiss>
                                <button id="dismissible-reuse-escape-inner">"Inner"</button>
                            </DismissibleLayer>
                        }
                    })
                }}
            </DismissibleLayer>
        }
    });

    for cycle in 0..2 {
        if cycle > 0 {
            inner_present.set(true);
        }

        let inner = html_element_by_id("dismissible-reuse-escape-inner");
        dispatch_escape_keydown(&inner);

        assert_eq!(
            inner_dismissals.lock().expect("inner dismissals lock").len(),
            cycle + 1
        );
        assert_eq!(
            outer_dismissals.lock().expect("outer dismissals lock").len(),
            cycle
        );
        assert!(
            document()
                .get_element_by_id("dismissible-reuse-escape-inner")
                .is_none()
        );

        let outer = html_element_by_id("dismissible-reuse-escape-outer");
        outer.focus().expect("focus outer");
        dispatch_escape_keydown(&outer);
    }

    assert_eq!(
        inner_dismissals.lock().expect("inner dismissals lock").as_slice(),
        &[DismissibleReason::Escape, DismissibleReason::Escape]
    );
    assert_eq!(
        outer_dismissals.lock().expect("outer dismissals lock").as_slice(),
        &[DismissibleReason::Escape, DismissibleReason::Escape]
    );

    drop(mount);
    remove_from_body(&host);
}

#[wasm_bindgen_test]
fn dismissible_layers_emit_no_pointer_or_focus_callbacks_after_full_teardown() {
    let host = append_div("dismissible-full-teardown-doc-host");
    let outside = append_button("dismissible-full-teardown-doc-outside");
    let outer_present = RwSignal::new(true);
    let outer_dismissals: Arc<Mutex<Vec<DismissibleReason>>> = Arc::new(Mutex::new(Vec::new()));
    let outer_dismissals_handle = Arc::clone(&outer_dismissals);
    let inner_dismissals: Arc<Mutex<Vec<DismissibleReason>>> = Arc::new(Mutex::new(Vec::new()));
    let inner_dismissals_handle = Arc::clone(&inner_dismissals);
    let outer_pointer_targets: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let outer_pointer_targets_handle = Arc::clone(&outer_pointer_targets);
    let inner_pointer_targets: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let inner_pointer_targets_handle = Arc::clone(&inner_pointer_targets);
    let outer_focus_targets: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let outer_focus_targets_handle = Arc::clone(&outer_focus_targets);
    let inner_focus_targets: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let inner_focus_targets_handle = Arc::clone(&inner_focus_targets);

    let mount = mount_to(host.clone(), move || {
        let outer_dismissals = Arc::clone(&outer_dismissals_handle);
        let inner_dismissals = Arc::clone(&inner_dismissals_handle);
        let outer_pointer_targets = Arc::clone(&outer_pointer_targets_handle);
        let inner_pointer_targets = Arc::clone(&inner_pointer_targets_handle);
        let outer_focus_targets = Arc::clone(&outer_focus_targets_handle);
        let inner_focus_targets = Arc::clone(&inner_focus_targets_handle);
        let outer_on_dismiss = Callback::new(move |reason| {
            outer_dismissals
                .lock()
                .expect("outer dismissals lock")
                .push(reason);
        });
        let inner_on_dismiss = Callback::new(move |reason| {
            inner_dismissals
                .lock()
                .expect("inner dismissals lock")
                .push(reason);
        });
        let outer_on_pointer_down_outside =
            Callback::new(move |event: leptos::ev::PointerEvent| {
                let target_id = event
                    .target()
                    .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                    .map(|element| element.id())
                    .unwrap_or_default();
                outer_pointer_targets
                    .lock()
                    .expect("outer pointer targets lock")
                    .push(target_id);
            });
        let inner_on_pointer_down_outside =
            Callback::new(move |event: leptos::ev::PointerEvent| {
                let target_id = event
                    .target()
                    .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                    .map(|element| element.id())
                    .unwrap_or_default();
                inner_pointer_targets
                    .lock()
                    .expect("inner pointer targets lock")
                    .push(target_id);
            });
        let outer_on_focus_outside = Callback::new(move |event: leptos::ev::FocusEvent| {
            let target_id = event
                .target()
                .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                .map(|element| element.id())
                .unwrap_or_default();
            outer_focus_targets
                .lock()
                .expect("outer focus targets lock")
                .push(target_id);
        });
        let inner_on_focus_outside = Callback::new(move |event: leptos::ev::FocusEvent| {
            let target_id = event
                .target()
                .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                .map(|element| element.id())
                .unwrap_or_default();
            inner_focus_targets
                .lock()
                .expect("inner focus targets lock")
                .push(target_id);
        });

        view! {
            {move || {
                outer_present.get().then(|| {
                    let outer_on_dismiss = outer_on_dismiss.clone();
                    let inner_on_dismiss = inner_on_dismiss.clone();
                    let outer_on_pointer_down_outside = outer_on_pointer_down_outside.clone();
                    let inner_on_pointer_down_outside = inner_on_pointer_down_outside.clone();
                    let outer_on_focus_outside = outer_on_focus_outside.clone();
                    let inner_on_focus_outside = inner_on_focus_outside.clone();
                    view! {
                        <DismissibleLayer
                            on_dismiss=outer_on_dismiss
                            on_pointer_down_outside=outer_on_pointer_down_outside
                            on_focus_outside=outer_on_focus_outside
                        >
                            <DismissibleLayer
                                on_dismiss=inner_on_dismiss
                                on_pointer_down_outside=inner_on_pointer_down_outside
                                on_focus_outside=inner_on_focus_outside
                            >
                                <button id="dismissible-full-teardown-doc-inner">"Inner"</button>
                            </DismissibleLayer>
                        </DismissibleLayer>
                    }
                })
            }}
        }
    });

    outer_present.set(false);

    assert!(host.first_element_child().is_none());
    assert!(
        document()
            .get_element_by_id("dismissible-full-teardown-doc-inner")
            .is_none()
    );

    dispatch_pointer_down(&outside);
    outside.focus().expect("focus outside after teardown");

    assert!(outer_dismissals.lock().expect("outer dismissals lock").is_empty());
    assert!(inner_dismissals.lock().expect("inner dismissals lock").is_empty());
    assert!(
        outer_pointer_targets
            .lock()
            .expect("outer pointer targets lock")
            .is_empty()
    );
    assert!(
        inner_pointer_targets
            .lock()
            .expect("inner pointer targets lock")
            .is_empty()
    );
    assert!(
        outer_focus_targets
            .lock()
            .expect("outer focus targets lock")
            .is_empty()
    );
    assert!(
        inner_focus_targets
            .lock()
            .expect("inner focus targets lock")
            .is_empty()
    );

    drop(mount);
    remove_from_body(&host);
    remove_from_body(&outside);
}

#[wasm_bindgen_test]
fn dismissible_layers_emit_no_escape_callbacks_after_full_teardown() {
    let host = append_div("dismissible-full-teardown-escape-host");
    let outside = append_button("dismissible-full-teardown-escape-outside");
    let outer_present = RwSignal::new(true);
    let outer_dismissals: Arc<Mutex<Vec<DismissibleReason>>> = Arc::new(Mutex::new(Vec::new()));
    let outer_dismissals_handle = Arc::clone(&outer_dismissals);
    let inner_dismissals: Arc<Mutex<Vec<DismissibleReason>>> = Arc::new(Mutex::new(Vec::new()));
    let inner_dismissals_handle = Arc::clone(&inner_dismissals);
    let outer_escape_keys: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let outer_escape_keys_handle = Arc::clone(&outer_escape_keys);
    let inner_escape_keys: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let inner_escape_keys_handle = Arc::clone(&inner_escape_keys);

    let mount = mount_to(host.clone(), move || {
        let outer_dismissals = Arc::clone(&outer_dismissals_handle);
        let inner_dismissals = Arc::clone(&inner_dismissals_handle);
        let outer_escape_keys = Arc::clone(&outer_escape_keys_handle);
        let inner_escape_keys = Arc::clone(&inner_escape_keys_handle);
        let outer_on_dismiss = Callback::new(move |reason| {
            outer_dismissals
                .lock()
                .expect("outer dismissals lock")
                .push(reason);
        });
        let inner_on_dismiss = Callback::new(move |reason| {
            inner_dismissals
                .lock()
                .expect("inner dismissals lock")
                .push(reason);
        });
        let outer_on_escape_key_down = Callback::new(move |event: leptos::ev::KeyboardEvent| {
            outer_escape_keys
                .lock()
                .expect("outer escape keys lock")
                .push(event.key());
        });
        let inner_on_escape_key_down = Callback::new(move |event: leptos::ev::KeyboardEvent| {
            inner_escape_keys
                .lock()
                .expect("inner escape keys lock")
                .push(event.key());
        });

        view! {
            {move || {
                outer_present.get().then(|| {
                    let outer_on_dismiss = outer_on_dismiss.clone();
                    let inner_on_dismiss = inner_on_dismiss.clone();
                    let outer_on_escape_key_down = outer_on_escape_key_down.clone();
                    let inner_on_escape_key_down = inner_on_escape_key_down.clone();
                    view! {
                        <DismissibleLayer
                            on_dismiss=outer_on_dismiss
                            on_escape_key_down=outer_on_escape_key_down
                        >
                            <DismissibleLayer
                                on_dismiss=inner_on_dismiss
                                on_escape_key_down=inner_on_escape_key_down
                            >
                                <button id="dismissible-full-teardown-escape-inner">"Inner"</button>
                            </DismissibleLayer>
                        </DismissibleLayer>
                    }
                })
            }}
        }
    });

    outer_present.set(false);

    assert!(host.first_element_child().is_none());
    assert!(
        document()
            .get_element_by_id("dismissible-full-teardown-escape-inner")
            .is_none()
    );

    dispatch_escape_keydown(&outside);

    assert!(outer_dismissals.lock().expect("outer dismissals lock").is_empty());
    assert!(inner_dismissals.lock().expect("inner dismissals lock").is_empty());
    assert!(
        outer_escape_keys
            .lock()
            .expect("outer escape keys lock")
            .is_empty()
    );
    assert!(
        inner_escape_keys
            .lock()
            .expect("inner escape keys lock")
            .is_empty()
    );

    drop(mount);
    remove_from_body(&host);
    remove_from_body(&outside);
}

#[wasm_bindgen_test]
fn dismissible_layers_handle_pointer_and_focus_once_after_full_teardown_and_remount() {
    let host = append_div("dismissible-remount-doc-host");
    let outside = append_button("dismissible-remount-doc-outside");
    let present = RwSignal::new(true);
    let dismissals: Arc<Mutex<Vec<DismissibleReason>>> = Arc::new(Mutex::new(Vec::new()));
    let dismissals_handle = Arc::clone(&dismissals);
    let pointer_targets: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let pointer_targets_handle = Arc::clone(&pointer_targets);
    let focus_targets: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let focus_targets_handle = Arc::clone(&focus_targets);

    let mount = mount_to(host.clone(), move || {
        let dismissals = Arc::clone(&dismissals_handle);
        let pointer_targets = Arc::clone(&pointer_targets_handle);
        let focus_targets = Arc::clone(&focus_targets_handle);
        let on_dismiss = Callback::new(move |reason| {
            dismissals.lock().expect("dismissals lock").push(reason);
        });
        let on_pointer_down_outside = Callback::new(move |event: leptos::ev::PointerEvent| {
            let target_id = event
                .target()
                .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                .map(|element| element.id())
                .unwrap_or_default();
            pointer_targets
                .lock()
                .expect("pointer targets lock")
                .push(target_id);
        });
        let on_focus_outside = Callback::new(move |event: leptos::ev::FocusEvent| {
            let target_id = event
                .target()
                .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                .map(|element| element.id())
                .unwrap_or_default();
            focus_targets
                .lock()
                .expect("focus targets lock")
                .push(target_id);
        });

        view! {
            {move || {
                present.get().then(|| {
                    let on_dismiss = on_dismiss.clone();
                    let on_pointer_down_outside = on_pointer_down_outside.clone();
                    let on_focus_outside = on_focus_outside.clone();
                    view! {
                        <DismissibleLayer
                            on_dismiss=on_dismiss
                            on_pointer_down_outside=on_pointer_down_outside
                            on_focus_outside=on_focus_outside
                        >
                            <button id="dismissible-remount-doc-inner">"Inner"</button>
                        </DismissibleLayer>
                    }
                })
            }}
        }
    });

    for cycle in 0..2 {
        if cycle > 0 {
            present.set(true);
        }

        let inner = html_element_by_id("dismissible-remount-doc-inner");
        inner.focus().expect("focus inner");
        dispatch_pointer_down(&outside);
        outside.focus().expect("focus outside");

        assert_eq!(
            dismissals.lock().expect("dismissals lock").len(),
            (cycle + 1) * 2
        );
        assert_eq!(
            pointer_targets.lock().expect("pointer targets lock").len(),
            cycle + 1
        );
        assert_eq!(
            focus_targets.lock().expect("focus targets lock").len(),
            cycle + 1
        );

        present.set(false);
        assert!(host.first_element_child().is_none());
    }

    assert_eq!(
        dismissals.lock().expect("dismissals lock").as_slice(),
        &[
            DismissibleReason::PointerDownOutside,
            DismissibleReason::FocusOutside,
            DismissibleReason::PointerDownOutside,
            DismissibleReason::FocusOutside
        ]
    );
    assert_eq!(
        pointer_targets
            .lock()
            .expect("pointer targets lock")
            .as_slice(),
        &[
            "dismissible-remount-doc-outside".to_string(),
            "dismissible-remount-doc-outside".to_string()
        ]
    );
    assert_eq!(
        focus_targets.lock().expect("focus targets lock").as_slice(),
        &[
            "dismissible-remount-doc-outside".to_string(),
            "dismissible-remount-doc-outside".to_string()
        ]
    );

    drop(mount);
    remove_from_body(&host);
    remove_from_body(&outside);
}

#[wasm_bindgen_test]
fn dismissible_layers_handle_escape_once_after_full_teardown_and_remount() {
    let host = append_div("dismissible-remount-escape-host");
    let present = RwSignal::new(true);
    let dismissals: Arc<Mutex<Vec<DismissibleReason>>> = Arc::new(Mutex::new(Vec::new()));
    let dismissals_handle = Arc::clone(&dismissals);
    let escape_keys: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let escape_keys_handle = Arc::clone(&escape_keys);

    let mount = mount_to(host.clone(), move || {
        let dismissals = Arc::clone(&dismissals_handle);
        let escape_keys = Arc::clone(&escape_keys_handle);
        let on_dismiss = Callback::new(move |reason| {
            dismissals.lock().expect("dismissals lock").push(reason);
        });
        let on_escape_key_down = Callback::new(move |event: leptos::ev::KeyboardEvent| {
            escape_keys
                .lock()
                .expect("escape keys lock")
                .push(event.key());
        });

        view! {
            {move || {
                present.get().then(|| {
                    let on_dismiss = on_dismiss.clone();
                    let on_escape_key_down = on_escape_key_down.clone();
                    view! {
                        <DismissibleLayer
                            on_dismiss=on_dismiss
                            on_escape_key_down=on_escape_key_down
                        >
                            <button id="dismissible-remount-escape-inner">"Inner"</button>
                        </DismissibleLayer>
                    }
                })
            }}
        }
    });

    for cycle in 0..2 {
        if cycle > 0 {
            present.set(true);
        }

        let inner = html_element_by_id("dismissible-remount-escape-inner");
        dispatch_escape_keydown(&inner);

        assert_eq!(
            dismissals.lock().expect("dismissals lock").len(),
            cycle + 1
        );
        assert_eq!(
            escape_keys.lock().expect("escape keys lock").len(),
            cycle + 1
        );

        present.set(false);
        assert!(host.first_element_child().is_none());
    }

    assert_eq!(
        dismissals.lock().expect("dismissals lock").as_slice(),
        &[DismissibleReason::Escape, DismissibleReason::Escape]
    );
    assert_eq!(
        escape_keys.lock().expect("escape keys lock").as_slice(),
        &["Escape".to_string(), "Escape".to_string()]
    );

    drop(mount);
    remove_from_body(&host);
}

#[wasm_bindgen_test]
fn portal_mounts_children_into_the_explicit_target() {
    let host = append_div("portal-host");
    let target = append_div("portal-target");
    let target_mount: web_sys::Element = target.clone().into();

    let mount = mount_to(host.clone(), move || {
        let target_mount = target_mount.clone();
        view! {
            <Portal mount=target_mount>
                <span>"Portaled"</span>
            </Portal>
        }
    });

    assert_eq!(host.text_content().unwrap_or_default(), "");
    assert_eq!(target.text_content().as_deref(), Some("Portaled"));

    drop(mount);
    assert_eq!(target.text_content().unwrap_or_default(), "");

    remove_from_body(&host);
    remove_from_body(&target);
}

#[wasm_bindgen_test]
fn portal_without_explicit_mount_appends_to_body_and_cleans_up_on_drop() {
    let host = append_div("portal-default-host");
    let body_children_before_mount = body().child_element_count();

    let mount = mount_to(host.clone(), move || {
        view! {
            <Portal>
                <span>"Default Portaled"</span>
            </Portal>
        }
    });

    assert_eq!(host.text_content().unwrap_or_default(), "");
    assert_eq!(body().child_element_count(), body_children_before_mount + 1);

    let portal_container = body()
        .last_element_child()
        .expect("default portal container");
    assert_eq!(portal_container.id(), "");
    assert_eq!(
        portal_container.text_content().as_deref(),
        Some("Default Portaled")
    );

    drop(mount);
    assert_eq!(host.text_content().unwrap_or_default(), "");
    assert_eq!(body().child_element_count(), body_children_before_mount);

    remove_from_body(&host);
}

#[wasm_bindgen_test]
fn portal_without_explicit_mount_repeated_remounts_leave_no_stranded_body_nodes() {
    let host = append_div("portal-default-remount-host");
    let present = RwSignal::new(true);
    let label = RwSignal::new("First");
    let body_children_before_mount = body().child_element_count();

    let mount = mount_to(host.clone(), move || {
        view! {
            {move || {
                present.get().then(|| {
                    let text = label.get();

                    view! {
                        <Portal>
                            <span>{text}</span>
                        </Portal>
                    }
                })
            }}
        }
    });

    assert_eq!(host.text_content().unwrap_or_default(), "");
    assert_eq!(body().child_element_count(), body_children_before_mount + 1);
    assert_eq!(
        body()
            .last_element_child()
            .expect("first default portal container")
            .text_content()
            .as_deref(),
        Some("First")
    );

    present.set(false);
    assert_eq!(body().child_element_count(), body_children_before_mount);

    label.set("Second");
    present.set(true);
    assert_eq!(body().child_element_count(), body_children_before_mount + 1);
    assert_eq!(
        body()
            .last_element_child()
            .expect("second default portal container")
            .text_content()
            .as_deref(),
        Some("Second")
    );

    present.set(false);
    assert_eq!(body().child_element_count(), body_children_before_mount);

    drop(mount);
    assert_eq!(body().child_element_count(), body_children_before_mount);

    remove_from_body(&host);
}

#[wasm_bindgen_test]
fn portal_repeated_remounts_clear_prior_targets_before_rendering_into_new_ones() {
    let host = append_div("portal-remount-host");
    let first_target = append_div("portal-remount-first-target");
    let second_target = append_div("portal-remount-second-target");
    let present = RwSignal::new(true);
    let use_first_target = RwSignal::new(true);

    let mount = mount_to(host.clone(), move || {
        view! {
            {move || {
                present.get().then(|| {
                    let mount = if use_first_target.get() {
                        document()
                            .get_element_by_id("portal-remount-first-target")
                            .expect("first portal target")
                    } else {
                        document()
                            .get_element_by_id("portal-remount-second-target")
                            .expect("second portal target")
                    };

                    view! {
                        <Portal mount=mount>
                            <span>"Portaled"</span>
                        </Portal>
                    }
                })
            }}
        }
    });

    assert_eq!(host.text_content().unwrap_or_default(), "");
    assert_eq!(first_target.text_content().as_deref(), Some("Portaled"));
    assert_eq!(first_target.child_element_count(), 1);
    assert_eq!(second_target.text_content().unwrap_or_default(), "");
    assert_eq!(second_target.child_element_count(), 0);

    present.set(false);
    assert_eq!(first_target.text_content().unwrap_or_default(), "");
    assert_eq!(first_target.child_element_count(), 0);
    assert_eq!(second_target.text_content().unwrap_or_default(), "");
    assert_eq!(second_target.child_element_count(), 0);

    use_first_target.set(false);
    present.set(true);
    assert_eq!(first_target.text_content().unwrap_or_default(), "");
    assert_eq!(first_target.child_element_count(), 0);
    assert_eq!(second_target.text_content().as_deref(), Some("Portaled"));
    assert_eq!(second_target.child_element_count(), 1);

    present.set(false);
    assert_eq!(first_target.text_content().unwrap_or_default(), "");
    assert_eq!(first_target.child_element_count(), 0);
    assert_eq!(second_target.text_content().unwrap_or_default(), "");
    assert_eq!(second_target.child_element_count(), 0);

    use_first_target.set(true);
    present.set(true);
    assert_eq!(first_target.text_content().as_deref(), Some("Portaled"));
    assert_eq!(first_target.child_element_count(), 1);
    assert_eq!(second_target.text_content().unwrap_or_default(), "");
    assert_eq!(second_target.child_element_count(), 0);

    drop(mount);
    assert_eq!(first_target.text_content().unwrap_or_default(), "");
    assert_eq!(first_target.child_element_count(), 0);
    assert_eq!(second_target.text_content().unwrap_or_default(), "");
    assert_eq!(second_target.child_element_count(), 0);

    remove_from_body(&host);
    remove_from_body(&first_target);
    remove_from_body(&second_target);
}

#[wasm_bindgen_test]
fn portal_repeated_remounts_into_the_same_target_do_not_duplicate_children() {
    let host = append_div("portal-repeat-host");
    let target = append_div("portal-repeat-target");
    let present = RwSignal::new(true);
    let label = RwSignal::new("First");

    let mount = mount_to(host.clone(), move || {
        view! {
            {move || {
                present.get().then(|| {
                    let text = label.get();
                    let mount = document()
                        .get_element_by_id("portal-repeat-target")
                        .expect("repeat portal target");
                    view! {
                        <Portal mount=mount>
                            <span>{text}</span>
                        </Portal>
                    }
                })
            }}
        }
    });

    assert_eq!(target.text_content().as_deref(), Some("First"));
    assert_eq!(target.child_element_count(), 1);

    present.set(false);
    assert_eq!(target.text_content().unwrap_or_default(), "");
    assert_eq!(target.child_element_count(), 0);

    label.set("Second");
    present.set(true);
    assert_eq!(target.text_content().as_deref(), Some("Second"));
    assert_eq!(target.child_element_count(), 1);

    present.set(false);
    assert_eq!(target.text_content().unwrap_or_default(), "");
    assert_eq!(target.child_element_count(), 0);

    label.set("Third");
    present.set(true);
    assert_eq!(target.text_content().as_deref(), Some("Third"));
    assert_eq!(target.child_element_count(), 1);

    drop(mount);
    assert_eq!(target.text_content().unwrap_or_default(), "");
    assert_eq!(target.child_element_count(), 0);

    remove_from_body(&host);
    remove_from_body(&target);
}

#[wasm_bindgen_test]
fn portal_retargets_live_between_explicit_targets_without_teardown() {
    let host = append_div("portal-retarget-host");
    let first_target = append_div("portal-retarget-first-target");
    let second_target = append_div("portal-retarget-second-target");
    let use_first_target = RwSignal::new(true);

    let mount = mount_to(host.clone(), move || {
        view! {
            {move || {
                let mount = if use_first_target.get() {
                    document()
                        .get_element_by_id("portal-retarget-first-target")
                        .expect("first retarget portal target")
                } else {
                    document()
                        .get_element_by_id("portal-retarget-second-target")
                        .expect("second retarget portal target")
                };

                view! {
                    <Portal mount=mount>
                        <span>"Retargeted"</span>
                    </Portal>
                }
            }}
        }
    });

    assert_eq!(host.text_content().unwrap_or_default(), "");
    assert_eq!(first_target.text_content().as_deref(), Some("Retargeted"));
    assert_eq!(first_target.child_element_count(), 1);
    assert_eq!(second_target.text_content().unwrap_or_default(), "");
    assert_eq!(second_target.child_element_count(), 0);

    use_first_target.set(false);
    assert_eq!(host.text_content().unwrap_or_default(), "");
    assert_eq!(first_target.text_content().unwrap_or_default(), "");
    assert_eq!(first_target.child_element_count(), 0);
    assert_eq!(second_target.text_content().as_deref(), Some("Retargeted"));
    assert_eq!(second_target.child_element_count(), 1);

    use_first_target.set(true);
    assert_eq!(host.text_content().unwrap_or_default(), "");
    assert_eq!(first_target.text_content().as_deref(), Some("Retargeted"));
    assert_eq!(first_target.child_element_count(), 1);
    assert_eq!(second_target.text_content().unwrap_or_default(), "");
    assert_eq!(second_target.child_element_count(), 0);

    drop(mount);
    assert_eq!(first_target.text_content().unwrap_or_default(), "");
    assert_eq!(first_target.child_element_count(), 0);
    assert_eq!(second_target.text_content().unwrap_or_default(), "");
    assert_eq!(second_target.child_element_count(), 0);

    remove_from_body(&host);
    remove_from_body(&first_target);
    remove_from_body(&second_target);
}

#[wasm_bindgen_test]
fn portal_live_retargeting_keeps_only_one_copy_while_content_changes() {
    let host = append_div("portal-retarget-content-host");
    let first_target = append_div("portal-retarget-content-first-target");
    let second_target = append_div("portal-retarget-content-second-target");
    let use_first_target = RwSignal::new(true);
    let label = RwSignal::new("One");

    let mount = mount_to(host.clone(), move || {
        view! {
            {move || {
                let mount = if use_first_target.get() {
                    document()
                        .get_element_by_id("portal-retarget-content-first-target")
                        .expect("first content retarget target")
                } else {
                    document()
                        .get_element_by_id("portal-retarget-content-second-target")
                        .expect("second content retarget target")
                };
                let text = label.get();

                view! {
                    <Portal mount=mount>
                        <span>{text}</span>
                    </Portal>
                }
            }}
        }
    });

    assert_eq!(first_target.text_content().as_deref(), Some("One"));
    assert_eq!(first_target.child_element_count(), 1);
    assert_eq!(second_target.child_element_count(), 0);

    label.set("Two");
    use_first_target.set(false);
    assert_eq!(first_target.text_content().unwrap_or_default(), "");
    assert_eq!(first_target.child_element_count(), 0);
    assert_eq!(second_target.text_content().as_deref(), Some("Two"));
    assert_eq!(second_target.child_element_count(), 1);

    label.set("Three");
    use_first_target.set(true);
    assert_eq!(first_target.text_content().as_deref(), Some("Three"));
    assert_eq!(first_target.child_element_count(), 1);
    assert_eq!(second_target.text_content().unwrap_or_default(), "");
    assert_eq!(second_target.child_element_count(), 0);

    drop(mount);
    assert_eq!(first_target.text_content().unwrap_or_default(), "");
    assert_eq!(first_target.child_element_count(), 0);
    assert_eq!(second_target.text_content().unwrap_or_default(), "");
    assert_eq!(second_target.child_element_count(), 0);

    remove_from_body(&host);
    remove_from_body(&first_target);
    remove_from_body(&second_target);
}

#[wasm_bindgen_test]
fn modal_hide_siblings_hides_nested_siblings_and_restores_previous_state() {
    let shell = append_div("modal-shell");
    let left = append_child_div(&shell, "modal-left");
    let active_branch = append_child_div(&shell, "modal-active-branch");
    let right = append_child_div(&shell, "modal-right");
    let root = append_child_div(&active_branch, "modal-root");
    let branch_sibling = append_child_div(&active_branch, "modal-branch-sibling");

    left.set_attribute("aria-hidden", "false")
        .expect("set previous aria-hidden");
    right
        .set_attribute("inert", "")
        .expect("set previous inert");

    let root_element: web_sys::Element = root.clone().into();
    let guard = modal_hide_siblings(&root_element).expect("modal guard");

    assert_eq!(attr(&left, "aria-hidden").as_deref(), Some("true"));
    assert!(left.has_attribute("inert"));
    assert_eq!(attr(&right, "aria-hidden").as_deref(), Some("true"));
    assert!(right.has_attribute("inert"));
    assert_eq!(attr(&branch_sibling, "aria-hidden").as_deref(), Some("true"));
    assert!(branch_sibling.has_attribute("inert"));

    assert!(attr(&shell, "aria-hidden").is_none());
    assert!(attr(&active_branch, "aria-hidden").is_none());
    assert!(attr(&root, "aria-hidden").is_none());

    drop(guard);

    assert_eq!(attr(&left, "aria-hidden").as_deref(), Some("false"));
    assert!(!left.has_attribute("inert"));
    assert!(attr(&right, "aria-hidden").is_none());
    assert!(right.has_attribute("inert"));
    assert!(attr(&branch_sibling, "aria-hidden").is_none());
    assert!(!branch_sibling.has_attribute("inert"));

    remove_from_body(&shell);
}

#[wasm_bindgen_test]
fn nested_modal_guards_keep_outer_siblings_hidden_until_last_guard_drops() {
    let shell = append_div("nested-modal-shell");
    let left = append_child_div(&shell, "nested-modal-left");
    let active_branch = append_child_div(&shell, "nested-modal-active-branch");
    let right = append_child_div(&shell, "nested-modal-right");
    let outer_root = append_child_div(&active_branch, "nested-modal-outer-root");
    let branch_sibling = append_child_div(&active_branch, "nested-modal-branch-sibling");
    let inner_sibling = append_child_div(&outer_root, "nested-modal-inner-sibling");
    let inner_root = append_child_div(&outer_root, "nested-modal-inner-root");

    let outer_root_element: web_sys::Element = outer_root.clone().into();
    let inner_root_element: web_sys::Element = inner_root.clone().into();

    let outer_guard = modal_hide_siblings(&outer_root_element).expect("outer modal guard");
    assert_eq!(attr(&left, "aria-hidden").as_deref(), Some("true"));
    assert_eq!(attr(&right, "aria-hidden").as_deref(), Some("true"));
    assert_eq!(attr(&branch_sibling, "aria-hidden").as_deref(), Some("true"));
    assert!(attr(&inner_sibling, "aria-hidden").is_none());

    let inner_guard = modal_hide_siblings(&inner_root_element).expect("inner modal guard");
    assert_eq!(attr(&inner_sibling, "aria-hidden").as_deref(), Some("true"));
    assert!(inner_sibling.has_attribute("inert"));

    drop(inner_guard);

    assert_eq!(attr(&left, "aria-hidden").as_deref(), Some("true"));
    assert_eq!(attr(&right, "aria-hidden").as_deref(), Some("true"));
    assert_eq!(attr(&branch_sibling, "aria-hidden").as_deref(), Some("true"));
    assert!(attr(&inner_sibling, "aria-hidden").is_none());
    assert!(!inner_sibling.has_attribute("inert"));

    drop(outer_guard);

    assert!(attr(&left, "aria-hidden").is_none());
    assert!(attr(&right, "aria-hidden").is_none());
    assert!(attr(&branch_sibling, "aria-hidden").is_none());

    remove_from_body(&shell);
}

#[wasm_bindgen_test]
fn focus_scope_traps_tab_within_the_live_scope() {
    let host = append_div("focus-scope-host");

    let mount = mount_to(host.clone(), move || {
        view! {
            <>
                <button id="focus-before">"Before"</button>
                <FocusScope trapped=true>
                    <button id="focus-first">"First"</button>
                    <button id="focus-second">"Second"</button>
                </FocusScope>
                <button id="focus-after">"After"</button>
            </>
        }
    });

    let first = document()
        .get_element_by_id("focus-first")
        .expect("first button")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("first html element");
    let second = document()
        .get_element_by_id("focus-second")
        .expect("second button")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("second html element");

    first.focus().expect("focus first");
    assert_eq!(active_id().as_deref(), Some("focus-first"));

    let first_tab = dispatch_tab_keydown(&first, false);
    assert!(first_tab.default_prevented());
    assert_eq!(active_id().as_deref(), Some("focus-second"));

    let second_tab = dispatch_tab_keydown(&second, false);
    assert!(second_tab.default_prevented());
    assert_eq!(active_id().as_deref(), Some("focus-first"));
    assert_ne!(active_id().as_deref(), Some("focus-after"));

    drop(mount);
    remove_from_body(&host);
}

#[wasm_bindgen_test]
fn focus_scope_wraps_shift_tab_to_the_last_focusable() {
    let host = append_div("focus-scope-shift-host");

    let mount = mount_to(host.clone(), move || {
        view! {
            <>
                <button id="focus-shift-before">"Before"</button>
                <FocusScope trapped=true>
                    <button id="focus-shift-first">"First"</button>
                    <button id="focus-shift-last">"Last"</button>
                </FocusScope>
                <button id="focus-shift-after">"After"</button>
            </>
        }
    });

    let first = document()
        .get_element_by_id("focus-shift-first")
        .expect("first shift button")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("first shift html element");

    first.focus().expect("focus first shift");
    assert_eq!(active_id().as_deref(), Some("focus-shift-first"));

    let shift_tab = dispatch_tab_keydown(&first, true);
    assert!(shift_tab.default_prevented());
    assert_eq!(active_id().as_deref(), Some("focus-shift-last"));
    assert_ne!(active_id().as_deref(), Some("focus-shift-before"));

    drop(mount);
    remove_from_body(&host);
}

#[wasm_bindgen_test]
fn focus_scope_auto_focuses_restores_previous_focus_and_runs_callbacks() {
    let previous = document()
        .create_element("button")
        .expect("create previous button")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("previous html element");
    previous.set_id("focus-lifecycle-previous");
    body()
        .append_child(&previous)
        .expect("append previous button");
    previous.focus().expect("focus previous");
    assert_eq!(active_id().as_deref(), Some("focus-lifecycle-previous"));

    let host = append_div("focus-lifecycle-host");
    let callbacks: Arc<Mutex<Vec<&'static str>>> = Arc::new(Mutex::new(Vec::new()));
    let mount_callbacks = Arc::clone(&callbacks);
    let unmount_callbacks = Arc::clone(&callbacks);

    let mount = mount_to(host.clone(), move || {
        let on_mount_auto_focus = {
            let callbacks = Arc::clone(&mount_callbacks);
            Callback::new(move |_| {
                callbacks
                    .lock()
                    .expect("mount callbacks lock")
                    .push("mount");
            })
        };
        let on_unmount_auto_focus = {
            let callbacks = Arc::clone(&unmount_callbacks);
            Callback::new(move |_| {
                callbacks
                    .lock()
                    .expect("unmount callbacks lock")
                    .push("unmount");
            })
        };

        view! {
            <FocusScope
                auto_focus=true
                return_focus=true
                on_mount_auto_focus=on_mount_auto_focus
                on_unmount_auto_focus=on_unmount_auto_focus
            >
                <button id="focus-lifecycle-first">"First"</button>
            </FocusScope>
        }
    });

    assert_eq!(active_id().as_deref(), Some("focus-lifecycle-first"));
    assert_eq!(
        callbacks.lock().expect("callbacks lock").as_slice(),
        &["mount"]
    );

    drop(mount);

    assert_eq!(active_id().as_deref(), Some("focus-lifecycle-previous"));
    assert_eq!(
        callbacks.lock().expect("callbacks lock").as_slice(),
        &["mount", "unmount"]
    );

    remove_from_body(&host);
    remove_from_body(&previous);
}

#[wasm_bindgen_test]
fn focus_scope_auto_focus_falls_back_to_the_wrapper_when_no_child_is_focusable() {
    let host = append_div("focus-wrapper-host");

    let mount = mount_to(host.clone(), move || {
        view! {
            <FocusScope auto_focus=true>
                <span id="focus-wrapper-child">"Only child"</span>
            </FocusScope>
        }
    });

    let wrapper = host
        .first_element_child()
        .expect("focus scope wrapper")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("wrapper html element");
    let wrapper_element: web_sys::Element = wrapper.clone().into();

    assert_eq!(wrapper.tab_index(), -1);
    assert!(active_element().is_some_and(|active| {
        active.is_same_node(Some(&wrapper_element))
    }));

    drop(mount);
    remove_from_body(&host);
}

#[wasm_bindgen_test]
fn presence_ignores_bubbled_child_transitionend_until_root_transitionend_completes_exit() {
    let host = append_div("presence-transition-host");
    let present = RwSignal::new(true);
    let exit_callbacks: Arc<Mutex<Vec<&'static str>>> = Arc::new(Mutex::new(Vec::new()));
    let exit_callbacks_handle = Arc::clone(&exit_callbacks);

    let mount = mount_to(host.clone(), move || {
        let on_exit_complete = {
            let exit_callbacks = Arc::clone(&exit_callbacks_handle);
            Callback::new(move |_| {
                exit_callbacks
                    .lock()
                    .expect("exit callbacks lock")
                    .push("transition");
            })
        };

        view! {
            <Presence
                present=Signal::derive(move || present.get())
                on_exit_complete=on_exit_complete
            >
                <div id="presence-transition-child">"Child"</div>
            </Presence>
        }
    });

    let root = host
        .first_element_child()
        .expect("presence root")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("presence root html element");
    root.style()
        .set_property("transition-duration", "10s")
        .expect("set transition duration");

    present.set(false);
    assert_eq!(attr(&root, "data-state").as_deref(), Some("closed"));
    assert!(host.first_element_child().is_some());

    let child = html_element_by_id("presence-transition-child");
    dispatch_transition_end(&child, true);
    assert!(host.first_element_child().is_some());
    assert!(
        exit_callbacks
            .lock()
            .expect("exit callbacks lock")
            .is_empty()
    );

    dispatch_transition_end(&root, true);
    assert!(host.first_element_child().is_none());
    assert_eq!(
        exit_callbacks.lock().expect("exit callbacks lock").as_slice(),
        &["transition"]
    );

    drop(mount);
    remove_from_body(&host);
}

#[wasm_bindgen_test]
fn presence_ignores_bubbled_child_animationend_until_root_animationend_completes_exit() {
    let host = append_div("presence-animation-host");
    let present = RwSignal::new(true);
    let exit_callbacks: Arc<Mutex<Vec<&'static str>>> = Arc::new(Mutex::new(Vec::new()));
    let exit_callbacks_handle = Arc::clone(&exit_callbacks);

    let mount = mount_to(host.clone(), move || {
        let on_exit_complete = {
            let exit_callbacks = Arc::clone(&exit_callbacks_handle);
            Callback::new(move |_| {
                exit_callbacks
                    .lock()
                    .expect("exit callbacks lock")
                    .push("animation");
            })
        };

        view! {
            <Presence
                present=Signal::derive(move || present.get())
                on_exit_complete=on_exit_complete
            >
                <div id="presence-animation-child">"Child"</div>
            </Presence>
        }
    });

    let root = host
        .first_element_child()
        .expect("presence root")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("presence root html element");
    root.style()
        .set_property("animation-duration", "10s")
        .expect("set animation duration");

    present.set(false);
    assert_eq!(attr(&root, "data-state").as_deref(), Some("closed"));
    assert!(host.first_element_child().is_some());

    let child = html_element_by_id("presence-animation-child");
    dispatch_animation_end(&child, true);
    assert!(host.first_element_child().is_some());
    assert!(
        exit_callbacks
            .lock()
            .expect("exit callbacks lock")
            .is_empty()
    );

    dispatch_animation_end(&root, true);
    assert!(host.first_element_child().is_none());
    assert_eq!(
        exit_callbacks.lock().expect("exit callbacks lock").as_slice(),
        &["animation"]
    );

    drop(mount);
    remove_from_body(&host);
}

#[wasm_bindgen_test]
async fn presence_timeout_fallback_unmounts_and_runs_exit_complete_once() {
    let host = append_div("presence-timeout-host");
    let present = RwSignal::new(true);
    let exit_callbacks: Arc<Mutex<Vec<&'static str>>> = Arc::new(Mutex::new(Vec::new()));
    let exit_callbacks_handle = Arc::clone(&exit_callbacks);

    let mount = mount_to(host.clone(), move || {
        let on_exit_complete = {
            let exit_callbacks = Arc::clone(&exit_callbacks_handle);
            Callback::new(move |_| {
                exit_callbacks
                    .lock()
                    .expect("exit callbacks lock")
                    .push("timeout");
            })
        };

        view! {
            <Presence
                present=Signal::derive(move || present.get())
                on_exit_complete=on_exit_complete
            >
                <div id="presence-timeout-child">"Child"</div>
            </Presence>
        }
    });

    let root = host
        .first_element_child()
        .expect("presence root")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("presence root html element");
    root.style()
        .set_property("transition-duration", "20ms")
        .expect("set transition duration");

    present.set(false);
    assert_eq!(attr(&root, "data-state").as_deref(), Some("closed"));
    assert!(host.first_element_child().is_some());
    assert!(
        exit_callbacks
            .lock()
            .expect("exit callbacks lock")
            .is_empty()
    );

    TimeoutFuture::new(60).await;

    assert!(host.first_element_child().is_none());
    assert_eq!(
        exit_callbacks.lock().expect("exit callbacks lock").as_slice(),
        &["timeout"]
    );

    drop(mount);
    remove_from_body(&host);
}

#[wasm_bindgen_test]
async fn presence_reopen_clears_pending_timeout_and_allows_a_later_timeout_exit() {
    let host = append_div("presence-timeout-cancel-host");
    let present = RwSignal::new(true);
    let exit_callbacks: Arc<Mutex<Vec<&'static str>>> = Arc::new(Mutex::new(Vec::new()));
    let exit_callbacks_handle = Arc::clone(&exit_callbacks);

    let mount = mount_to(host.clone(), move || {
        let on_exit_complete = {
            let exit_callbacks = Arc::clone(&exit_callbacks_handle);
            Callback::new(move |_| {
                exit_callbacks
                    .lock()
                    .expect("exit callbacks lock")
                    .push("timeout");
            })
        };

        view! {
            <Presence
                present=Signal::derive(move || present.get())
                on_exit_complete=on_exit_complete
            >
                <div id="presence-timeout-cancel-child">"Child"</div>
            </Presence>
        }
    });

    let root = host
        .first_element_child()
        .expect("presence root")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("presence root html element");
    root.style()
        .set_property("transition-duration", "80ms")
        .expect("set transition duration");

    present.set(false);
    assert_eq!(attr(&root, "data-state").as_deref(), Some("closed"));
    assert!(host.first_element_child().is_some());

    TimeoutFuture::new(20).await;
    present.set(true);
    assert_eq!(attr(&root, "data-state").as_deref(), Some("open"));
    assert!(host.first_element_child().is_some());
    assert!(
        exit_callbacks
            .lock()
            .expect("exit callbacks lock")
            .is_empty()
    );

    TimeoutFuture::new(100).await;
    assert!(host.first_element_child().is_some());
    assert_eq!(attr(&root, "data-state").as_deref(), Some("open"));
    assert!(
        exit_callbacks
            .lock()
            .expect("exit callbacks lock")
            .is_empty()
    );

    present.set(false);
    assert_eq!(attr(&root, "data-state").as_deref(), Some("closed"));
    TimeoutFuture::new(120).await;

    assert!(host.first_element_child().is_none());
    assert_eq!(
        exit_callbacks.lock().expect("exit callbacks lock").as_slice(),
        &["timeout"]
    );

    drop(mount);
    remove_from_body(&host);
}

#[wasm_bindgen_test]
fn presence_reopen_ignores_stale_root_transitionend_and_allows_a_later_transition_exit() {
    let host = append_div("presence-stale-transition-host");
    let present = RwSignal::new(true);
    let exit_callbacks: Arc<Mutex<Vec<&'static str>>> = Arc::new(Mutex::new(Vec::new()));
    let exit_callbacks_handle = Arc::clone(&exit_callbacks);

    let mount = mount_to(host.clone(), move || {
        let on_exit_complete = {
            let exit_callbacks = Arc::clone(&exit_callbacks_handle);
            Callback::new(move |_| {
                exit_callbacks
                    .lock()
                    .expect("exit callbacks lock")
                    .push("transition");
            })
        };

        view! {
            <Presence
                present=Signal::derive(move || present.get())
                on_exit_complete=on_exit_complete
            >
                <div id="presence-stale-transition-child">"Child"</div>
            </Presence>
        }
    });

    let root = host
        .first_element_child()
        .expect("presence root")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("presence root html element");
    root.style()
        .set_property("transition-duration", "10s")
        .expect("set transition duration");

    present.set(false);
    assert_eq!(attr(&root, "data-state").as_deref(), Some("closed"));
    assert!(host.first_element_child().is_some());

    present.set(true);
    assert_eq!(attr(&root, "data-state").as_deref(), Some("open"));
    assert!(host.first_element_child().is_some());
    assert!(
        exit_callbacks
            .lock()
            .expect("exit callbacks lock")
            .is_empty()
    );

    dispatch_transition_end(&root, true);
    assert!(host.first_element_child().is_some());
    assert_eq!(attr(&root, "data-state").as_deref(), Some("open"));
    assert!(
        exit_callbacks
            .lock()
            .expect("exit callbacks lock")
            .is_empty()
    );

    present.set(false);
    assert_eq!(attr(&root, "data-state").as_deref(), Some("closed"));
    dispatch_transition_end(&root, true);

    assert!(host.first_element_child().is_none());
    assert_eq!(
        exit_callbacks.lock().expect("exit callbacks lock").as_slice(),
        &["transition"]
    );

    drop(mount);
    remove_from_body(&host);
}

#[wasm_bindgen_test]
fn presence_reopen_ignores_stale_root_animationend_and_allows_a_later_animation_exit() {
    let host = append_div("presence-stale-animation-host");
    let present = RwSignal::new(true);
    let exit_callbacks: Arc<Mutex<Vec<&'static str>>> = Arc::new(Mutex::new(Vec::new()));
    let exit_callbacks_handle = Arc::clone(&exit_callbacks);

    let mount = mount_to(host.clone(), move || {
        let on_exit_complete = {
            let exit_callbacks = Arc::clone(&exit_callbacks_handle);
            Callback::new(move |_| {
                exit_callbacks
                    .lock()
                    .expect("exit callbacks lock")
                    .push("animation");
            })
        };

        view! {
            <Presence
                present=Signal::derive(move || present.get())
                on_exit_complete=on_exit_complete
            >
                <div id="presence-stale-animation-child">"Child"</div>
            </Presence>
        }
    });

    let root = host
        .first_element_child()
        .expect("presence root")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("presence root html element");
    root.style()
        .set_property("animation-duration", "10s")
        .expect("set animation duration");

    present.set(false);
    assert_eq!(attr(&root, "data-state").as_deref(), Some("closed"));
    assert!(host.first_element_child().is_some());

    present.set(true);
    assert_eq!(attr(&root, "data-state").as_deref(), Some("open"));
    assert!(host.first_element_child().is_some());
    assert!(
        exit_callbacks
            .lock()
            .expect("exit callbacks lock")
            .is_empty()
    );

    dispatch_animation_end(&root, true);
    assert!(host.first_element_child().is_some());
    assert_eq!(attr(&root, "data-state").as_deref(), Some("open"));
    assert!(
        exit_callbacks
            .lock()
            .expect("exit callbacks lock")
            .is_empty()
    );

    present.set(false);
    assert_eq!(attr(&root, "data-state").as_deref(), Some("closed"));
    dispatch_animation_end(&root, true);

    assert!(host.first_element_child().is_none());
    assert_eq!(
        exit_callbacks.lock().expect("exit callbacks lock").as_slice(),
        &["animation"]
    );

    drop(mount);
    remove_from_body(&host);
}

#[wasm_bindgen_test]
fn presence_without_exit_motion_unmounts_immediately_and_runs_exit_complete_once() {
    let host = append_div("presence-immediate-host");
    let present = RwSignal::new(true);
    let exit_callbacks: Arc<Mutex<Vec<&'static str>>> = Arc::new(Mutex::new(Vec::new()));
    let exit_callbacks_handle = Arc::clone(&exit_callbacks);

    let mount = mount_to(host.clone(), move || {
        let on_exit_complete = {
            let exit_callbacks = Arc::clone(&exit_callbacks_handle);
            Callback::new(move |_| {
                exit_callbacks
                    .lock()
                    .expect("exit callbacks lock")
                    .push("immediate");
            })
        };

        view! {
            <Presence
                present=Signal::derive(move || present.get())
                on_exit_complete=on_exit_complete
            >
                <div id="presence-immediate-child">"Child"</div>
            </Presence>
        }
    });

    let root = host
        .first_element_child()
        .expect("presence root")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("presence root html element");
    root.style()
        .set_property("transition-duration", "0s")
        .expect("set transition duration");
    root.style()
        .set_property("animation-duration", "0s")
        .expect("set animation duration");

    present.set(false);

    assert!(host.first_element_child().is_none());
    assert_eq!(
        exit_callbacks.lock().expect("exit callbacks lock").as_slice(),
        &["immediate"]
    );

    drop(mount);
    remove_from_body(&host);
}

#[wasm_bindgen_test]
async fn presence_timeout_fallback_waits_for_the_longest_computed_exit_path() {
    let host = append_div("presence-longest-timeout-host");
    let present = RwSignal::new(true);
    let exit_callbacks: Arc<Mutex<Vec<&'static str>>> = Arc::new(Mutex::new(Vec::new()));
    let exit_callbacks_handle = Arc::clone(&exit_callbacks);

    let mount = mount_to(host.clone(), move || {
        let on_exit_complete = {
            let exit_callbacks = Arc::clone(&exit_callbacks_handle);
            Callback::new(move |_| {
                exit_callbacks
                    .lock()
                    .expect("exit callbacks lock")
                    .push("longest");
            })
        };

        view! {
            <Presence
                present=Signal::derive(move || present.get())
                on_exit_complete=on_exit_complete
            >
                <div id="presence-longest-timeout-child">"Child"</div>
            </Presence>
        }
    });

    let root = host
        .first_element_child()
        .expect("presence root")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("presence root html element");
    root.style()
        .set_property("transition-duration", "30ms")
        .expect("set transition duration");
    root.style()
        .set_property("transition-delay", "10ms")
        .expect("set transition delay");
    root.style()
        .set_property("animation-duration", "20ms")
        .expect("set animation duration");
    root.style()
        .set_property("animation-delay", "90ms")
        .expect("set animation delay");

    present.set(false);
    assert_eq!(attr(&root, "data-state").as_deref(), Some("closed"));
    assert!(host.first_element_child().is_some());
    assert!(
        exit_callbacks
            .lock()
            .expect("exit callbacks lock")
            .is_empty()
    );

    TimeoutFuture::new(70).await;

    assert!(host.first_element_child().is_some());
    assert!(
        exit_callbacks
            .lock()
            .expect("exit callbacks lock")
            .is_empty()
    );

    TimeoutFuture::new(80).await;

    assert!(host.first_element_child().is_none());
    assert_eq!(
        exit_callbacks.lock().expect("exit callbacks lock").as_slice(),
        &["longest"]
    );

    drop(mount);
    remove_from_body(&host);
}

#[wasm_bindgen_test]
fn scroll_lock_release_restores_previous_body_styles() {
    let overflow_before = body_style("overflow");
    let position_before = body_style("position");
    let top_before = body_style("top");
    let width_before = body_style("width");

    set_body_style("overflow", "scroll");
    set_body_style("position", "relative");
    set_body_style("top", "12px");
    set_body_style("width", "75%");

    let guard = scroll_lock_acquire().expect("acquire scroll lock");

    assert_eq!(body_style("overflow"), "hidden");
    assert_eq!(body_style("position"), "fixed");
    assert_eq!(body_style("width"), "100%");
    assert_ne!(body_style("top"), "12px");

    scroll_lock_release().expect("release scroll lock");

    assert_eq!(body_style("overflow"), "scroll");
    assert_eq!(body_style("position"), "relative");
    assert_eq!(body_style("top"), "12px");
    assert_eq!(body_style("width"), "75%");

    drop(guard);

    set_body_style("overflow", &overflow_before);
    set_body_style("position", &position_before);
    set_body_style("top", &top_before);
    set_body_style("width", &width_before);
}

#[wasm_bindgen_test]
fn nested_scroll_lock_guards_preserve_body_lock_until_the_last_drop() {
    let overflow_before = body_style("overflow");
    let position_before = body_style("position");
    let top_before = body_style("top");
    let width_before = body_style("width");

    set_body_style("overflow", "auto");
    set_body_style("position", "absolute");
    set_body_style("top", "8px");
    set_body_style("width", "60%");

    let outer_guard = scroll_lock_acquire().expect("acquire outer scroll lock");
    let inner_guard = scroll_lock_acquire().expect("acquire inner scroll lock");

    assert_eq!(body_style("overflow"), "hidden");
    assert_eq!(body_style("position"), "fixed");
    assert_eq!(body_style("width"), "100%");

    drop(inner_guard);

    assert_eq!(body_style("overflow"), "hidden");
    assert_eq!(body_style("position"), "fixed");
    assert_eq!(body_style("width"), "100%");

    drop(outer_guard);

    assert_eq!(body_style("overflow"), "auto");
    assert_eq!(body_style("position"), "absolute");
    assert_eq!(body_style("top"), "8px");
    assert_eq!(body_style("width"), "60%");

    set_body_style("overflow", &overflow_before);
    set_body_style("position", &position_before);
    set_body_style("top", &top_before);
    set_body_style("width", &width_before);
}

#[wasm_bindgen_test]
fn mixed_scroll_lock_release_and_guard_drop_restore_styles_after_the_final_logical_release() {
    let overflow_before = body_style("overflow");
    let position_before = body_style("position");
    let top_before = body_style("top");
    let width_before = body_style("width");

    set_body_style("overflow", "clip");
    set_body_style("position", "sticky");
    set_body_style("top", "6px");
    set_body_style("width", "55%");

    let outer_guard = scroll_lock_acquire().expect("acquire outer scroll lock");
    let inner_guard = scroll_lock_acquire().expect("acquire inner scroll lock");

    assert_eq!(body_style("overflow"), "hidden");
    assert_eq!(body_style("position"), "fixed");
    assert_eq!(body_style("width"), "100%");

    scroll_lock_release().expect("release one scroll lock");

    assert_eq!(body_style("overflow"), "hidden");
    assert_eq!(body_style("position"), "fixed");
    assert_eq!(body_style("width"), "100%");

    drop(outer_guard);

    assert_eq!(body_style("overflow"), "clip");
    assert_eq!(body_style("position"), "sticky");
    assert_eq!(body_style("top"), "6px");
    assert_eq!(body_style("width"), "55%");

    drop(inner_guard);

    assert_eq!(body_style("overflow"), "clip");
    assert_eq!(body_style("position"), "sticky");
    assert_eq!(body_style("top"), "6px");
    assert_eq!(body_style("width"), "55%");

    set_body_style("overflow", &overflow_before);
    set_body_style("position", &position_before);
    set_body_style("top", &top_before);
    set_body_style("width", &width_before);
}

#[wasm_bindgen_test]
fn scroll_lock_reacquire_captures_a_fresh_body_style_snapshot() {
    let overflow_before = body_style("overflow");
    let position_before = body_style("position");
    let top_before = body_style("top");
    let width_before = body_style("width");

    set_body_style("overflow", "hidden");
    set_body_style("position", "relative");
    set_body_style("top", "4px");
    set_body_style("width", "70%");

    let first_guard = scroll_lock_acquire().expect("acquire first scroll lock");
    assert_eq!(body_style("position"), "fixed");
    assert_eq!(body_style("width"), "100%");

    drop(first_guard);

    assert_eq!(body_style("overflow"), "hidden");
    assert_eq!(body_style("position"), "relative");
    assert_eq!(body_style("top"), "4px");
    assert_eq!(body_style("width"), "70%");

    set_body_style("overflow", "visible");
    set_body_style("position", "absolute");
    set_body_style("top", "14px");
    set_body_style("width", "45%");

    let second_guard = scroll_lock_acquire().expect("acquire second scroll lock");
    assert_eq!(body_style("overflow"), "hidden");
    assert_eq!(body_style("position"), "fixed");
    assert_eq!(body_style("width"), "100%");

    drop(second_guard);

    assert_eq!(body_style("overflow"), "visible");
    assert_eq!(body_style("position"), "absolute");
    assert_eq!(body_style("top"), "14px");
    assert_eq!(body_style("width"), "45%");

    set_body_style("overflow", &overflow_before);
    set_body_style("position", &position_before);
    set_body_style("top", &top_before);
    set_body_style("width", &width_before);
}

#[wasm_bindgen_test]
fn extra_scroll_lock_release_after_full_unlock_leaves_restored_styles_unchanged() {
    let overflow_before = body_style("overflow");
    let position_before = body_style("position");
    let top_before = body_style("top");
    let width_before = body_style("width");

    set_body_style("overflow", "overlay");
    set_body_style("position", "relative");
    set_body_style("top", "11px");
    set_body_style("width", "52%");

    let guard = scroll_lock_acquire().expect("acquire scroll lock");

    assert_eq!(body_style("overflow"), "hidden");
    assert_eq!(body_style("position"), "fixed");
    assert_eq!(body_style("width"), "100%");

    drop(guard);

    assert_eq!(body_style("overflow"), "overlay");
    assert_eq!(body_style("position"), "relative");
    assert_eq!(body_style("top"), "11px");
    assert_eq!(body_style("width"), "52%");

    scroll_lock_release().expect("extra release on unlocked body");

    assert_eq!(body_style("overflow"), "overlay");
    assert_eq!(body_style("position"), "relative");
    assert_eq!(body_style("top"), "11px");
    assert_eq!(body_style("width"), "52%");

    set_body_style("overflow", &overflow_before);
    set_body_style("position", &position_before);
    set_body_style("top", &top_before);
    set_body_style("width", &width_before);
}

#[wasm_bindgen_test]
fn scroll_lock_restores_the_original_snapshot_after_mid_lock_style_mutation() {
    let overflow_before = body_style("overflow");
    let position_before = body_style("position");
    let top_before = body_style("top");
    let width_before = body_style("width");

    set_body_style("overflow", "visible");
    set_body_style("position", "relative");
    set_body_style("top", "9px");
    set_body_style("width", "41%");

    let guard = scroll_lock_acquire().expect("acquire scroll lock");

    assert_eq!(body_style("overflow"), "hidden");
    assert_eq!(body_style("position"), "fixed");
    assert_eq!(body_style("width"), "100%");

    set_body_style("overflow", "clip");
    set_body_style("position", "absolute");
    set_body_style("top", "21px");
    set_body_style("width", "88%");

    assert_eq!(body_style("overflow"), "clip");
    assert_eq!(body_style("position"), "absolute");
    assert_eq!(body_style("top"), "21px");
    assert_eq!(body_style("width"), "88%");

    drop(guard);

    assert_eq!(body_style("overflow"), "visible");
    assert_eq!(body_style("position"), "relative");
    assert_eq!(body_style("top"), "9px");
    assert_eq!(body_style("width"), "41%");

    set_body_style("overflow", &overflow_before);
    set_body_style("position", &position_before);
    set_body_style("top", &top_before);
    set_body_style("width", &width_before);
}

#[wasm_bindgen_test]
async fn scroll_lock_restores_the_captured_scroll_position_on_final_unlock() {
    let overflow_before = body_style("overflow");
    let position_before = body_style("position");
    let top_before = body_style("top");
    let width_before = body_style("width");
    let scroll_before = scroll_y();

    let spacer = append_div("scroll-lock-scroll-spacer");
    spacer
        .style()
        .set_property("height", "4000px")
        .expect("set spacer height");
    spacer
        .style()
        .set_property("width", "1px")
        .expect("set spacer width");

    set_body_style("overflow", "auto");
    set_body_style("position", "relative");
    set_body_style("top", "13px");
    set_body_style("width", "64%");

    window().scroll_to_with_x_and_y(0.0, 240.0);
    TimeoutFuture::new(20).await;

    let captured_scroll = scroll_y();
    assert!(captured_scroll >= 200.0, "captured scroll: {captured_scroll}");

    let guard = scroll_lock_acquire().expect("acquire scroll lock");

    assert_eq!(body_style("overflow"), "hidden");
    assert_eq!(body_style("position"), "fixed");
    assert_eq!(body_style("width"), "100%");
    assert_eq!(body_style("top"), format!("-{}px", captured_scroll));

    window().scroll_to_with_x_and_y(0.0, 0.0);
    TimeoutFuture::new(20).await;

    drop(guard);
    TimeoutFuture::new(20).await;

    assert_eq!(body_style("overflow"), "auto");
    assert_eq!(body_style("position"), "relative");
    assert_eq!(body_style("top"), "13px");
    assert_eq!(body_style("width"), "64%");
    assert!((scroll_y() - captured_scroll).abs() < 1.0);

    set_body_style("overflow", &overflow_before);
    set_body_style("position", &position_before);
    set_body_style("top", &top_before);
    set_body_style("width", &width_before);
    window().scroll_to_with_x_and_y(0.0, scroll_before);
    TimeoutFuture::new(20).await;
    remove_from_body(&spacer);
}

#[wasm_bindgen_test]
async fn scroll_lock_reacquire_captures_a_fresh_scroll_snapshot() {
    let overflow_before = body_style("overflow");
    let position_before = body_style("position");
    let top_before = body_style("top");
    let width_before = body_style("width");
    let scroll_before = scroll_y();

    let spacer = append_div("scroll-lock-reacquire-scroll-spacer");
    spacer
        .style()
        .set_property("height", "5000px")
        .expect("set spacer height");
    spacer
        .style()
        .set_property("width", "1px")
        .expect("set spacer width");

    set_body_style("overflow", "auto");
    set_body_style("position", "relative");
    set_body_style("top", "10px");
    set_body_style("width", "58%");

    window().scroll_to_with_x_and_y(0.0, 180.0);
    TimeoutFuture::new(20).await;
    let first_scroll = scroll_y();
    assert!(first_scroll >= 150.0, "first scroll: {first_scroll}");

    let first_guard = scroll_lock_acquire().expect("acquire first scroll lock");
    assert_eq!(body_style("top"), format!("-{}px", first_scroll));

    drop(first_guard);
    TimeoutFuture::new(20).await;

    assert_eq!(body_style("overflow"), "auto");
    assert_eq!(body_style("position"), "relative");
    assert_eq!(body_style("top"), "10px");
    assert_eq!(body_style("width"), "58%");
    assert!((scroll_y() - first_scroll).abs() < 1.0);

    set_body_style("overflow", "scroll");
    set_body_style("position", "absolute");
    set_body_style("top", "17px");
    set_body_style("width", "46%");

    window().scroll_to_with_x_and_y(0.0, 420.0);
    TimeoutFuture::new(20).await;
    let second_scroll = scroll_y();
    assert!(
        second_scroll >= first_scroll + 150.0,
        "second scroll: {second_scroll}, first scroll: {first_scroll}"
    );

    let second_guard = scroll_lock_acquire().expect("acquire second scroll lock");

    assert_eq!(body_style("overflow"), "hidden");
    assert_eq!(body_style("position"), "fixed");
    assert_eq!(body_style("width"), "100%");
    assert_eq!(body_style("top"), format!("-{}px", second_scroll));

    window().scroll_to_with_x_and_y(0.0, 0.0);
    TimeoutFuture::new(20).await;

    drop(second_guard);
    TimeoutFuture::new(20).await;

    assert_eq!(body_style("overflow"), "scroll");
    assert_eq!(body_style("position"), "absolute");
    assert_eq!(body_style("top"), "17px");
    assert_eq!(body_style("width"), "46%");
    assert!((scroll_y() - second_scroll).abs() < 1.0);

    set_body_style("overflow", &overflow_before);
    set_body_style("position", &position_before);
    set_body_style("top", &top_before);
    set_body_style("width", &width_before);
    window().scroll_to_with_x_and_y(0.0, scroll_before);
    TimeoutFuture::new(20).await;
    remove_from_body(&spacer);
}

#[wasm_bindgen_test]
async fn nested_scroll_lock_keeps_the_outer_scroll_snapshot_until_final_unlock() {
    let overflow_before = body_style("overflow");
    let position_before = body_style("position");
    let top_before = body_style("top");
    let width_before = body_style("width");
    let scroll_before = scroll_y();

    let spacer = append_div("nested-scroll-lock-scroll-spacer");
    spacer
        .style()
        .set_property("height", "5000px")
        .expect("set spacer height");
    spacer
        .style()
        .set_property("width", "1px")
        .expect("set spacer width");

    set_body_style("overflow", "auto");
    set_body_style("position", "relative");
    set_body_style("top", "15px");
    set_body_style("width", "54%");

    window().scroll_to_with_x_and_y(0.0, 260.0);
    TimeoutFuture::new(20).await;
    let outer_scroll = scroll_y();
    assert!(outer_scroll >= 220.0, "outer scroll: {outer_scroll}");

    let outer_guard = scroll_lock_acquire().expect("acquire outer scroll lock");

    assert_eq!(body_style("overflow"), "hidden");
    assert_eq!(body_style("position"), "fixed");
    assert_eq!(body_style("width"), "100%");
    assert_eq!(body_style("top"), format!("-{}px", outer_scroll));

    window().scroll_to_with_x_and_y(0.0, 0.0);
    TimeoutFuture::new(20).await;

    let inner_guard = scroll_lock_acquire().expect("acquire inner scroll lock");

    assert_eq!(body_style("overflow"), "hidden");
    assert_eq!(body_style("position"), "fixed");
    assert_eq!(body_style("width"), "100%");
    assert_eq!(body_style("top"), format!("-{}px", outer_scroll));

    drop(inner_guard);
    TimeoutFuture::new(20).await;

    assert_eq!(body_style("overflow"), "hidden");
    assert_eq!(body_style("position"), "fixed");
    assert_eq!(body_style("width"), "100%");
    assert_eq!(body_style("top"), format!("-{}px", outer_scroll));

    drop(outer_guard);
    TimeoutFuture::new(20).await;

    assert_eq!(body_style("overflow"), "auto");
    assert_eq!(body_style("position"), "relative");
    assert_eq!(body_style("top"), "15px");
    assert_eq!(body_style("width"), "54%");
    assert!((scroll_y() - outer_scroll).abs() < 1.0);

    set_body_style("overflow", &overflow_before);
    set_body_style("position", &position_before);
    set_body_style("top", &top_before);
    set_body_style("width", &width_before);
    window().scroll_to_with_x_and_y(0.0, scroll_before);
    TimeoutFuture::new(20).await;
    remove_from_body(&spacer);
}
