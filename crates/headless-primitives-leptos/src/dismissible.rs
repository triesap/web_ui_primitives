#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
use std::{cell::Cell, rc::Rc};

use leptos::ev::{FocusEvent, KeyboardEvent, PointerEvent};
use leptos::html;
use leptos::prelude::*;

/// Reason emitted by [`DismissibleLayer`] when it requests dismissal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DismissibleReason {
    Escape,
    PointerDownOutside,
    FocusOutside,
}

/// Returns `true` when a key value should dismiss via escape handling.
pub fn dismissible_is_escape(key: &str) -> bool {
    key == "Escape"
}

/// Returns `true` when an interaction target is outside the active layer.
pub fn dismissible_is_outside(is_inside: bool) -> bool {
    !is_inside
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Default)]
struct DismissibleState {
    next_id: u64,
    layers: Vec<u64>,
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    static DISMISSIBLE_STATE: RefCell<DismissibleState> = RefCell::new(DismissibleState::default());
}

#[cfg(target_arch = "wasm32")]
fn dismissible_state_with<T>(f: impl FnOnce(&mut DismissibleState) -> T) -> T {
    DISMISSIBLE_STATE.with(|state| f(&mut state.borrow_mut()))
}

#[cfg(target_arch = "wasm32")]
fn dismissible_layer_register() -> u64 {
    dismissible_state_with(|state| {
        let id = state.next_id;
        state.next_id = state.next_id.saturating_add(1);
        state.layers.push(id);
        id
    })
}

#[cfg(target_arch = "wasm32")]
fn dismissible_layer_unregister(id: u64) {
    dismissible_state_with(|state| dismissible_layer_remove_from_stack(&mut state.layers, id));
}

#[cfg(target_arch = "wasm32")]
fn dismissible_layer_is_topmost(id: u64) -> bool {
    dismissible_state_with(|state| dismissible_layer_is_topmost_for_stack(&state.layers, id))
}

#[cfg(not(target_arch = "wasm32"))]
fn dismissible_layer_is_topmost(_id: u64) -> bool {
    false
}

#[component]
/// Headless layer that reports escape, outside pointer, and outside focus
/// dismissal requests.
///
/// `disable_pointer_down_outside_dismiss` suppresses pointer-down-outside
/// dismissal handling. This prop does not mutate CSS `pointer-events`.
pub fn DismissibleLayer(
    #[prop(optional)] on_dismiss: Option<Callback<DismissibleReason>>,
    #[prop(optional)] on_escape_key_down: Option<Callback<KeyboardEvent>>,
    #[prop(optional)] on_pointer_down_outside: Option<Callback<PointerEvent>>,
    #[prop(optional)] on_focus_outside: Option<Callback<FocusEvent>>,
    #[prop(optional)] disable_pointer_down_outside_dismiss: bool,
    children: ChildrenFn,
) -> impl IntoView {
    let node_ref = NodeRef::<html::Div>::new();
    let disable_pointer_down_outside_dismiss =
        pointer_down_outside_dismiss_disabled(disable_pointer_down_outside_dismiss);
    let layer_id = Rc::new(Cell::new(None::<u64>));

    let on_keydown = {
        let layer_id = Rc::clone(&layer_id);
        move |event: KeyboardEvent| {
            let Some(layer_id) = layer_id.get() else {
                return;
            };
            if !dismissible_layer_is_topmost(layer_id) {
                return;
            }
            if !dismissible_is_escape(&event.key()) {
                return;
            }
            if let Some(callback) = on_escape_key_down.as_ref() {
                callback.run(event.clone());
            }
            if let Some(callback) = on_dismiss.as_ref() {
                callback.run(DismissibleReason::Escape);
            }
        }
    };

    #[cfg(target_arch = "wasm32")]
    {
        use send_wrapper::SendWrapper;
        use wasm_bindgen::JsCast;
        use wasm_bindgen::closure::Closure;

        let on_dismiss = on_dismiss.clone();
        let on_pointer_down_outside = on_pointer_down_outside.clone();
        let on_focus_outside = on_focus_outside.clone();
        let layer_id = Rc::clone(&layer_id);
        let node_ref = node_ref;

        node_ref.on_load(move |root| {
            let document = match web_sys::window().and_then(|window| window.document()) {
                Some(document) => document,
                None => return,
            };
            let id = dismissible_layer_register();
            layer_id.set(Some(id));

            if !disable_pointer_down_outside_dismiss {
                let root_pointer = root.clone();
                let on_dismiss = on_dismiss.clone();
                let on_pointer_down_outside = on_pointer_down_outside.clone();
                let handler = Closure::wrap(Box::new(move |event: web_sys::PointerEvent| {
                    if !dismissible_layer_is_topmost(id) {
                        return;
                    }
                    let target = event
                        .target()
                        .and_then(|target| target.dyn_into::<web_sys::Node>().ok());
                    let is_inside = target
                        .as_ref()
                        .map(|node| root_pointer.contains(Some(node)))
                        .unwrap_or(false);
                    if dismissible_is_outside(is_inside) {
                        if let Some(callback) = on_pointer_down_outside.as_ref() {
                            callback.run(event.clone());
                        }
                        if let Some(callback) = on_dismiss.as_ref() {
                            callback.run(DismissibleReason::PointerDownOutside);
                        }
                    }
                }) as Box<dyn FnMut(_)>);
                let _ = document.add_event_listener_with_callback(
                    "pointerdown",
                    handler.as_ref().unchecked_ref(),
                );
                let cleanup_doc = SendWrapper::new(document.clone());
                let cleanup_handler = SendWrapper::new(handler);
                on_cleanup(move || {
                    let document = cleanup_doc.take();
                    let handler = cleanup_handler.take();
                    let _ = document.remove_event_listener_with_callback(
                        "pointerdown",
                        handler.as_ref().unchecked_ref(),
                    );
                });
            }

            let root_focus = root.clone();
            let on_dismiss = on_dismiss.clone();
            let on_focus_outside = on_focus_outside.clone();
            let focus_handler = Closure::wrap(Box::new(move |event: web_sys::FocusEvent| {
                if !dismissible_layer_is_topmost(id) {
                    return;
                }
                let target = event
                    .target()
                    .and_then(|target| target.dyn_into::<web_sys::Node>().ok());
                let is_inside = target
                    .as_ref()
                    .map(|node| root_focus.contains(Some(node)))
                    .unwrap_or(false);
                if dismissible_is_outside(is_inside) {
                    if let Some(callback) = on_focus_outside.as_ref() {
                        callback.run(event.clone());
                    }
                    if let Some(callback) = on_dismiss.as_ref() {
                        callback.run(DismissibleReason::FocusOutside);
                    }
                }
            }) as Box<dyn FnMut(_)>);
            let _ = document.add_event_listener_with_callback(
                "focusin",
                focus_handler.as_ref().unchecked_ref(),
            );
            let cleanup_doc = SendWrapper::new(document);
            let cleanup_handler = SendWrapper::new(focus_handler);
            on_cleanup(move || {
                let document = cleanup_doc.take();
                let handler = cleanup_handler.take();
                let _ = document.remove_event_listener_with_callback(
                    "focusin",
                    handler.as_ref().unchecked_ref(),
                );
            });

            on_cleanup(move || {
                dismissible_layer_unregister(id);
            });
        });
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = on_pointer_down_outside;
        let _ = on_focus_outside;
        let _ = disable_pointer_down_outside_dismiss;
        let _ = layer_id;
    }

    view! {
        <div node_ref=node_ref on:keydown=on_keydown>
            {children()}
        </div>
    }
}

fn pointer_down_outside_dismiss_disabled(disable_pointer_down_outside_dismiss: bool) -> bool {
    disable_pointer_down_outside_dismiss
}

#[cfg_attr(not(any(test, target_arch = "wasm32")), allow(dead_code))]
fn dismissible_layer_is_topmost_for_stack(layers: &[u64], id: u64) -> bool {
    layers.last().copied() == Some(id)
}

#[cfg_attr(not(any(test, target_arch = "wasm32")), allow(dead_code))]
fn dismissible_layer_remove_from_stack(layers: &mut Vec<u64>, id: u64) {
    if let Some(index) = layers.iter().position(|layer_id| *layer_id == id) {
        layers.remove(index);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        dismissible_is_escape, dismissible_is_outside, dismissible_layer_is_topmost_for_stack,
        dismissible_layer_remove_from_stack, pointer_down_outside_dismiss_disabled,
    };

    #[test]
    fn dismissible_escape_match() {
        assert!(dismissible_is_escape("Escape"));
        assert!(!dismissible_is_escape("Enter"));
    }

    #[test]
    fn dismissible_outside_check() {
        assert!(dismissible_is_outside(false));
        assert!(!dismissible_is_outside(true));
    }

    #[test]
    fn canonical_pointer_down_outside_dismiss_flag_disables_behavior() {
        assert!(pointer_down_outside_dismiss_disabled(true));
    }

    #[test]
    fn outside_pointer_dismiss_stays_enabled_when_both_flags_are_false() {
        assert!(!pointer_down_outside_dismiss_disabled(false));
    }

    #[test]
    fn dismissible_layer_topmost_check_uses_last_registered_layer() {
        assert!(!dismissible_layer_is_topmost_for_stack(&[], 0));
        assert!(!dismissible_layer_is_topmost_for_stack(&[1, 2], 1));
        assert!(dismissible_layer_is_topmost_for_stack(&[1, 2], 2));
    }

    #[test]
    fn dismissible_layer_remove_from_stack_reveals_the_previous_topmost_layer() {
        let mut layers = vec![1, 2, 3];

        dismissible_layer_remove_from_stack(&mut layers, 2);
        assert_eq!(layers, vec![1, 3]);
        assert!(dismissible_layer_is_topmost_for_stack(&layers, 3));

        dismissible_layer_remove_from_stack(&mut layers, 3);
        assert_eq!(layers, vec![1]);
        assert!(dismissible_layer_is_topmost_for_stack(&layers, 1));
    }
}
