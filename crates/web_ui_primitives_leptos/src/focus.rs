//! Focus management primitives for headless overlays and composites.

use leptos::ev::KeyboardEvent;
use leptos::html;
use leptos::prelude::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

const FOCUSABLE_SELECTOR: &str =
    "a[href],button,textarea,input,select,[tabindex]:not([tabindex='-1'])";

/// Returns the selector used to find focusable descendants in a focus scope.
pub fn focus_scope_selector() -> &'static str {
    FOCUSABLE_SELECTOR
}

/// Returns the next focus index for a trapped tab sequence.
///
/// `shift = true` moves backward; otherwise it moves forward.
pub fn focus_scope_next_index(current: usize, count: usize, shift: bool) -> usize {
    focus_scope_target_index(Some(current), count, shift)
}

#[derive(Clone, Default)]
/// Options for [`use_focus_scope`].
pub struct FocusScopeOptions {
    pub trapped: bool,
    pub auto_focus: bool,
    pub return_focus: bool,
    pub on_mount_auto_focus: Option<Callback<()>>,
    pub on_unmount_auto_focus: Option<Callback<()>>,
}

#[derive(Clone)]
/// Handle returned by [`use_focus_scope`].
pub struct FocusScopeBinding<E>
where
    E: html::ElementType,
{
    node_ref: NodeRef<E>,
}

impl<E> FocusScopeBinding<E>
where
    E: html::ElementType,
{
    /// Returns the [`NodeRef`] that should be attached to the scoped element.
    pub fn node_ref(&self) -> NodeRef<E> {
        self.node_ref
    }
}

#[cfg(not(target_arch = "wasm32"))]
/// Creates a wrapper-free focus scope binding.
pub fn use_focus_scope<E>(options: FocusScopeOptions) -> FocusScopeBinding<E>
where
    E: html::ElementType,
    E::Output: 'static,
{
    let _ = options;
    FocusScopeBinding {
        node_ref: NodeRef::<E>::new(),
    }
}

#[cfg(target_arch = "wasm32")]
/// Creates a wrapper-free focus scope binding.
pub fn use_focus_scope<E>(options: FocusScopeOptions) -> FocusScopeBinding<E>
where
    E: html::ElementType,
    E::Output: wasm_bindgen::JsCast + Clone + 'static,
{
    let node_ref = NodeRef::<E>::new();
    attach_focus_scope(node_ref, options);
    FocusScopeBinding { node_ref }
}

fn focus_scope_target_index(current: Option<usize>, count: usize, shift: bool) -> usize {
    if count == 0 {
        return 0;
    }
    match current {
        Some(current) if shift => {
            if current == 0 {
                count - 1
            } else {
                current - 1
            }
        }
        Some(current) if current + 1 >= count => 0,
        Some(current) => current + 1,
        None if shift => count - 1,
        None => 0,
    }
}

#[component]
/// Wraps children in a focus scope with optional trapping and auto-focus.
///
/// On wasm, this can:
///
/// - auto-focus the first focusable descendant on mount
/// - trap tab navigation within the scope
/// - restore the previously focused element on unmount
///
/// On non-wasm targets, those side effects are skipped and the component
/// renders a focusable wrapper element around `children`.
pub fn FocusScope(
    #[prop(optional)] trapped: bool,
    #[prop(optional)] auto_focus: bool,
    #[prop(optional)] return_focus: bool,
    #[prop(optional)] on_mount_auto_focus: Option<Callback<()>>,
    #[prop(optional)] on_unmount_auto_focus: Option<Callback<()>>,
    children: ChildrenFn,
) -> impl IntoView {
    let scope = use_focus_scope::<html::Div>(FocusScopeOptions {
        trapped,
        auto_focus,
        return_focus,
        on_mount_auto_focus,
        on_unmount_auto_focus,
    });

    let on_keydown = move |event: KeyboardEvent| {
        if trapped && event.key() == "Escape" {
            event.prevent_default();
        }
    };

    view! {
        <div node_ref=scope.node_ref() tabindex="-1" on:keydown=on_keydown>
            {children()}
        </div>
    }
}

#[cfg(target_arch = "wasm32")]
fn attach_focus_scope<E>(node_ref: NodeRef<E>, options: FocusScopeOptions)
where
    E: html::ElementType,
    E::Output: wasm_bindgen::JsCast + Clone + 'static,
{
    use send_wrapper::SendWrapper;
    use wasm_bindgen::JsCast;
    use wasm_bindgen::closure::Closure;

    let document = web_sys::window().and_then(|window| window.document());
    let previous_focus = document
        .as_ref()
        .and_then(|document| document.active_element())
        .map(SendWrapper::new);
    let mounted = RwSignal::new(false);
    let FocusScopeOptions {
        trapped,
        auto_focus,
        return_focus,
        on_mount_auto_focus,
        on_unmount_auto_focus,
    } = options;

    if trapped {
        if let Some(document) = document.clone() {
            let trap_node_ref = node_ref;
            let document_focus = document.clone();
            let handler = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
                if event.key() != "Tab" {
                    return;
                }
                if !focus_event_target_is_inside(event.target(), trap_node_ref) {
                    return;
                }
                let Some(root) = trap_node_ref
                    .get_untracked()
                    .and_then(|root| root.dyn_into::<web_sys::Element>().ok())
                else {
                    return;
                };
                event.prevent_default();
                let _ = focus_scope_cycle(&root, &document_focus, event.shift_key());
            }) as Box<dyn FnMut(_)>);
            let _ = document
                .add_event_listener_with_callback("keydown", handler.as_ref().unchecked_ref());
            let cleanup_document = SendWrapper::new(document);
            let cleanup_handler = SendWrapper::new(handler);
            on_cleanup(move || {
                let document = cleanup_document.take();
                let handler = cleanup_handler.take();
                let _ = document.remove_event_listener_with_callback(
                    "keydown",
                    handler.as_ref().unchecked_ref(),
                );
            });
        }
    }

    let effect = RenderEffect::new(move |_| {
        if mounted.get() {
            return;
        }
        let Some(document) = document.clone() else {
            mounted.set(true);
            return;
        };
        let Some(root) = node_ref.get() else {
            return;
        };
        let Ok(root) = root.dyn_into::<web_sys::Element>() else {
            mounted.set(true);
            return;
        };

        if auto_focus {
            let _ = focus_scope_focus_first(&root, &document);
            if let Some(callback) = on_mount_auto_focus.as_ref() {
                callback.run(());
            }
        }

        mounted.set(true);
    });

    on_cleanup(move || {
        drop(effect);
        if return_focus {
            if let Some(element) = previous_focus {
                let element = element.take();
                let _ = element
                    .dyn_ref::<web_sys::HtmlElement>()
                    .map(|el| el.focus());
            }
            if let Some(callback) = on_unmount_auto_focus.as_ref() {
                callback.run(());
            }
        }
    });
}

#[cfg(target_arch = "wasm32")]
fn focus_event_target_is_inside<E>(
    target: Option<web_sys::EventTarget>,
    node_ref: NodeRef<E>,
) -> bool
where
    E: html::ElementType,
    E::Output: wasm_bindgen::JsCast + Clone + 'static,
{
    use wasm_bindgen::JsCast;

    let target = target.and_then(|target| target.dyn_into::<web_sys::Node>().ok());
    let root = node_ref
        .get_untracked()
        .and_then(|root| root.dyn_into::<web_sys::Node>().ok());

    target
        .as_ref()
        .and_then(|node| root.as_ref().map(|root| root.contains(Some(node))))
        .unwrap_or(false)
}

#[cfg(target_arch = "wasm32")]
fn focus_scope_focus_first(
    root: &web_sys::Element,
    document: &web_sys::Document,
) -> Result<(), ()> {
    let list = root
        .query_selector_all(FOCUSABLE_SELECTOR)
        .map_err(|_| ())?;
    if list.length() == 0 {
        if let Some(element) = root.dyn_ref::<web_sys::HtmlElement>() {
            let _ = element.focus();
        }
        return Ok(());
    }
    let first = list
        .item(0)
        .and_then(|node| node.dyn_into::<web_sys::HtmlElement>().ok())
        .or_else(|| {
            document
                .active_element()
                .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok())
        });
    if let Some(element) = first {
        let _ = element.focus();
    }
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn focus_scope_cycle(
    root: &web_sys::Element,
    document: &web_sys::Document,
    shift: bool,
) -> Result<(), ()> {
    let list = root
        .query_selector_all(FOCUSABLE_SELECTOR)
        .map_err(|_| ())?;
    let count = list.length() as usize;
    if count == 0 {
        return Ok(());
    }
    let active = document.active_element();
    let mut current_index = None;
    if let Some(active) = active {
        for index in 0..count {
            let candidate = list
                .item(index as u32)
                .and_then(|node| node.dyn_into::<web_sys::Element>().ok());
            if let Some(candidate) = candidate {
                if active.is_same_node(Some(&candidate)) {
                    current_index = Some(index);
                    break;
                }
            }
        }
    }
    let next_index = focus_scope_target_index(current_index, count, shift);
    if let Some(next) = list
        .item(next_index as u32)
        .and_then(|node| node.dyn_into::<web_sys::HtmlElement>().ok())
    {
        let _ = next.focus();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{focus_scope_next_index, focus_scope_selector, focus_scope_target_index};

    #[test]
    fn focus_scope_selector_is_non_empty() {
        assert!(!focus_scope_selector().is_empty());
    }

    #[test]
    fn focus_scope_next_index_wraps() {
        assert_eq!(focus_scope_next_index(0, 3, true), 2);
        assert_eq!(focus_scope_next_index(2, 3, false), 0);
    }

    #[test]
    fn focus_scope_target_index_starts_from_scope_root() {
        assert_eq!(focus_scope_target_index(None, 3, false), 0);
        assert_eq!(focus_scope_target_index(None, 3, true), 2);
    }
}
