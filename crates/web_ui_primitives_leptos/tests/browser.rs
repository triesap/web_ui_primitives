#![cfg(target_arch = "wasm32")]

use gloo_timers::future::TimeoutFuture;
use leptos::html;
use leptos::mount::mount_to;
use leptos::prelude::*;
use std::cell::Cell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use wasm_bindgen_test::*;
use web_ui_primitives_leptos::{
    DialogLayerOptions, DismissibleEscapeKeyDownEvent, DismissibleFocusOutsideEvent,
    DismissibleLayer, DismissibleLayerOptions, DismissiblePointerDownOutsideEvent,
    DismissibleReason, FocusScope, FocusScopeOptions, MenuLayerOptions, Portal, Presence,
    modal_hide_siblings, scroll_lock_acquire, scroll_lock_release, use_dialog_layer,
    use_dismissible_layer, use_focus_scope, use_menu_layer, use_presence,
};

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

fn focus_element(target: &web_sys::HtmlElement) {
    let observed = Rc::new(Cell::new(false));
    let observed_by_listener = Rc::clone(&observed);
    let listener = Closure::wrap(Box::new(move |_event: web_sys::FocusEvent| {
        observed_by_listener.set(true);
    }) as Box<dyn FnMut(_)>);
    let document = document();
    document
        .add_event_listener_with_callback("focusin", listener.as_ref().unchecked_ref())
        .expect("attach focus observation");

    target.focus().expect("focus element");

    document
        .remove_event_listener_with_callback("focusin", listener.as_ref().unchecked_ref())
        .expect("detach focus observation");
    if observed.get() {
        return;
    }

    let init = web_sys::FocusEventInit::new();
    init.set_bubbles(true);
    init.set_cancelable(false);
    init.set_composed(true);
    let event =
        web_sys::FocusEvent::new_with_focus_event_init_dict("focusin", &init).expect("focus event");
    target.dispatch_event(&event).expect("dispatch focusin");
}

async fn render_tick() {
    TimeoutFuture::new(0).await;
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

fn dispatch_pointer_down(target: &web_sys::HtmlElement) -> web_sys::PointerEvent {
    let init = web_sys::PointerEventInit::new();
    init.set_bubbles(true);
    init.set_cancelable(true);
    init.set_composed(true);
    let event = web_sys::PointerEvent::new_with_event_init_dict("pointerdown", &init)
        .expect("pointer event");
    target.dispatch_event(&event).expect("dispatch pointerdown");
    event
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
    init.set_property_name("opacity");
    let event = web_sys::TransitionEvent::new_with_event_init_dict("transitionend", &init)
        .expect("transition event");
    target
        .dispatch_event(&event)
        .expect("dispatch transitionend");
}

fn dispatch_transition_cancel(target: &web_sys::HtmlElement, property: &str) {
    let init = web_sys::TransitionEventInit::new();
    init.set_bubbles(true);
    init.set_property_name(property);
    let event = web_sys::TransitionEvent::new_with_event_init_dict("transitioncancel", &init)
        .expect("transition cancel event");
    target
        .dispatch_event(&event)
        .expect("dispatch transitioncancel");
}

fn dispatch_animation_end(target: &web_sys::HtmlElement, bubbles: bool) {
    let init = web_sys::AnimationEventInit::new();
    init.set_bubbles(bubbles);
    init.set_animation_name("fade");
    let event = web_sys::AnimationEvent::new_with_event_init_dict("animationend", &init)
        .expect("animation event");
    target
        .dispatch_event(&event)
        .expect("dispatch animationend");
}

fn dispatch_animation_cancel(target: &web_sys::HtmlElement, name: &str) {
    let init = web_sys::AnimationEventInit::new();
    init.set_bubbles(true);
    init.set_animation_name(name);
    let event = web_sys::AnimationEvent::new_with_event_init_dict("animationcancel", &init)
        .expect("animation cancel event");
    target
        .dispatch_event(&event)
        .expect("dispatch animationcancel");
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

fn browser_dialog_layer(
    open: RwSignal<bool>,
    dismissals: Arc<Mutex<Vec<DismissibleReason>>>,
) -> impl IntoView {
    let open_signal = Signal::derive(move || open.get());
    let mut options = DialogLayerOptions::new(open_signal);
    options.on_dismiss = Some(Callback::new(move |reason| {
        dismissals.lock().expect("dismissals lock").push(reason);
        open.set(false);
    }));
    let dialog = use_dialog_layer::<html::Div>(options);
    let node_ref = dialog.node_ref();

    view! {
        <Portal>
            <div
                id="dialog-layer-root"
                node_ref=node_ref
                role="dialog"
                aria-label="Test dialog"
                tabindex="-1"
                data-state="open"
            >
                <button id="dialog-layer-inside">"Inside"</button>
            </div>
        </Portal>
    }
}

fn browser_readiness_dialog_surface(
    node_ref: NodeRef<html::Div>,
    open: Signal<bool>,
    transition_end: Callback<leptos::ev::TransitionEvent>,
    transition_cancel: Callback<leptos::ev::TransitionEvent>,
    animation_end: Callback<leptos::ev::AnimationEvent>,
    animation_cancel: Callback<leptos::ev::AnimationEvent>,
) -> impl IntoView {
    view! {
        <div
            id="dialog-readiness-root"
            node_ref=node_ref
            role="dialog"
            aria-label="Readiness dialog"
            tabindex="-1"
            data-state=move || if open.get() { "open" } else { "closed" }
            style="transition-property: opacity; transition-duration: 10ms;"
            on:transitionend=move |event| transition_end.run(event)
            on:transitioncancel=move |event| transition_cancel.run(event)
            on:animationend=move |event| animation_end.run(event)
            on:animationcancel=move |event| animation_cancel.run(event)
        >
            <button id="dialog-readiness-first">"First"</button>
            <button id="dialog-readiness-second">"Second"</button>
        </div>
    }
}

fn browser_readiness_dialog_layer(
    open: RwSignal<bool>,
    branch: web_sys::Element,
    dismissals: Arc<Mutex<Vec<DismissibleReason>>>,
    pointer_targets: Arc<Mutex<Vec<String>>>,
    focus_targets: Arc<Mutex<Vec<String>>>,
) -> impl IntoView {
    let open_signal = Signal::derive(move || open.get());
    let data_state_open = open_signal;
    let mut options = DialogLayerOptions::new(open_signal);
    options.branches = vec![branch];
    options.on_dismiss = Some(Callback::new(move |reason| {
        dismissals.lock().expect("dismissals lock").push(reason);
        open.set(false);
    }));
    options.on_pointer_down_outside = Some(Callback::new(
        move |event: DismissiblePointerDownOutsideEvent| {
            let target_id = event
                .event()
                .target()
                .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                .map(|element| element.id())
                .unwrap_or_default();
            pointer_targets
                .lock()
                .expect("pointer targets lock")
                .push(target_id);
        },
    ));
    options.on_focus_outside = Some(Callback::new(move |event: DismissibleFocusOutsideEvent| {
        let target_id = event
            .event()
            .target()
            .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
            .map(|element| element.id())
            .unwrap_or_default();
        focus_targets
            .lock()
            .expect("focus targets lock")
            .push(target_id);
    }));
    let dialog = use_dialog_layer::<html::Div>(options);

    view! {
        {move || -> AnyView {
            if !dialog.is_rendered() {
                ().into_any()
            } else {
                let node_ref = dialog.node_ref();
                let transition_end = dialog.transition_end_handler();
                let transition_cancel = dialog.transition_cancel_handler();
                let animation_end = dialog.animation_end_handler();
                let animation_cancel = dialog.animation_cancel_handler();

                view! {
                    <Portal>
                        {move || browser_readiness_dialog_surface(
                            node_ref,
                            data_state_open,
                            transition_end,
                            transition_cancel,
                            animation_end,
                            animation_cancel,
                        )}
                    </Portal>
                }
                .into_any()
            }
        }}
    }
}

fn browser_menu_layer(
    open: RwSignal<bool>,
    dismissals: Arc<Mutex<Vec<DismissibleReason>>>,
) -> impl IntoView {
    let open_signal = Signal::derive(move || open.get());
    let mut options = MenuLayerOptions::new(open_signal);
    options.on_dismiss = Some(Callback::new(move |reason| {
        dismissals.lock().expect("dismissals lock").push(reason);
        open.set(false);
    }));
    let menu = use_menu_layer::<html::Div>(options);

    view! {
        {move || -> AnyView {
            if !menu.is_rendered() {
                ().into_any()
            } else {
                let node_ref = menu.node_ref();
                let transition_end = menu.transition_end_handler();
                let transition_cancel = menu.transition_cancel_handler();
                let animation_end = menu.animation_end_handler();
                let animation_cancel = menu.animation_cancel_handler();

                view! {
                    <div
                        id="menu-layer-root"
                        node_ref=node_ref
                        role="menu"
                        tabindex="-1"
                        data-state=menu.data_state()
                        on:transitionend=move |event| transition_end.run(event)
                        on:transitioncancel=move |event| transition_cancel.run(event)
                        on:animationend=move |event| animation_end.run(event)
                        on:animationcancel=move |event| animation_cancel.run(event)
                    >
                        <button id="menu-layer-first" type="button">"English"</button>
                        <button id="menu-layer-second" type="button">"Spanish"</button>
                    </div>
                }
                .into_any()
            }
        }}
    }
}

#[wasm_bindgen_test]
async fn dialog_layer_composes_modal_focus_dismissible_portal_and_scroll_lock() {
    let host = append_div("dialog-layer-host");
    let outside = append_button("dialog-layer-outside");
    let original_overflow = body_style("overflow");
    let original_position = body_style("position");
    let open = RwSignal::new(false);
    let dismissals: Arc<Mutex<Vec<DismissibleReason>>> = Arc::new(Mutex::new(Vec::new()));
    let dismissals_handle = Arc::clone(&dismissals);

    let mount = mount_to(host.clone(), move || {
        let dismissals = Arc::clone(&dismissals_handle);

        view! {
            <button id="dialog-layer-trigger">"Open"</button>
            {move || -> AnyView {
                if open.get() {
                    browser_dialog_layer(open, Arc::clone(&dismissals)).into_any()
                } else {
                    ().into_any()
                }
            }}
        }
    });

    render_tick().await;
    let trigger = html_element_by_id("dialog-layer-trigger");
    focus_element(&trigger);
    open.set(true);
    render_tick().await;
    render_tick().await;

    let inside = html_element_by_id("dialog-layer-inside");
    assert_eq!(active_id().as_deref(), Some("dialog-layer-inside"));
    assert_eq!(attr(&outside, "aria-hidden").as_deref(), Some("true"));
    assert!(outside.has_attribute("inert"));
    assert_eq!(body_style("overflow"), "hidden");
    assert_eq!(body_style("position"), "fixed");

    dispatch_escape_keydown(&inside);
    render_tick().await;
    render_tick().await;

    assert_eq!(
        dismissals.lock().expect("dismissals lock").as_slice(),
        &[DismissibleReason::Escape]
    );
    assert!(document().get_element_by_id("dialog-layer-root").is_none());
    assert_eq!(attr(&outside, "aria-hidden"), None);
    assert!(!outside.has_attribute("inert"));
    assert_eq!(body_style("overflow"), original_overflow);
    assert_eq!(body_style("position"), original_position);
    assert_eq!(active_id().as_deref(), Some("dialog-layer-trigger"));

    drop(mount);
    render_tick().await;
    remove_from_body(&host);
    remove_from_body(&outside);
}

#[wasm_bindgen_test]
async fn menu_layer_composes_presence_focus_and_dismissible_behavior() {
    let host = append_div("menu-layer-host");
    let outside = append_button("menu-layer-outside");
    let open = RwSignal::new(false);
    let dismissals: Arc<Mutex<Vec<DismissibleReason>>> = Arc::new(Mutex::new(Vec::new()));
    let dismissals_handle = Arc::clone(&dismissals);

    let mount = mount_to(host.clone(), move || {
        let dismissals = Arc::clone(&dismissals_handle);

        view! {
            <button id="menu-layer-trigger">"Locale"</button>
            {move || browser_menu_layer(open, Arc::clone(&dismissals))}
        }
    });

    render_tick().await;
    let trigger = html_element_by_id("menu-layer-trigger");
    focus_element(&trigger);
    open.set(true);
    render_tick().await;
    render_tick().await;

    let first = html_element_by_id("menu-layer-first");
    assert_eq!(active_id().as_deref(), Some("menu-layer-first"));
    assert_eq!(
        attr(&html_element_by_id("menu-layer-root"), "data-state").as_deref(),
        Some("open")
    );

    dispatch_escape_keydown(&first);
    TimeoutFuture::new(40).await;
    render_tick().await;
    assert_eq!(
        dismissals.lock().expect("dismissals lock").as_slice(),
        &[DismissibleReason::Escape]
    );
    assert_eq!(active_id().as_deref(), Some("menu-layer-trigger"));
    assert!(document().get_element_by_id("menu-layer-root").is_none());

    focus_element(&trigger);
    open.set(true);
    render_tick().await;
    render_tick().await;
    assert_eq!(active_id().as_deref(), Some("menu-layer-first"));

    dispatch_pointer_down(&outside);
    TimeoutFuture::new(40).await;
    render_tick().await;
    assert_eq!(
        dismissals.lock().expect("dismissals lock").as_slice(),
        &[
            DismissibleReason::Escape,
            DismissibleReason::PointerDownOutside
        ]
    );
    assert_eq!(active_id().as_deref(), Some("menu-layer-trigger"));
    assert!(document().get_element_by_id("menu-layer-root").is_none());

    drop(mount);
    render_tick().await;
    remove_from_body(&host);
    remove_from_body(&outside);
}

#[wasm_bindgen_test]
async fn dialog_layer_readiness_fixture_covers_overlay_dismissal_and_exit_presence() {
    let host = append_div("dialog-readiness-host");
    let branch = append_button("dialog-readiness-branch");
    let outside = append_button("dialog-readiness-outside");
    let return_target = append_button("dialog-readiness-return");
    let original_overflow = body_style("overflow");
    let original_position = body_style("position");
    let open = RwSignal::new(false);
    let layer_mounted = RwSignal::new(false);
    let branch_element: web_sys::Element = branch.clone().into();
    let dismissals: Arc<Mutex<Vec<DismissibleReason>>> = Arc::new(Mutex::new(Vec::new()));
    let pointer_targets: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let focus_targets: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let dismissals_handle = Arc::clone(&dismissals);
    let pointer_targets_handle = Arc::clone(&pointer_targets);
    let focus_targets_handle = Arc::clone(&focus_targets);

    let mount = mount_to(host.clone(), move || {
        let dismissals = Arc::clone(&dismissals_handle);
        let pointer_targets = Arc::clone(&pointer_targets_handle);
        let focus_targets = Arc::clone(&focus_targets_handle);
        let branch_element = branch_element.clone();

        view! {
            <button id="dialog-readiness-trigger">"Open"</button>
            {move || -> AnyView {
                if layer_mounted.get() {
                    browser_readiness_dialog_layer(
                        open,
                        branch_element.clone(),
                        Arc::clone(&dismissals),
                        Arc::clone(&pointer_targets),
                        Arc::clone(&focus_targets),
                    )
                    .into_any()
                } else {
                    ().into_any()
                }
            }}
        }
    });

    render_tick().await;
    focus_element(&return_target);
    layer_mounted.set(true);
    render_tick().await;
    open.set(true);
    render_tick().await;
    render_tick().await;

    let root = html_element_by_id("dialog-readiness-root");
    let first = html_element_by_id("dialog-readiness-first");
    assert_eq!(active_id().as_deref(), Some("dialog-readiness-first"));
    assert_eq!(attr(&root, "data-state").as_deref(), Some("open"));
    assert_eq!(attr(&outside, "aria-hidden").as_deref(), Some("true"));
    assert!(outside.has_attribute("inert"));
    assert_eq!(body_style("overflow"), "hidden");
    assert_eq!(body_style("position"), "fixed");

    let tab = dispatch_tab_keydown(&first, false);
    render_tick().await;
    assert!(tab.default_prevented());
    assert_eq!(active_id().as_deref(), Some("dialog-readiness-second"));

    dispatch_pointer_down(&branch);
    render_tick().await;
    assert!(dismissals.lock().expect("dismissals lock").is_empty());

    dispatch_pointer_down(&outside);
    render_tick().await;
    assert_eq!(
        dismissals.lock().expect("dismissals lock").as_slice(),
        &[DismissibleReason::PointerDownOutside]
    );
    assert_eq!(
        pointer_targets
            .lock()
            .expect("pointer targets lock")
            .as_slice(),
        &["dialog-readiness-outside".to_string()]
    );
    assert_eq!(attr(&root, "data-state").as_deref(), Some("closed"));
    assert!(
        document()
            .get_element_by_id("dialog-readiness-root")
            .is_some()
    );
    assert_eq!(attr(&outside, "aria-hidden"), None);
    assert!(!outside.has_attribute("inert"));
    assert_eq!(body_style("overflow"), original_overflow);
    assert_eq!(body_style("position"), original_position);

    TimeoutFuture::new(100).await;
    render_tick().await;
    assert!(
        document()
            .get_element_by_id("dialog-readiness-root")
            .is_none()
    );
    render_tick().await;
    assert_eq!(active_id().as_deref(), Some("dialog-readiness-return"));

    open.set(true);
    render_tick().await;
    render_tick().await;
    assert_eq!(active_id().as_deref(), Some("dialog-readiness-first"));
    focus_element(&outside);
    render_tick().await;
    let root = html_element_by_id("dialog-readiness-root");
    assert_eq!(
        dismissals.lock().expect("dismissals lock").as_slice(),
        &[
            DismissibleReason::PointerDownOutside,
            DismissibleReason::FocusOutside
        ]
    );
    assert_eq!(
        focus_targets.lock().expect("focus targets lock").as_slice(),
        &["dialog-readiness-outside".to_string()]
    );
    assert_eq!(attr(&root, "data-state").as_deref(), Some("closed"));
    TimeoutFuture::new(100).await;
    render_tick().await;
    assert!(
        document()
            .get_element_by_id("dialog-readiness-root")
            .is_none()
    );

    open.set(true);
    render_tick().await;
    render_tick().await;
    let first = html_element_by_id("dialog-readiness-first");
    dispatch_escape_keydown(&first);
    render_tick().await;
    let root = html_element_by_id("dialog-readiness-root");
    assert_eq!(
        dismissals.lock().expect("dismissals lock").as_slice(),
        &[
            DismissibleReason::PointerDownOutside,
            DismissibleReason::FocusOutside,
            DismissibleReason::Escape
        ]
    );
    assert_eq!(attr(&root, "data-state").as_deref(), Some("closed"));
    TimeoutFuture::new(100).await;
    render_tick().await;
    assert!(
        document()
            .get_element_by_id("dialog-readiness-root")
            .is_none()
    );

    drop(mount);
    render_tick().await;
    remove_from_body(&host);
    remove_from_body(&branch);
    remove_from_body(&outside);
    remove_from_body(&return_target);
}

#[wasm_bindgen_test]
async fn dismissible_layer_binding_attaches_to_the_target_element_without_a_wrapper() {
    let host = append_div("dismissible-binding-host");
    let outside = append_div("dismissible-binding-outside");
    let dismissals: Arc<Mutex<Vec<DismissibleReason>>> = Arc::new(Mutex::new(Vec::new()));
    let dismissals_handle = Arc::clone(&dismissals);

    let mount = mount_to(host.clone(), move || {
        let dismissals = Arc::clone(&dismissals_handle);
        let layer = use_dismissible_layer::<html::Section>(DismissibleLayerOptions {
            on_dismiss: Some(Callback::new(move |reason| {
                dismissals.lock().expect("dismissals lock").push(reason);
            })),
            ..Default::default()
        });

        view! {
            <section id="dismissible-binding-root" node_ref=layer.node_ref()>
                <button id="dismissible-binding-inside">"Inside"</button>
            </section>
        }
    });

    render_tick().await;
    let root = host.first_element_child().expect("binding root");
    assert_eq!(host.child_element_count(), 1);
    assert_eq!(root.tag_name(), "SECTION");
    assert_eq!(root.id(), "dismissible-binding-root");

    let inside = html_element_by_id("dismissible-binding-inside");
    dispatch_pointer_down(&inside);
    render_tick().await;
    assert!(dismissals.lock().expect("dismissals lock").is_empty());

    dispatch_pointer_down(&outside);
    render_tick().await;
    assert_eq!(
        dismissals.lock().expect("dismissals lock").as_slice(),
        &[DismissibleReason::PointerDownOutside]
    );

    let inside = html_element_by_id("dismissible-binding-inside");
    dispatch_escape_keydown(&inside);
    render_tick().await;
    assert_eq!(
        dismissals.lock().expect("dismissals lock").as_slice(),
        &[
            DismissibleReason::PointerDownOutside,
            DismissibleReason::Escape
        ]
    );

    drop(mount);
    render_tick().await;
    remove_from_body(&host);
    remove_from_body(&outside);
}

#[wasm_bindgen_test]
async fn dismissible_layer_cancellable_events_can_prevent_dismissal() {
    let host = append_div("dismissible-cancel-host");
    let outside = append_button("dismissible-cancel-outside");
    let dismissals: Arc<Mutex<Vec<DismissibleReason>>> = Arc::new(Mutex::new(Vec::new()));
    let dismissals_handle = Arc::clone(&dismissals);
    let handled: Arc<Mutex<Vec<&'static str>>> = Arc::new(Mutex::new(Vec::new()));
    let pointer_handled = Arc::clone(&handled);
    let focus_handled = Arc::clone(&handled);
    let escape_handled = Arc::clone(&handled);

    let mount = mount_to(host.clone(), move || {
        let dismissals = Arc::clone(&dismissals_handle);
        let pointer_handled = Arc::clone(&pointer_handled);
        let focus_handled = Arc::clone(&focus_handled);
        let escape_handled = Arc::clone(&escape_handled);

        view! {
            <DismissibleLayer
                on_dismiss=Callback::new(move |reason| {
                    dismissals.lock().expect("dismissals lock").push(reason);
                })
                on_pointer_down_outside=Callback::new(move |event: DismissiblePointerDownOutsideEvent| {
                    event.prevent_default();
                    pointer_handled
                        .lock()
                        .expect("pointer handled lock")
                        .push("pointer");
                })
                on_focus_outside=Callback::new(move |event: DismissibleFocusOutsideEvent| {
                    event.prevent_default();
                    focus_handled
                        .lock()
                        .expect("focus handled lock")
                        .push("focus");
                })
                on_escape_key_down=Callback::new(move |event: DismissibleEscapeKeyDownEvent| {
                    event.prevent_default();
                    escape_handled
                        .lock()
                        .expect("escape handled lock")
                        .push("escape");
                })
            >
                <button id="dismissible-cancel-inside">"Inside"</button>
            </DismissibleLayer>
        }
    });

    let inside = html_element_by_id("dismissible-cancel-inside");
    focus_element(&inside);
    render_tick().await;

    let pointer = dispatch_pointer_down(&outside);
    render_tick().await;
    focus_element(&outside);
    render_tick().await;
    let escape = dispatch_escape_keydown(&inside);
    render_tick().await;

    assert!(pointer.default_prevented());
    assert!(escape.default_prevented());
    assert!(dismissals.lock().expect("dismissals lock").is_empty());
    assert_eq!(
        handled.lock().expect("handled lock").as_slice(),
        &["pointer", "focus", "escape"]
    );

    drop(mount);
    render_tick().await;
    remove_from_body(&host);
    remove_from_body(&outside);
}

#[wasm_bindgen_test]
async fn dismissible_layer_branches_are_treated_as_inside_the_layer() {
    let host = append_div("dismissible-branch-host");
    let branch = append_button("dismissible-branch-target");
    let outside = append_button("dismissible-branch-outside");
    let branch_element: web_sys::Element = branch.clone().into();
    let dismissals: Arc<Mutex<Vec<DismissibleReason>>> = Arc::new(Mutex::new(Vec::new()));
    let dismissals_handle = Arc::clone(&dismissals);

    let mount = mount_to(host.clone(), move || {
        let dismissals = Arc::clone(&dismissals_handle);
        let branch_element = branch_element.clone();

        view! {
            <DismissibleLayer
                branches=vec![branch_element]
                on_dismiss=Callback::new(move |reason| {
                    dismissals.lock().expect("dismissals lock").push(reason);
                })
            >
                <button id="dismissible-branch-inside">"Inside"</button>
            </DismissibleLayer>
        }
    });

    let inside = html_element_by_id("dismissible-branch-inside");
    focus_element(&inside);
    render_tick().await;

    dispatch_pointer_down(&branch);
    render_tick().await;
    focus_element(&branch);
    render_tick().await;
    assert!(dismissals.lock().expect("dismissals lock").is_empty());

    dispatch_escape_keydown(&branch);
    render_tick().await;
    assert_eq!(
        dismissals.lock().expect("dismissals lock").as_slice(),
        &[DismissibleReason::Escape]
    );

    dispatch_pointer_down(&outside);
    render_tick().await;
    assert_eq!(
        dismissals.lock().expect("dismissals lock").as_slice(),
        &[
            DismissibleReason::Escape,
            DismissibleReason::PointerDownOutside
        ]
    );

    drop(mount);
    render_tick().await;
    remove_from_body(&host);
    remove_from_body(&branch);
    remove_from_body(&outside);
}

#[wasm_bindgen_test]
async fn dismissible_layer_ignores_inside_pointerdown_and_reports_outside_pointerdown() {
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
    render_tick().await;
    assert!(dismissals.lock().expect("dismissals lock").is_empty());

    dispatch_pointer_down(&outside);
    render_tick().await;
    assert_eq!(
        dismissals.lock().expect("dismissals lock").as_slice(),
        &[DismissibleReason::PointerDownOutside]
    );

    drop(mount);
    render_tick().await;
    remove_from_body(&host);
    remove_from_body(&outside);
}

#[wasm_bindgen_test]
async fn dismissible_layer_reports_escape_to_callback_and_dismiss_reason() {
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
                on_escape_key_down=Callback::new(move |event: DismissibleEscapeKeyDownEvent| {
                    escape_keys
                        .lock()
                        .expect("escape keys lock")
                        .push(event.event().key());
                })
            >
                <button id="dismissible-escape-inside">"Inside"</button>
            </DismissibleLayer>
        }
    });

    let inside = html_element_by_id("dismissible-escape-inside");
    let escape = dispatch_escape_keydown(&inside);
    render_tick().await;

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
    render_tick().await;
    remove_from_body(&host);
}

#[wasm_bindgen_test]
async fn dismissible_layer_handles_escape_and_pointer_outside_in_live_dom() {
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
    render_tick().await;
    dispatch_pointer_down(&outside);
    render_tick().await;

    assert_eq!(
        dismissals.lock().expect("dismissals lock").as_slice(),
        &[
            DismissibleReason::Escape,
            DismissibleReason::PointerDownOutside
        ]
    );

    drop(mount);
    render_tick().await;
    remove_from_body(&host);
    remove_from_body(&outside);
}

#[wasm_bindgen_test]
async fn dismissible_layer_ignores_focus_moves_within_the_layer() {
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
                on_focus_outside=Callback::new(move |event: DismissibleFocusOutsideEvent| {
                    let target_id = event
                        .event().target()
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

    focus_element(&first);
    render_tick().await;
    focus_element(&second);
    render_tick().await;

    assert!(dismissals.lock().expect("dismissals lock").is_empty());
    assert!(focus_targets.lock().expect("focus targets lock").is_empty());

    drop(mount);
    render_tick().await;
    remove_from_body(&host);
}

#[wasm_bindgen_test]
async fn dismissible_layer_reports_focus_outside_via_callback_and_reason() {
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
                on_focus_outside=Callback::new(move |event: DismissibleFocusOutsideEvent| {
                    let target_id = event
                        .event().target()
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

    focus_element(&inside);
    render_tick().await;
    assert!(dismissals.lock().expect("dismissals lock").is_empty());
    assert!(focus_targets.lock().expect("focus targets lock").is_empty());

    focus_element(&outside);
    render_tick().await;

    assert_eq!(
        dismissals.lock().expect("dismissals lock").as_slice(),
        &[DismissibleReason::FocusOutside]
    );
    assert_eq!(
        focus_targets.lock().expect("focus targets lock").as_slice(),
        &["dismissible-focus-outside-target".to_string()]
    );

    drop(mount);
    render_tick().await;
    remove_from_body(&host);
    remove_from_body(&outside);
}

#[wasm_bindgen_test]
async fn dismissible_layer_suppresses_pointer_outside_when_disabled() {
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
                on_pointer_down_outside=Callback::new(move |event: DismissiblePointerDownOutsideEvent| {
                    let target_id = event
                        .event().target()
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
    render_tick().await;

    assert!(dismissals.lock().expect("dismissals lock").is_empty());
    assert!(
        pointer_targets
            .lock()
            .expect("pointer targets lock")
            .is_empty()
    );

    drop(mount);
    render_tick().await;
    remove_from_body(&host);
    remove_from_body(&outside);
}

#[wasm_bindgen_test]
async fn dismissible_layer_keeps_escape_and_focus_outside_active_when_pointer_dismiss_is_disabled()
{
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
                on_focus_outside=Callback::new(move |event: DismissibleFocusOutsideEvent| {
                    let target_id = event
                        .event().target()
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

    focus_element(&inside);
    render_tick().await;
    dispatch_escape_keydown(&inside);
    render_tick().await;
    dispatch_pointer_down(&outside);
    render_tick().await;
    focus_element(&outside);
    render_tick().await;

    assert_eq!(
        dismissals.lock().expect("dismissals lock").as_slice(),
        &[DismissibleReason::Escape, DismissibleReason::FocusOutside]
    );
    assert_eq!(
        focus_targets.lock().expect("focus targets lock").as_slice(),
        &["dismissible-pointer-disabled-focus-target".to_string()]
    );

    drop(mount);
    render_tick().await;
    remove_from_body(&host);
    remove_from_body(&outside);
}

#[wasm_bindgen_test]
async fn dismissible_layer_routes_pointer_outside_only_to_matching_callbacks() {
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
                on_pointer_down_outside=Callback::new(move |event: DismissiblePointerDownOutsideEvent| {
                    let target_id = event
                        .event().target()
                        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                        .map(|element| element.id())
                        .unwrap_or_default();
                    pointer_targets
                        .lock()
                        .expect("pointer targets lock")
                        .push(target_id);
                })
                on_focus_outside=Callback::new(move |event: DismissibleFocusOutsideEvent| {
                    let target_id = event
                        .event().target()
                        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                        .map(|element| element.id())
                        .unwrap_or_default();
                    focus_targets
                        .lock()
                        .expect("focus targets lock")
                        .push(target_id);
                })
                on_escape_key_down=Callback::new(move |event: DismissibleEscapeKeyDownEvent| {
                    escape_keys
                        .lock()
                        .expect("escape keys lock")
                        .push(event.event().key());
                })
            >
                <button id="dismissible-callback-pointer-inside">"Inside"</button>
            </DismissibleLayer>
        }
    });

    dispatch_pointer_down(&outside);
    render_tick().await;

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
    render_tick().await;
    remove_from_body(&host);
    remove_from_body(&outside);
}

#[wasm_bindgen_test]
async fn dismissible_layer_routes_focus_and_escape_only_to_matching_callbacks() {
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
                on_pointer_down_outside=Callback::new(move |event: DismissiblePointerDownOutsideEvent| {
                    let target_id = event
                        .event().target()
                        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                        .map(|element| element.id())
                        .unwrap_or_default();
                    pointer_targets
                        .lock()
                        .expect("pointer targets lock")
                        .push(target_id);
                })
                on_focus_outside=Callback::new(move |event: DismissibleFocusOutsideEvent| {
                    let target_id = event
                        .event().target()
                        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                        .map(|element| element.id())
                        .unwrap_or_default();
                    focus_targets
                        .lock()
                        .expect("focus targets lock")
                        .push(target_id);
                })
                on_escape_key_down=Callback::new(move |event: DismissibleEscapeKeyDownEvent| {
                    escape_keys
                        .lock()
                        .expect("escape keys lock")
                        .push(event.event().key());
                })
            >
                <button id="dismissible-callback-focus-inside">"Inside"</button>
            </DismissibleLayer>
        }
    });

    let inside = html_element_by_id("dismissible-callback-focus-inside");

    focus_element(&inside);
    render_tick().await;
    focus_element(&outside);
    render_tick().await;
    focus_element(&inside);
    render_tick().await;
    dispatch_escape_keydown(&inside);
    render_tick().await;

    assert_eq!(
        dismissals.lock().expect("dismissals lock").as_slice(),
        &[DismissibleReason::FocusOutside, DismissibleReason::Escape]
    );
    assert!(
        pointer_targets
            .lock()
            .expect("pointer targets lock")
            .is_empty()
    );
    assert_eq!(
        focus_targets.lock().expect("focus targets lock").as_slice(),
        &["dismissible-callback-focus-outside".to_string()]
    );
    assert_eq!(
        escape_keys.lock().expect("escape keys lock").as_slice(),
        &["Escape".to_string()]
    );

    drop(mount);
    render_tick().await;
    remove_from_body(&host);
    remove_from_body(&outside);
}

#[wasm_bindgen_test]
async fn nested_dismissible_layers_route_pointer_and_focus_outside_to_the_topmost_layer() {
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
                <DismissibleLayer on_dismiss=inner_on_dismiss>
                    <button id="dismissible-stack-inner">"Inner"</button>
                </DismissibleLayer>
            </DismissibleLayer>
        }
    });

    let inner = html_element_by_id("dismissible-stack-inner");
    let outer_only = html_element_by_id("dismissible-stack-outer-only");

    dispatch_pointer_down(&outer_only);
    render_tick().await;

    assert!(
        outer_dismissals
            .lock()
            .expect("outer dismissals lock")
            .is_empty()
    );
    assert_eq!(
        inner_dismissals
            .lock()
            .expect("inner dismissals lock")
            .as_slice(),
        &[DismissibleReason::PointerDownOutside]
    );

    focus_element(&inner);
    render_tick().await;
    focus_element(&outer_only);
    render_tick().await;

    assert!(
        outer_dismissals
            .lock()
            .expect("outer dismissals lock")
            .is_empty()
    );
    assert_eq!(
        inner_dismissals
            .lock()
            .expect("inner dismissals lock")
            .as_slice(),
        &[
            DismissibleReason::PointerDownOutside,
            DismissibleReason::FocusOutside
        ]
    );

    drop(mount);
    render_tick().await;
    remove_from_body(&host);
}

#[wasm_bindgen_test]
async fn nested_dismissible_layers_route_escape_to_the_topmost_layer() {
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
                <DismissibleLayer on_dismiss=inner_on_dismiss>
                    <button id="dismissible-stack-escape-inner">"Inner"</button>
                </DismissibleLayer>
            </DismissibleLayer>
        }
    });

    let inner = html_element_by_id("dismissible-stack-escape-inner");
    dispatch_escape_keydown(&inner);
    render_tick().await;

    assert!(
        outer_dismissals
            .lock()
            .expect("outer dismissals lock")
            .is_empty()
    );
    assert_eq!(
        inner_dismissals
            .lock()
            .expect("inner dismissals lock")
            .as_slice(),
        &[DismissibleReason::Escape]
    );

    drop(mount);
    render_tick().await;
    remove_from_body(&host);
}

#[wasm_bindgen_test]
async fn nested_dismissible_layers_restore_outer_pointer_and_focus_after_inner_unmount() {
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
                        let on_dismiss = inner_on_dismiss;
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
    render_tick().await;

    assert!(
        outer_dismissals
            .lock()
            .expect("outer dismissals lock")
            .is_empty()
    );
    assert_eq!(
        inner_dismissals
            .lock()
            .expect("inner dismissals lock")
            .as_slice(),
        &[DismissibleReason::PointerDownOutside]
    );
    assert!(
        document()
            .get_element_by_id("dismissible-stack-restore-inner")
            .is_none()
    );

    dispatch_pointer_down(&outside);
    render_tick().await;
    assert_eq!(
        outer_dismissals
            .lock()
            .expect("outer dismissals lock")
            .as_slice(),
        &[DismissibleReason::PointerDownOutside]
    );

    focus_element(&outer);
    render_tick().await;
    focus_element(&outside);
    render_tick().await;
    assert_eq!(
        outer_dismissals
            .lock()
            .expect("outer dismissals lock")
            .as_slice(),
        &[
            DismissibleReason::PointerDownOutside,
            DismissibleReason::FocusOutside
        ]
    );
    assert_eq!(
        inner_dismissals
            .lock()
            .expect("inner dismissals lock")
            .as_slice(),
        &[DismissibleReason::PointerDownOutside]
    );

    drop(mount);
    render_tick().await;
    remove_from_body(&host);
    remove_from_body(&outside);
}

#[wasm_bindgen_test]
async fn nested_dismissible_layers_restore_outer_escape_after_inner_unmount() {
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
                        let on_dismiss = inner_on_dismiss;
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
    render_tick().await;

    assert!(
        outer_dismissals
            .lock()
            .expect("outer dismissals lock")
            .is_empty()
    );
    assert_eq!(
        inner_dismissals
            .lock()
            .expect("inner dismissals lock")
            .as_slice(),
        &[DismissibleReason::Escape]
    );
    assert!(
        document()
            .get_element_by_id("dismissible-stack-restore-escape-inner")
            .is_none()
    );

    let outer = html_element_by_id("dismissible-stack-restore-escape-outer");
    focus_element(&outer);
    render_tick().await;
    dispatch_escape_keydown(&outer);
    render_tick().await;

    assert_eq!(
        outer_dismissals
            .lock()
            .expect("outer dismissals lock")
            .as_slice(),
        &[DismissibleReason::Escape]
    );
    assert_eq!(
        inner_dismissals
            .lock()
            .expect("inner dismissals lock")
            .as_slice(),
        &[DismissibleReason::Escape]
    );

    drop(mount);
    render_tick().await;
    remove_from_body(&host);
}

#[wasm_bindgen_test]
async fn stacked_dismissible_layers_suppress_pointer_outside_for_all_layers_when_topmost_pointer_dismiss_is_disabled()
 {
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
            Callback::new(move |event: DismissiblePointerDownOutsideEvent| {
                let target_id = event
                    .event()
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
            Callback::new(move |event: DismissiblePointerDownOutsideEvent| {
                let target_id = event
                    .event()
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
    render_tick().await;

    assert!(
        outer_dismissals
            .lock()
            .expect("outer dismissals lock")
            .is_empty()
    );
    assert!(
        inner_dismissals
            .lock()
            .expect("inner dismissals lock")
            .is_empty()
    );
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
    render_tick().await;
    remove_from_body(&host);
    remove_from_body(&outside);
}

#[wasm_bindgen_test]
async fn stacked_dismissible_layers_keep_focus_and_escape_owned_by_the_topmost_suppressed_layer() {
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
        let outer_on_focus_outside = Callback::new(move |event: DismissibleFocusOutsideEvent| {
            let target_id = event
                .event()
                .target()
                .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                .map(|element| element.id())
                .unwrap_or_default();
            outer_focus_targets
                .lock()
                .expect("outer focus targets lock")
                .push(target_id);
        });
        let inner_on_focus_outside = Callback::new(move |event: DismissibleFocusOutsideEvent| {
            let target_id = event
                .event()
                .target()
                .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                .map(|element| element.id())
                .unwrap_or_default();
            inner_focus_targets
                .lock()
                .expect("inner focus targets lock")
                .push(target_id);
        });
        let outer_on_escape_key_down =
            Callback::new(move |event: DismissibleEscapeKeyDownEvent| {
                outer_escape_keys
                    .lock()
                    .expect("outer escape keys lock")
                    .push(event.event().key());
            });
        let inner_on_escape_key_down =
            Callback::new(move |event: DismissibleEscapeKeyDownEvent| {
                inner_escape_keys
                    .lock()
                    .expect("inner escape keys lock")
                    .push(event.event().key());
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

    focus_element(&inner);
    render_tick().await;
    focus_element(&outside);
    render_tick().await;
    focus_element(&inner);
    render_tick().await;
    dispatch_escape_keydown(&inner);
    render_tick().await;

    assert!(
        outer_dismissals
            .lock()
            .expect("outer dismissals lock")
            .is_empty()
    );
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
        inner_dismissals
            .lock()
            .expect("inner dismissals lock")
            .as_slice(),
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
    render_tick().await;
    remove_from_body(&host);
    remove_from_body(&outside);
}

#[wasm_bindgen_test]
async fn stacked_suppressed_dismissible_layers_restore_outer_pointer_and_focus_after_inner_unmount()
{
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
        let outer_on_focus_outside = Callback::new(move |event: DismissibleFocusOutsideEvent| {
            let target_id = event
                .event()
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
                        let on_dismiss = inner_on_dismiss;
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
    focus_element(&inner);
    render_tick().await;
    focus_element(&outside);
    render_tick().await;

    assert!(
        outer_dismissals
            .lock()
            .expect("outer dismissals lock")
            .is_empty()
    );
    assert_eq!(
        inner_dismissals
            .lock()
            .expect("inner dismissals lock")
            .as_slice(),
        &[DismissibleReason::FocusOutside]
    );
    assert!(
        document()
            .get_element_by_id("dismissible-stack-suppressed-restore-inner")
            .is_none()
    );

    dispatch_pointer_down(&outside);
    render_tick().await;
    let outer = html_element_by_id("dismissible-stack-suppressed-restore-outer");
    focus_element(&outer);
    render_tick().await;
    focus_element(&outside);
    render_tick().await;

    assert_eq!(
        outer_dismissals
            .lock()
            .expect("outer dismissals lock")
            .as_slice(),
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
        inner_dismissals
            .lock()
            .expect("inner dismissals lock")
            .as_slice(),
        &[DismissibleReason::FocusOutside]
    );

    drop(mount);
    render_tick().await;
    remove_from_body(&host);
    remove_from_body(&outside);
}

#[wasm_bindgen_test]
async fn stacked_suppressed_dismissible_layers_restore_outer_escape_after_inner_unmount() {
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
                        let on_dismiss = inner_on_dismiss;
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
    render_tick().await;

    assert!(
        outer_dismissals
            .lock()
            .expect("outer dismissals lock")
            .is_empty()
    );
    assert_eq!(
        inner_dismissals
            .lock()
            .expect("inner dismissals lock")
            .as_slice(),
        &[DismissibleReason::Escape]
    );
    assert!(
        document()
            .get_element_by_id("dismissible-stack-suppressed-restore-escape-inner")
            .is_none()
    );

    let outer = html_element_by_id("dismissible-stack-suppressed-restore-escape-outer");
    focus_element(&outer);
    render_tick().await;
    dispatch_escape_keydown(&outer);
    render_tick().await;

    assert_eq!(
        outer_dismissals
            .lock()
            .expect("outer dismissals lock")
            .as_slice(),
        &[DismissibleReason::Escape]
    );
    assert_eq!(
        inner_dismissals
            .lock()
            .expect("inner dismissals lock")
            .as_slice(),
        &[DismissibleReason::Escape]
    );

    drop(mount);
    render_tick().await;
    remove_from_body(&host);
}

#[wasm_bindgen_test]
async fn dismissible_stack_cleanup_handles_middle_sibling_removal_for_pointer_and_focus() {
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
        let outer_on_focus_outside = Callback::new(move |event: DismissibleFocusOutsideEvent| {
            let target_id = event
                .event()
                .target()
                .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                .map(|element| element.id())
                .unwrap_or_default();
            outer_focus_targets
                .lock()
                .expect("outer focus targets lock")
                .push(target_id);
        });
        let inner_on_focus_outside = Callback::new(move |event: DismissibleFocusOutsideEvent| {
            let target_id = event
                .event()
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
                        let on_dismiss = inner_on_dismiss;
                        let on_focus_outside = inner_on_focus_outside;
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
    render_tick().await;
    assert!(
        document()
            .get_element_by_id("dismissible-nonlifo-doc-middle")
            .is_none()
    );

    let inner = html_element_by_id("dismissible-nonlifo-doc-inner");
    let outer = html_element_by_id("dismissible-nonlifo-doc-outer");

    focus_element(&inner);
    render_tick().await;
    focus_element(&outer);
    render_tick().await;
    dispatch_pointer_down(&outside);
    render_tick().await;

    assert!(
        outer_dismissals
            .lock()
            .expect("outer dismissals lock")
            .is_empty()
    );
    assert!(
        outer_focus_targets
            .lock()
            .expect("outer focus targets lock")
            .is_empty()
    );
    assert_eq!(
        inner_dismissals
            .lock()
            .expect("inner dismissals lock")
            .as_slice(),
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
    render_tick().await;
    assert!(
        document()
            .get_element_by_id("dismissible-nonlifo-doc-inner")
            .is_none()
    );

    dispatch_pointer_down(&outside);
    render_tick().await;
    focus_element(&outer);
    render_tick().await;
    focus_element(&outside);
    render_tick().await;

    assert_eq!(
        outer_dismissals
            .lock()
            .expect("outer dismissals lock")
            .as_slice(),
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
        inner_dismissals
            .lock()
            .expect("inner dismissals lock")
            .as_slice(),
        &[
            DismissibleReason::FocusOutside,
            DismissibleReason::PointerDownOutside
        ]
    );

    drop(mount);
    render_tick().await;
    remove_from_body(&host);
    remove_from_body(&outside);
}

#[wasm_bindgen_test]
async fn dismissible_stack_cleanup_handles_middle_sibling_removal_for_escape() {
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
                        let on_dismiss = inner_on_dismiss;
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
    render_tick().await;
    assert!(
        document()
            .get_element_by_id("dismissible-nonlifo-escape-middle")
            .is_none()
    );

    let inner = html_element_by_id("dismissible-nonlifo-escape-inner");
    dispatch_escape_keydown(&inner);
    render_tick().await;

    assert!(
        outer_dismissals
            .lock()
            .expect("outer dismissals lock")
            .is_empty()
    );
    assert_eq!(
        inner_dismissals
            .lock()
            .expect("inner dismissals lock")
            .as_slice(),
        &[DismissibleReason::Escape]
    );

    inner_present.set(false);
    render_tick().await;
    assert!(
        document()
            .get_element_by_id("dismissible-nonlifo-escape-inner")
            .is_none()
    );

    let outer = html_element_by_id("dismissible-nonlifo-escape-outer");
    focus_element(&outer);
    render_tick().await;
    dispatch_escape_keydown(&outer);
    render_tick().await;

    assert_eq!(
        outer_dismissals
            .lock()
            .expect("outer dismissals lock")
            .as_slice(),
        &[DismissibleReason::Escape]
    );
    assert_eq!(
        inner_dismissals
            .lock()
            .expect("inner dismissals lock")
            .as_slice(),
        &[DismissibleReason::Escape]
    );

    drop(mount);
    render_tick().await;
    remove_from_body(&host);
}

#[wasm_bindgen_test]
async fn dismissible_cleanup_reuse_cycles_restore_outer_pointer_and_focus_each_time() {
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
        let outer_on_focus_outside = Callback::new(move |event: DismissibleFocusOutsideEvent| {
            let target_id = event
                .event()
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
                        let on_dismiss = inner_on_dismiss;
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
            render_tick().await;
        }

        let outer = html_element_by_id("dismissible-reuse-doc-outer");
        dispatch_pointer_down(&outer);
        render_tick().await;

        assert_eq!(
            inner_dismissals
                .lock()
                .expect("inner dismissals lock")
                .len(),
            cycle + 1
        );
        assert_eq!(
            outer_dismissals
                .lock()
                .expect("outer dismissals lock")
                .len(),
            cycle * 2
        );
        assert!(
            document()
                .get_element_by_id("dismissible-reuse-doc-inner")
                .is_none()
        );

        dispatch_pointer_down(&outside);
        render_tick().await;
        focus_element(&outer);
        render_tick().await;
        focus_element(&outside);
        render_tick().await;
    }

    assert_eq!(
        inner_dismissals
            .lock()
            .expect("inner dismissals lock")
            .as_slice(),
        &[
            DismissibleReason::PointerDownOutside,
            DismissibleReason::PointerDownOutside
        ]
    );
    assert_eq!(
        outer_dismissals
            .lock()
            .expect("outer dismissals lock")
            .as_slice(),
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
    render_tick().await;
    remove_from_body(&host);
    remove_from_body(&outside);
}

#[wasm_bindgen_test]
async fn dismissible_cleanup_reuse_cycles_restore_outer_escape_each_time() {
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
                        let on_dismiss = inner_on_dismiss;
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
            render_tick().await;
        }

        let inner = html_element_by_id("dismissible-reuse-escape-inner");
        dispatch_escape_keydown(&inner);
        render_tick().await;

        assert_eq!(
            inner_dismissals
                .lock()
                .expect("inner dismissals lock")
                .len(),
            cycle + 1
        );
        assert_eq!(
            outer_dismissals
                .lock()
                .expect("outer dismissals lock")
                .len(),
            cycle
        );
        assert!(
            document()
                .get_element_by_id("dismissible-reuse-escape-inner")
                .is_none()
        );

        let outer = html_element_by_id("dismissible-reuse-escape-outer");
        focus_element(&outer);
        render_tick().await;
        dispatch_escape_keydown(&outer);
        render_tick().await;
    }

    assert_eq!(
        inner_dismissals
            .lock()
            .expect("inner dismissals lock")
            .as_slice(),
        &[DismissibleReason::Escape, DismissibleReason::Escape]
    );
    assert_eq!(
        outer_dismissals
            .lock()
            .expect("outer dismissals lock")
            .as_slice(),
        &[DismissibleReason::Escape, DismissibleReason::Escape]
    );

    drop(mount);
    render_tick().await;
    remove_from_body(&host);
}

#[wasm_bindgen_test]
async fn dismissible_layers_emit_no_pointer_or_focus_callbacks_after_full_teardown() {
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
            Callback::new(move |event: DismissiblePointerDownOutsideEvent| {
                let target_id = event
                    .event()
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
            Callback::new(move |event: DismissiblePointerDownOutsideEvent| {
                let target_id = event
                    .event()
                    .target()
                    .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                    .map(|element| element.id())
                    .unwrap_or_default();
                inner_pointer_targets
                    .lock()
                    .expect("inner pointer targets lock")
                    .push(target_id);
            });
        let outer_on_focus_outside = Callback::new(move |event: DismissibleFocusOutsideEvent| {
            let target_id = event
                .event()
                .target()
                .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                .map(|element| element.id())
                .unwrap_or_default();
            outer_focus_targets
                .lock()
                .expect("outer focus targets lock")
                .push(target_id);
        });
        let inner_on_focus_outside = Callback::new(move |event: DismissibleFocusOutsideEvent| {
            let target_id = event
                .event()
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
    render_tick().await;

    assert!(host.first_element_child().is_none());
    assert!(
        document()
            .get_element_by_id("dismissible-full-teardown-doc-inner")
            .is_none()
    );

    dispatch_pointer_down(&outside);
    render_tick().await;
    focus_element(&outside);
    render_tick().await;

    assert!(
        outer_dismissals
            .lock()
            .expect("outer dismissals lock")
            .is_empty()
    );
    assert!(
        inner_dismissals
            .lock()
            .expect("inner dismissals lock")
            .is_empty()
    );
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
    render_tick().await;
    remove_from_body(&host);
    remove_from_body(&outside);
}

#[wasm_bindgen_test]
async fn dismissible_layers_emit_no_escape_callbacks_after_full_teardown() {
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
        let outer_on_escape_key_down =
            Callback::new(move |event: DismissibleEscapeKeyDownEvent| {
                outer_escape_keys
                    .lock()
                    .expect("outer escape keys lock")
                    .push(event.event().key());
            });
        let inner_on_escape_key_down =
            Callback::new(move |event: DismissibleEscapeKeyDownEvent| {
                inner_escape_keys
                    .lock()
                    .expect("inner escape keys lock")
                    .push(event.event().key());
            });

        view! {
            {move || {
                outer_present.get().then(|| {
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
    render_tick().await;

    assert!(host.first_element_child().is_none());
    assert!(
        document()
            .get_element_by_id("dismissible-full-teardown-escape-inner")
            .is_none()
    );

    dispatch_escape_keydown(&outside);
    render_tick().await;

    assert!(
        outer_dismissals
            .lock()
            .expect("outer dismissals lock")
            .is_empty()
    );
    assert!(
        inner_dismissals
            .lock()
            .expect("inner dismissals lock")
            .is_empty()
    );
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
    render_tick().await;
    remove_from_body(&host);
    remove_from_body(&outside);
}

#[wasm_bindgen_test]
async fn dismissible_layers_handle_pointer_and_focus_once_after_full_teardown_and_remount() {
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
        let on_pointer_down_outside =
            Callback::new(move |event: DismissiblePointerDownOutsideEvent| {
                let target_id = event
                    .event()
                    .target()
                    .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                    .map(|element| element.id())
                    .unwrap_or_default();
                pointer_targets
                    .lock()
                    .expect("pointer targets lock")
                    .push(target_id);
            });
        let on_focus_outside = Callback::new(move |event: DismissibleFocusOutsideEvent| {
            let target_id = event
                .event()
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
            render_tick().await;
        }

        let inner = html_element_by_id("dismissible-remount-doc-inner");
        focus_element(&inner);
        render_tick().await;
        dispatch_pointer_down(&outside);
        render_tick().await;
        focus_element(&outside);
        render_tick().await;

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
        render_tick().await;
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
    render_tick().await;
    remove_from_body(&host);
    remove_from_body(&outside);
}

#[wasm_bindgen_test]
async fn dismissible_layers_handle_escape_once_after_full_teardown_and_remount() {
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
        let on_escape_key_down = Callback::new(move |event: DismissibleEscapeKeyDownEvent| {
            escape_keys
                .lock()
                .expect("escape keys lock")
                .push(event.event().key());
        });

        view! {
            {move || {
                present.get().then(|| {
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
            render_tick().await;
        }

        let inner = html_element_by_id("dismissible-remount-escape-inner");
        dispatch_escape_keydown(&inner);
        render_tick().await;

        assert_eq!(dismissals.lock().expect("dismissals lock").len(), cycle + 1);
        assert_eq!(
            escape_keys.lock().expect("escape keys lock").len(),
            cycle + 1
        );

        present.set(false);
        render_tick().await;
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
    render_tick().await;
    remove_from_body(&host);
}

#[wasm_bindgen_test]
async fn portal_mounts_children_into_the_explicit_target() {
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
    render_tick().await;
    assert_eq!(target.text_content().unwrap_or_default(), "");

    remove_from_body(&host);
    remove_from_body(&target);
}

#[wasm_bindgen_test]
async fn portal_without_explicit_mount_appends_to_body_and_cleans_up_on_drop() {
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
    render_tick().await;
    assert_eq!(host.text_content().unwrap_or_default(), "");
    assert_eq!(body().child_element_count(), body_children_before_mount);

    remove_from_body(&host);
}

#[wasm_bindgen_test]
async fn portal_without_explicit_mount_repeated_remounts_leave_no_stranded_body_nodes() {
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
    render_tick().await;
    assert_eq!(body().child_element_count(), body_children_before_mount);

    label.set("Second");
    render_tick().await;
    present.set(true);
    render_tick().await;
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
    render_tick().await;
    assert_eq!(body().child_element_count(), body_children_before_mount);

    drop(mount);
    render_tick().await;
    assert_eq!(body().child_element_count(), body_children_before_mount);

    remove_from_body(&host);
}

#[wasm_bindgen_test]
async fn portal_repeated_remounts_clear_prior_targets_before_rendering_into_new_ones() {
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
    render_tick().await;
    assert_eq!(first_target.text_content().unwrap_or_default(), "");
    assert_eq!(first_target.child_element_count(), 0);
    assert_eq!(second_target.text_content().unwrap_or_default(), "");
    assert_eq!(second_target.child_element_count(), 0);

    use_first_target.set(false);
    render_tick().await;
    present.set(true);
    render_tick().await;
    assert_eq!(first_target.text_content().unwrap_or_default(), "");
    assert_eq!(first_target.child_element_count(), 0);
    assert_eq!(second_target.text_content().as_deref(), Some("Portaled"));
    assert_eq!(second_target.child_element_count(), 1);

    present.set(false);
    render_tick().await;
    assert_eq!(first_target.text_content().unwrap_or_default(), "");
    assert_eq!(first_target.child_element_count(), 0);
    assert_eq!(second_target.text_content().unwrap_or_default(), "");
    assert_eq!(second_target.child_element_count(), 0);

    use_first_target.set(true);
    render_tick().await;
    present.set(true);
    render_tick().await;
    assert_eq!(first_target.text_content().as_deref(), Some("Portaled"));
    assert_eq!(first_target.child_element_count(), 1);
    assert_eq!(second_target.text_content().unwrap_or_default(), "");
    assert_eq!(second_target.child_element_count(), 0);

    drop(mount);
    render_tick().await;
    assert_eq!(first_target.text_content().unwrap_or_default(), "");
    assert_eq!(first_target.child_element_count(), 0);
    assert_eq!(second_target.text_content().unwrap_or_default(), "");
    assert_eq!(second_target.child_element_count(), 0);

    remove_from_body(&host);
    remove_from_body(&first_target);
    remove_from_body(&second_target);
}

#[wasm_bindgen_test]
async fn portal_repeated_remounts_into_the_same_target_do_not_duplicate_children() {
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
    render_tick().await;
    assert_eq!(target.text_content().unwrap_or_default(), "");
    assert_eq!(target.child_element_count(), 0);

    label.set("Second");
    render_tick().await;
    present.set(true);
    render_tick().await;
    assert_eq!(target.text_content().as_deref(), Some("Second"));
    assert_eq!(target.child_element_count(), 1);

    present.set(false);
    render_tick().await;
    assert_eq!(target.text_content().unwrap_or_default(), "");
    assert_eq!(target.child_element_count(), 0);

    label.set("Third");
    render_tick().await;
    present.set(true);
    render_tick().await;
    assert_eq!(target.text_content().as_deref(), Some("Third"));
    assert_eq!(target.child_element_count(), 1);

    drop(mount);
    render_tick().await;
    assert_eq!(target.text_content().unwrap_or_default(), "");
    assert_eq!(target.child_element_count(), 0);

    remove_from_body(&host);
    remove_from_body(&target);
}

#[wasm_bindgen_test]
async fn portal_retargets_live_between_explicit_targets_without_teardown() {
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
    render_tick().await;
    assert_eq!(host.text_content().unwrap_or_default(), "");
    assert_eq!(first_target.text_content().unwrap_or_default(), "");
    assert_eq!(first_target.child_element_count(), 0);
    assert_eq!(second_target.text_content().as_deref(), Some("Retargeted"));
    assert_eq!(second_target.child_element_count(), 1);

    use_first_target.set(true);
    render_tick().await;
    assert_eq!(host.text_content().unwrap_or_default(), "");
    assert_eq!(first_target.text_content().as_deref(), Some("Retargeted"));
    assert_eq!(first_target.child_element_count(), 1);
    assert_eq!(second_target.text_content().unwrap_or_default(), "");
    assert_eq!(second_target.child_element_count(), 0);

    drop(mount);
    render_tick().await;
    assert_eq!(first_target.text_content().unwrap_or_default(), "");
    assert_eq!(first_target.child_element_count(), 0);
    assert_eq!(second_target.text_content().unwrap_or_default(), "");
    assert_eq!(second_target.child_element_count(), 0);

    remove_from_body(&host);
    remove_from_body(&first_target);
    remove_from_body(&second_target);
}

#[wasm_bindgen_test]
async fn portal_live_retargeting_keeps_only_one_copy_while_content_changes() {
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
    render_tick().await;
    use_first_target.set(false);
    render_tick().await;
    assert_eq!(first_target.text_content().unwrap_or_default(), "");
    assert_eq!(first_target.child_element_count(), 0);
    assert_eq!(second_target.text_content().as_deref(), Some("Two"));
    assert_eq!(second_target.child_element_count(), 1);

    label.set("Three");
    render_tick().await;
    use_first_target.set(true);
    render_tick().await;
    assert_eq!(first_target.text_content().as_deref(), Some("Three"));
    assert_eq!(first_target.child_element_count(), 1);
    assert_eq!(second_target.text_content().unwrap_or_default(), "");
    assert_eq!(second_target.child_element_count(), 0);

    drop(mount);
    render_tick().await;
    assert_eq!(first_target.text_content().unwrap_or_default(), "");
    assert_eq!(first_target.child_element_count(), 0);
    assert_eq!(second_target.text_content().unwrap_or_default(), "");
    assert_eq!(second_target.child_element_count(), 0);

    remove_from_body(&host);
    remove_from_body(&first_target);
    remove_from_body(&second_target);
}

#[wasm_bindgen_test]
async fn modal_hide_siblings_hides_nested_siblings_and_restores_previous_state() {
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
    assert_eq!(
        attr(&branch_sibling, "aria-hidden").as_deref(),
        Some("true")
    );
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
async fn nested_modal_guards_keep_outer_siblings_hidden_until_last_guard_drops() {
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
    assert_eq!(
        attr(&branch_sibling, "aria-hidden").as_deref(),
        Some("true")
    );
    assert!(attr(&inner_sibling, "aria-hidden").is_none());

    let inner_guard = modal_hide_siblings(&inner_root_element).expect("inner modal guard");
    assert_eq!(attr(&inner_sibling, "aria-hidden").as_deref(), Some("true"));
    assert!(inner_sibling.has_attribute("inert"));

    drop(inner_guard);

    assert_eq!(attr(&left, "aria-hidden").as_deref(), Some("true"));
    assert_eq!(attr(&right, "aria-hidden").as_deref(), Some("true"));
    assert_eq!(
        attr(&branch_sibling, "aria-hidden").as_deref(),
        Some("true")
    );
    assert!(attr(&inner_sibling, "aria-hidden").is_none());
    assert!(!inner_sibling.has_attribute("inert"));

    drop(outer_guard);

    assert!(attr(&left, "aria-hidden").is_none());
    assert!(attr(&right, "aria-hidden").is_none());
    assert!(attr(&branch_sibling, "aria-hidden").is_none());

    remove_from_body(&shell);
}

#[wasm_bindgen_test]
async fn focus_scope_binding_attaches_to_the_target_element_without_a_wrapper() {
    let host = append_div("focus-binding-host");

    let mount = mount_to(host.clone(), move || {
        let scope = use_focus_scope::<html::Section>(FocusScopeOptions {
            trapped: true,
            auto_focus: true,
            ..Default::default()
        });

        view! {
            <section id="focus-binding-root" node_ref=scope.node_ref() tabindex="-1">
                <button id="focus-binding-first">"First"</button>
                <button id="focus-binding-second">"Second"</button>
            </section>
        }
    });

    render_tick().await;
    let root = host.first_element_child().expect("focus binding root");
    assert_eq!(host.child_element_count(), 1);
    assert_eq!(root.tag_name(), "SECTION");
    assert_eq!(root.id(), "focus-binding-root");
    assert_eq!(active_id().as_deref(), Some("focus-binding-first"));

    let first = html_element_by_id("focus-binding-first");
    let second = html_element_by_id("focus-binding-second");

    let first_tab = dispatch_tab_keydown(&first, false);
    render_tick().await;
    assert!(first_tab.default_prevented());
    assert_eq!(active_id().as_deref(), Some("focus-binding-second"));

    let second_tab = dispatch_tab_keydown(&second, false);
    render_tick().await;
    assert!(second_tab.default_prevented());
    assert_eq!(active_id().as_deref(), Some("focus-binding-first"));

    drop(mount);
    render_tick().await;
    remove_from_body(&host);
}

#[wasm_bindgen_test]
async fn focus_scope_binding_auto_focuses_the_target_when_no_child_is_tabbable() {
    let host = append_div("focus-binding-empty-host");

    let mount = mount_to(host.clone(), move || {
        let scope = use_focus_scope::<html::Section>(FocusScopeOptions {
            auto_focus: true,
            ..Default::default()
        });

        view! {
            <section id="focus-binding-empty-root" node_ref=scope.node_ref() tabindex="-1">
                <span id="focus-binding-empty-child">"Only child"</span>
            </section>
        }
    });

    render_tick().await;

    let root = html_element_by_id("focus-binding-empty-root");
    let root_element: web_sys::Element = root.clone().into();
    assert!(active_element().is_some_and(|active| active.is_same_node(Some(&root_element))));

    drop(mount);
    render_tick().await;
    remove_from_body(&host);
}

#[wasm_bindgen_test]
async fn focus_scope_skips_disabled_hidden_inert_and_negative_tabindex_candidates() {
    let host = append_div("focus-filter-host");

    let mount = mount_to(host.clone(), move || {
        view! {
            <FocusScope trapped=true>
                <button id="focus-filter-first">"First"</button>
                <button id="focus-filter-disabled">"Disabled"</button>
                <button id="focus-filter-hidden">"Hidden"</button>
                <button id="focus-filter-inert">"Inert"</button>
                <button id="focus-filter-aria-hidden">"Aria hidden"</button>
                <button id="focus-filter-display-none">"Display none"</button>
                <button id="focus-filter-visibility-hidden">"Visibility hidden"</button>
                <button id="focus-filter-negative-tabindex" tabindex="-2">"Negative"</button>
                <button id="focus-filter-last">"Last"</button>
            </FocusScope>
        }
    });

    render_tick().await;
    html_element_by_id("focus-filter-disabled")
        .set_attribute("disabled", "")
        .expect("set disabled");
    html_element_by_id("focus-filter-hidden")
        .set_attribute("hidden", "")
        .expect("set hidden");
    html_element_by_id("focus-filter-inert")
        .set_attribute("inert", "")
        .expect("set inert");
    html_element_by_id("focus-filter-aria-hidden")
        .set_attribute("aria-hidden", "true")
        .expect("set aria hidden");
    html_element_by_id("focus-filter-display-none")
        .style()
        .set_property("display", "none")
        .expect("set display none");
    html_element_by_id("focus-filter-visibility-hidden")
        .style()
        .set_property("visibility", "hidden")
        .expect("set visibility hidden");

    let first = html_element_by_id("focus-filter-first");
    let last = html_element_by_id("focus-filter-last");

    focus_element(&first);
    render_tick().await;

    let first_tab = dispatch_tab_keydown(&first, false);
    render_tick().await;
    assert!(first_tab.default_prevented());
    assert_eq!(active_id().as_deref(), Some("focus-filter-last"));

    let shift_tab = dispatch_tab_keydown(&last, true);
    render_tick().await;
    assert!(shift_tab.default_prevented());
    assert_eq!(active_id().as_deref(), Some("focus-filter-first"));

    drop(mount);
    render_tick().await;
    remove_from_body(&host);
}

#[wasm_bindgen_test]
async fn nested_focus_scopes_trap_only_their_owned_candidates() {
    let host = append_div("focus-nested-host");

    let mount = mount_to(host.clone(), move || {
        view! {
            <FocusScope trapped=true>
                <button id="focus-nested-outer-first">"Outer first"</button>
                <FocusScope trapped=true>
                    <button id="focus-nested-inner-first">"Inner first"</button>
                    <button id="focus-nested-inner-second">"Inner second"</button>
                </FocusScope>
                <button id="focus-nested-outer-last">"Outer last"</button>
            </FocusScope>
        }
    });

    render_tick().await;

    let outer_first = html_element_by_id("focus-nested-outer-first");
    focus_element(&outer_first);
    render_tick().await;

    let outer_tab = dispatch_tab_keydown(&outer_first, false);
    render_tick().await;
    assert!(outer_tab.default_prevented());
    assert_eq!(active_id().as_deref(), Some("focus-nested-outer-last"));

    let inner_first = html_element_by_id("focus-nested-inner-first");
    let inner_second = html_element_by_id("focus-nested-inner-second");
    focus_element(&inner_first);
    render_tick().await;

    let inner_tab = dispatch_tab_keydown(&inner_first, false);
    render_tick().await;
    assert!(inner_tab.default_prevented());
    assert_eq!(active_id().as_deref(), Some("focus-nested-inner-second"));

    let inner_wrap_tab = dispatch_tab_keydown(&inner_second, false);
    render_tick().await;
    assert!(inner_wrap_tab.default_prevented());
    assert_eq!(active_id().as_deref(), Some("focus-nested-inner-first"));
    assert_ne!(active_id().as_deref(), Some("focus-nested-outer-last"));

    drop(mount);
    render_tick().await;
    remove_from_body(&host);
}

#[wasm_bindgen_test]
async fn focus_scope_traps_tab_within_the_live_scope() {
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

    focus_element(&first);
    render_tick().await;
    assert_eq!(active_id().as_deref(), Some("focus-first"));

    let first_tab = dispatch_tab_keydown(&first, false);
    render_tick().await;
    assert!(first_tab.default_prevented());
    assert_eq!(active_id().as_deref(), Some("focus-second"));

    let second_tab = dispatch_tab_keydown(&second, false);
    render_tick().await;
    assert!(second_tab.default_prevented());
    assert_eq!(active_id().as_deref(), Some("focus-first"));
    assert_ne!(active_id().as_deref(), Some("focus-after"));

    drop(mount);
    render_tick().await;
    remove_from_body(&host);
}

#[wasm_bindgen_test]
async fn focus_scope_wraps_shift_tab_to_the_last_focusable() {
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

    focus_element(&first);
    render_tick().await;
    assert_eq!(active_id().as_deref(), Some("focus-shift-first"));

    let shift_tab = dispatch_tab_keydown(&first, true);
    render_tick().await;
    assert!(shift_tab.default_prevented());
    assert_eq!(active_id().as_deref(), Some("focus-shift-last"));
    assert_ne!(active_id().as_deref(), Some("focus-shift-before"));

    drop(mount);
    render_tick().await;
    remove_from_body(&host);
}

#[wasm_bindgen_test]
async fn focus_scope_auto_focuses_restores_previous_focus_and_runs_callbacks() {
    let previous = document()
        .create_element("button")
        .expect("create previous button")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("previous html element");
    previous.set_id("focus-lifecycle-previous");
    body()
        .append_child(&previous)
        .expect("append previous button");
    focus_element(&previous);
    render_tick().await;
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
    render_tick().await;

    assert_eq!(active_id().as_deref(), Some("focus-lifecycle-first"));
    assert_eq!(
        callbacks.lock().expect("callbacks lock").as_slice(),
        &["mount"]
    );

    drop(mount);
    render_tick().await;

    assert_eq!(active_id().as_deref(), Some("focus-lifecycle-previous"));
    assert_eq!(
        callbacks.lock().expect("callbacks lock").as_slice(),
        &["mount", "unmount"]
    );

    remove_from_body(&host);
    remove_from_body(&previous);
}

#[wasm_bindgen_test]
async fn focus_scope_auto_focus_falls_back_to_the_wrapper_when_no_child_is_focusable() {
    let host = append_div("focus-wrapper-host");

    let mount = mount_to(host.clone(), move || {
        view! {
            <FocusScope auto_focus=true>
                <span id="focus-wrapper-child">"Only child"</span>
            </FocusScope>
        }
    });
    render_tick().await;

    let wrapper = host
        .first_element_child()
        .expect("focus scope wrapper")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("wrapper html element");
    let wrapper_element: web_sys::Element = wrapper.clone().into();

    assert_eq!(wrapper.tab_index(), -1);
    assert!(active_element().is_some_and(|active| { active.is_same_node(Some(&wrapper_element)) }));

    drop(mount);
    render_tick().await;
    remove_from_body(&host);
}

#[wasm_bindgen_test]
async fn presence_binding_attaches_to_the_target_element_without_a_wrapper() {
    let host = append_div("presence-binding-host");
    let present = RwSignal::new(true);

    let mount = mount_to(host.clone(), move || {
        let presence = use_presence::<html::Section>(Signal::derive(move || present.get()), None);

        view! {
            {move || -> AnyView {
                if !presence.is_rendered() {
                    ().into_any()
                } else {
                    let node_ref = presence.node_ref();
                    let data_state = presence.clone();
                    let transition_end = presence.transition_end_handler();
                    let transition_cancel = presence.transition_cancel_handler();
                    let animation_end = presence.animation_end_handler();
                    let animation_cancel = presence.animation_cancel_handler();

                    view! {
                        <section
                            id="presence-binding-root"
                            node_ref=node_ref
                            data-state=move || data_state.data_state()
                            on:transitionend=move |event| transition_end.run(event)
                            on:transitioncancel=move |event| transition_cancel.run(event)
                            on:animationend=move |event| animation_end.run(event)
                            on:animationcancel=move |event| animation_cancel.run(event)
                        >
                            "Present"
                        </section>
                    }
                    .into_any()
                }
            }}
        }
    });

    render_tick().await;
    let root = host.first_element_child().expect("presence binding root");
    assert_eq!(host.child_element_count(), 1);
    assert_eq!(root.tag_name(), "SECTION");
    assert_eq!(root.id(), "presence-binding-root");
    assert_eq!(
        attr(&html_element_by_id("presence-binding-root"), "data-state").as_deref(),
        Some("open")
    );

    present.set(false);
    TimeoutFuture::new(40).await;
    assert!(host.first_element_child().is_none());

    drop(mount);
    render_tick().await;
    remove_from_body(&host);
}

#[wasm_bindgen_test]
async fn presence_ignores_bubbled_child_transitionend_until_root_transitionend_completes_exit() {
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
        .set_property("transition-property", "opacity")
        .expect("set transition property");
    root.style()
        .set_property("transition-duration", "10s")
        .expect("set transition duration");

    present.set(false);
    render_tick().await;
    assert_eq!(attr(&root, "data-state").as_deref(), Some("closed"));
    assert!(host.first_element_child().is_some());

    let child = html_element_by_id("presence-transition-child");
    dispatch_transition_end(&child, true);
    render_tick().await;
    assert!(host.first_element_child().is_some());
    assert!(
        exit_callbacks
            .lock()
            .expect("exit callbacks lock")
            .is_empty()
    );

    dispatch_transition_end(&root, true);
    TimeoutFuture::new(40).await;
    assert!(host.first_element_child().is_none());
    assert_eq!(
        exit_callbacks
            .lock()
            .expect("exit callbacks lock")
            .as_slice(),
        &["transition"]
    );

    drop(mount);
    render_tick().await;
    remove_from_body(&host);
}

#[wasm_bindgen_test]
async fn presence_ignores_bubbled_child_animationend_until_root_animationend_completes_exit() {
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
        .set_property("animation-name", "fade")
        .expect("set animation name");
    root.style()
        .set_property("animation-duration", "10s")
        .expect("set animation duration");

    present.set(false);
    render_tick().await;
    assert_eq!(attr(&root, "data-state").as_deref(), Some("closed"));
    assert!(host.first_element_child().is_some());

    let child = html_element_by_id("presence-animation-child");
    dispatch_animation_end(&child, true);
    render_tick().await;
    assert!(host.first_element_child().is_some());
    assert!(
        exit_callbacks
            .lock()
            .expect("exit callbacks lock")
            .is_empty()
    );

    dispatch_animation_end(&root, true);
    TimeoutFuture::new(40).await;
    assert!(host.first_element_child().is_none());
    assert_eq!(
        exit_callbacks
            .lock()
            .expect("exit callbacks lock")
            .as_slice(),
        &["animation"]
    );

    drop(mount);
    render_tick().await;
    remove_from_body(&host);
}

#[wasm_bindgen_test]
async fn presence_waits_for_every_keyed_track_and_accepts_cancel_events() {
    let host = append_div("presence-multi-track-host");
    let present = RwSignal::new(true);
    let exits = Arc::new(Mutex::new(0_u32));
    let exits_handle = Arc::clone(&exits);
    let mount = mount_to(host.clone(), move || {
        let on_exit_complete = {
            let exits = Arc::clone(&exits_handle);
            Callback::new(move |_| *exits.lock().expect("exits lock") += 1)
        };
        view! {
            <Presence
                present=Signal::derive(move || present.get())
                on_exit_complete=on_exit_complete
            >
                "Tracks"
            </Presence>
        }
    });
    let root = host
        .first_element_child()
        .expect("presence root")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("presence root html element");
    root.style()
        .set_property("transition-property", "opacity, transform")
        .expect("set transition properties");
    root.style()
        .set_property("transition-duration", "10s")
        .expect("set transition durations");
    root.style()
        .set_property("animation-name", "fade")
        .expect("set animation name");
    root.style()
        .set_property("animation-duration", "10s")
        .expect("set animation duration");

    present.set(false);
    render_tick().await;
    dispatch_transition_cancel(&root, "opacity");
    TimeoutFuture::new(20).await;
    assert!(host.first_element_child().is_some());

    let transition = web_sys::TransitionEventInit::new();
    transition.set_bubbles(true);
    transition.set_property_name("transform");
    let transition =
        web_sys::TransitionEvent::new_with_event_init_dict("transitionend", &transition)
            .expect("transform transition event");
    root.dispatch_event(&transition)
        .expect("dispatch transform transition");
    TimeoutFuture::new(20).await;
    assert!(host.first_element_child().is_some());

    dispatch_animation_cancel(&root, "fade");
    TimeoutFuture::new(40).await;
    assert!(host.first_element_child().is_none());
    assert_eq!(*exits.lock().expect("exits lock"), 1);

    drop(mount);
    render_tick().await;
    remove_from_body(&host);
}

#[wasm_bindgen_test]
async fn presence_recomputes_when_exit_motion_changes_without_an_event() {
    let host = append_div("presence-style-change-host");
    let present = RwSignal::new(true);
    let mount = mount_to(host.clone(), move || {
        view! {
            <Presence present=Signal::derive(move || present.get())>
                "Style change"
            </Presence>
        }
    });
    let root = host
        .first_element_child()
        .expect("presence root")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("presence root html element");
    root.style()
        .set_property("transition-property", "opacity")
        .expect("set transition property");
    root.style()
        .set_property("transition-duration", "10s")
        .expect("set transition duration");

    present.set(false);
    render_tick().await;
    assert!(host.first_element_child().is_some());
    root.style()
        .set_property("transition-property", "none")
        .expect("remove transition property");
    root.style()
        .set_property("transition-duration", "0s")
        .expect("remove transition duration");

    TimeoutFuture::new(60).await;
    assert!(host.first_element_child().is_none());

    drop(mount);
    render_tick().await;
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
        .set_property("transition-property", "opacity")
        .expect("set transition property");
    root.style()
        .set_property("transition-duration", "20ms")
        .expect("set transition duration");

    present.set(false);
    render_tick().await;
    assert_eq!(attr(&root, "data-state").as_deref(), Some("closed"));
    assert!(host.first_element_child().is_some());
    assert!(
        exit_callbacks
            .lock()
            .expect("exit callbacks lock")
            .is_empty()
    );

    TimeoutFuture::new(100).await;

    assert!(host.first_element_child().is_none());
    assert_eq!(
        exit_callbacks
            .lock()
            .expect("exit callbacks lock")
            .as_slice(),
        &["timeout"]
    );

    drop(mount);
    render_tick().await;
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
        .set_property("transition-property", "opacity")
        .expect("set transition property");
    root.style()
        .set_property("transition-duration", "80ms")
        .expect("set transition duration");

    present.set(false);
    render_tick().await;
    assert_eq!(attr(&root, "data-state").as_deref(), Some("closed"));
    assert!(host.first_element_child().is_some());

    TimeoutFuture::new(20).await;
    present.set(true);
    render_tick().await;
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
    render_tick().await;
    assert_eq!(attr(&root, "data-state").as_deref(), Some("closed"));
    TimeoutFuture::new(160).await;

    assert!(host.first_element_child().is_none());
    assert_eq!(
        exit_callbacks
            .lock()
            .expect("exit callbacks lock")
            .as_slice(),
        &["timeout"]
    );

    drop(mount);
    render_tick().await;
    remove_from_body(&host);
}

#[wasm_bindgen_test]
async fn presence_reopen_ignores_stale_root_transitionend_and_allows_a_later_transition_exit() {
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
        .set_property("transition-property", "opacity")
        .expect("set transition property");
    root.style()
        .set_property("transition-duration", "10s")
        .expect("set transition duration");

    present.set(false);
    render_tick().await;
    assert_eq!(attr(&root, "data-state").as_deref(), Some("closed"));
    assert!(host.first_element_child().is_some());

    present.set(true);
    render_tick().await;
    assert_eq!(attr(&root, "data-state").as_deref(), Some("open"));
    assert!(host.first_element_child().is_some());
    assert!(
        exit_callbacks
            .lock()
            .expect("exit callbacks lock")
            .is_empty()
    );

    dispatch_transition_end(&root, true);
    TimeoutFuture::new(40).await;
    assert!(host.first_element_child().is_some());
    assert_eq!(attr(&root, "data-state").as_deref(), Some("open"));
    assert!(
        exit_callbacks
            .lock()
            .expect("exit callbacks lock")
            .is_empty()
    );

    present.set(false);
    render_tick().await;
    assert_eq!(attr(&root, "data-state").as_deref(), Some("closed"));
    dispatch_transition_end(&root, true);
    TimeoutFuture::new(40).await;

    assert!(host.first_element_child().is_none());
    assert_eq!(
        exit_callbacks
            .lock()
            .expect("exit callbacks lock")
            .as_slice(),
        &["transition"]
    );

    drop(mount);
    render_tick().await;
    remove_from_body(&host);
}

#[wasm_bindgen_test]
async fn presence_reopen_ignores_stale_root_animationend_and_allows_a_later_animation_exit() {
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
        .set_property("animation-name", "fade")
        .expect("set animation name");
    root.style()
        .set_property("animation-duration", "10s")
        .expect("set animation duration");

    present.set(false);
    render_tick().await;
    assert_eq!(attr(&root, "data-state").as_deref(), Some("closed"));
    assert!(host.first_element_child().is_some());

    present.set(true);
    render_tick().await;
    assert_eq!(attr(&root, "data-state").as_deref(), Some("open"));
    assert!(host.first_element_child().is_some());
    assert!(
        exit_callbacks
            .lock()
            .expect("exit callbacks lock")
            .is_empty()
    );

    dispatch_animation_end(&root, true);
    TimeoutFuture::new(40).await;
    assert!(host.first_element_child().is_some());
    assert_eq!(attr(&root, "data-state").as_deref(), Some("open"));
    assert!(
        exit_callbacks
            .lock()
            .expect("exit callbacks lock")
            .is_empty()
    );

    present.set(false);
    assert!(host.first_element_child().is_some());
    TimeoutFuture::new(40).await;
    assert_eq!(attr(&root, "data-state").as_deref(), Some("closed"));
    dispatch_animation_end(&root, true);
    TimeoutFuture::new(40).await;

    assert!(host.first_element_child().is_none());
    assert_eq!(
        exit_callbacks
            .lock()
            .expect("exit callbacks lock")
            .as_slice(),
        &["animation"]
    );

    drop(mount);
    render_tick().await;
    remove_from_body(&host);
}

#[wasm_bindgen_test]
async fn presence_without_exit_motion_unmounts_immediately_and_runs_exit_complete_once() {
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
        .set_property("transition-property", "opacity")
        .expect("set transition property");
    root.style()
        .set_property("transition-duration", "0s")
        .expect("set transition duration");
    root.style()
        .set_property("animation-name", "fade")
        .expect("set animation name");
    root.style()
        .set_property("animation-duration", "0s")
        .expect("set animation duration");

    present.set(false);
    assert!(host.first_element_child().is_some());
    TimeoutFuture::new(40).await;

    assert!(host.first_element_child().is_none());
    assert_eq!(
        exit_callbacks
            .lock()
            .expect("exit callbacks lock")
            .as_slice(),
        &["immediate"]
    );

    drop(mount);
    render_tick().await;
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
        .set_property("transition-property", "opacity, transform")
        .expect("set transition property");
    root.style()
        .set_property("transition-duration", "30ms, 20ms")
        .expect("set transition duration");
    root.style()
        .set_property("transition-delay", "10ms, 90ms")
        .expect("set transition delay");

    present.set(false);
    render_tick().await;
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

    TimeoutFuture::new(120).await;

    assert!(host.first_element_child().is_none());
    assert_eq!(
        exit_callbacks
            .lock()
            .expect("exit callbacks lock")
            .as_slice(),
        &["longest"]
    );

    drop(mount);
    render_tick().await;
    remove_from_body(&host);
}

#[wasm_bindgen_test]
async fn scroll_lock_release_restores_previous_body_styles() {
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
async fn nested_scroll_lock_guards_preserve_body_lock_until_the_last_drop() {
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
async fn mixed_scroll_lock_release_and_guard_drop_restore_styles_after_the_final_logical_release() {
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
async fn scroll_lock_reacquire_captures_a_fresh_body_style_snapshot() {
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
async fn extra_scroll_lock_release_after_full_unlock_leaves_restored_styles_unchanged() {
    let overflow_before = body_style("overflow");
    let position_before = body_style("position");
    let top_before = body_style("top");
    let width_before = body_style("width");

    set_body_style("overflow", "scroll");
    set_body_style("position", "relative");
    set_body_style("top", "11px");
    set_body_style("width", "52%");

    let guard = scroll_lock_acquire().expect("acquire scroll lock");

    assert_eq!(body_style("overflow"), "hidden");
    assert_eq!(body_style("position"), "fixed");
    assert_eq!(body_style("width"), "100%");

    drop(guard);

    assert_eq!(body_style("overflow"), "scroll");
    assert_eq!(body_style("position"), "relative");
    assert_eq!(body_style("top"), "11px");
    assert_eq!(body_style("width"), "52%");

    scroll_lock_release().expect("extra release on unlocked body");

    assert_eq!(body_style("overflow"), "scroll");
    assert_eq!(body_style("position"), "relative");
    assert_eq!(body_style("top"), "11px");
    assert_eq!(body_style("width"), "52%");

    set_body_style("overflow", &overflow_before);
    set_body_style("position", &position_before);
    set_body_style("top", &top_before);
    set_body_style("width", &width_before);
}

#[wasm_bindgen_test]
async fn scroll_lock_restores_the_original_snapshot_after_mid_lock_style_mutation() {
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
    assert!(
        captured_scroll >= 200.0,
        "captured scroll: {captured_scroll}"
    );

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
