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

#[component]
/// Headless layer that reports escape, outside pointer, and outside focus
/// dismissal requests.
///
/// `disable_pointer_down_outside_dismiss` suppresses pointer-down-outside
/// dismissal handling.
///
/// `disable_outside_pointer_events` remains available as a compatibility alias
/// for the same behavior. Neither prop mutates CSS `pointer-events`.
pub fn DismissibleLayer(
    #[prop(optional)] on_dismiss: Option<Callback<DismissibleReason>>,
    #[prop(optional)] on_escape_key_down: Option<Callback<KeyboardEvent>>,
    #[prop(optional)] on_pointer_down_outside: Option<Callback<PointerEvent>>,
    #[prop(optional)] on_focus_outside: Option<Callback<FocusEvent>>,
    #[prop(optional)] disable_pointer_down_outside_dismiss: bool,
    #[prop(optional)] disable_outside_pointer_events: bool,
    children: ChildrenFn,
) -> impl IntoView {
    let node_ref = NodeRef::<html::Div>::new();
    let disable_pointer_down_outside_dismiss = pointer_down_outside_dismiss_disabled(
        disable_pointer_down_outside_dismiss,
        disable_outside_pointer_events,
    );

    let on_keydown = move |event: KeyboardEvent| {
        if !dismissible_is_escape(&event.key()) {
            return;
        }
        if let Some(callback) = on_escape_key_down.as_ref() {
            callback.run(event.clone());
        }
        if let Some(callback) = on_dismiss.as_ref() {
            callback.run(DismissibleReason::Escape);
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
        let node_ref = node_ref;

        node_ref.on_load(move |root| {
            let document = match web_sys::window().and_then(|window| window.document()) {
                Some(document) => document,
                None => return,
            };

            if !disable_pointer_down_outside_dismiss {
                let root_pointer = root.clone();
                let on_dismiss = on_dismiss.clone();
                let on_pointer_down_outside = on_pointer_down_outside.clone();
                let handler = Closure::wrap(Box::new(move |event: web_sys::PointerEvent| {
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
        });
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = on_pointer_down_outside;
        let _ = on_focus_outside;
        let _ = disable_pointer_down_outside_dismiss;
        let _ = disable_outside_pointer_events;
    }

    view! {
        <div node_ref=node_ref on:keydown=on_keydown>
            {children()}
        </div>
    }
}

fn pointer_down_outside_dismiss_disabled(
    disable_pointer_down_outside_dismiss: bool,
    disable_outside_pointer_events: bool,
) -> bool {
    disable_pointer_down_outside_dismiss || disable_outside_pointer_events
}

#[cfg(test)]
mod tests {
    use super::{
        dismissible_is_escape, dismissible_is_outside, pointer_down_outside_dismiss_disabled,
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
        assert!(pointer_down_outside_dismiss_disabled(true, false));
    }

    #[test]
    fn legacy_pointer_event_flag_remains_a_compatibility_alias() {
        assert!(pointer_down_outside_dismiss_disabled(false, true));
    }

    #[test]
    fn outside_pointer_dismiss_stays_enabled_when_both_flags_are_false() {
        assert!(!pointer_down_outside_dismiss_disabled(false, false));
    }
}
