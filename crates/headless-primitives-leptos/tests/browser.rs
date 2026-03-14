#![cfg(target_arch = "wasm32")]

use headless_primitives_leptos::{
    DismissibleLayer, DismissibleReason, FocusScope, Portal, Presence, modal_hide_siblings,
    scroll_lock_acquire, scroll_lock_release,
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
