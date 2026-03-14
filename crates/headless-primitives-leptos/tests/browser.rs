#![cfg(target_arch = "wasm32")]

use headless_primitives_leptos::{
    DismissibleLayer, DismissibleReason, FocusScope, Portal, modal_hide_siblings,
};
use leptos::mount::mount_to;
use leptos::prelude::*;
use std::sync::{Arc, Mutex};
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

fn document() -> web_sys::Document {
    web_sys::window()
        .expect("window")
        .document()
        .expect("document")
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

fn dispatch_tab_keydown(target: &web_sys::HtmlElement, shift: bool) -> web_sys::KeyboardEvent {
    let init = web_sys::KeyboardEventInit::new();
    init.set_bubbles(true);
    init.set_cancelable(true);
    init.set_key("Tab");
    init.set_shift_key(shift);
    let event = web_sys::KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init)
        .expect("keyboard event");
    target.dispatch_event(&event).expect("dispatch keydown");
    event
}

fn attr(element: &web_sys::HtmlElement, name: &str) -> Option<String> {
    element.get_attribute(name)
}

fn active_id() -> Option<String> {
    document().active_element().map(|element| element.id())
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
