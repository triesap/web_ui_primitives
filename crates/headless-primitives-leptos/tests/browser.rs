#![cfg(target_arch = "wasm32")]

use headless_primitives_leptos::{DismissibleLayer, DismissibleReason, Portal};
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
