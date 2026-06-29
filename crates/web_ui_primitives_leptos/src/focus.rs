//! Focus management primitives for headless overlays and composites.

use leptos::ev::KeyboardEvent;
use leptos::html;
use leptos::prelude::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

const FOCUSABLE_SELECTOR: &str =
    "a[href],button,textarea,input,select,[tabindex]:not([tabindex='-1'])";
#[cfg(target_arch = "wasm32")]
const FOCUS_SCOPE_ATTR: &str = "data-web-ui-focus-scope";
#[cfg(target_arch = "wasm32")]
const FOCUS_SCOPE_MARKER_SELECTOR: &str = "[data-web-ui-focus-scope]";

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
    use_focus_scope_with_node_ref(NodeRef::<E>::new(), options)
}

#[cfg(not(target_arch = "wasm32"))]
/// Creates a wrapper-free focus scope binding from an existing [`NodeRef`].
pub fn use_focus_scope_with_node_ref<E>(
    node_ref: NodeRef<E>,
    options: FocusScopeOptions,
) -> FocusScopeBinding<E>
where
    E: html::ElementType,
    E::Output: 'static,
{
    let _ = options;
    FocusScopeBinding { node_ref }
}

#[cfg(target_arch = "wasm32")]
/// Creates a wrapper-free focus scope binding.
pub fn use_focus_scope<E>(options: FocusScopeOptions) -> FocusScopeBinding<E>
where
    E: html::ElementType,
    E::Output: wasm_bindgen::JsCast + Clone + 'static,
{
    use_focus_scope_with_node_ref(NodeRef::<E>::new(), options)
}

#[cfg(target_arch = "wasm32")]
/// Creates a wrapper-free focus scope binding from an existing [`NodeRef`].
pub fn use_focus_scope_with_node_ref<E>(
    node_ref: NodeRef<E>,
    options: FocusScopeOptions,
) -> FocusScopeBinding<E>
where
    E: html::ElementType,
    E::Output: wasm_bindgen::JsCast + Clone + 'static,
{
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
                if !focus_event_target_is_owned_by_scope(event.target(), &root) {
                    return;
                }
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
        let _ = root.set_attribute(FOCUS_SCOPE_ATTR, "");

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
fn focus_event_target_is_owned_by_scope(
    target: Option<web_sys::EventTarget>,
    root: &web_sys::Element,
) -> bool {
    use wasm_bindgen::JsCast;

    let Some(target) = target.and_then(|target| target.dyn_into::<web_sys::Element>().ok()) else {
        return false;
    };
    let closest_scope = target.closest(FOCUS_SCOPE_MARKER_SELECTOR).ok().flatten();

    closest_scope
        .as_ref()
        .is_some_and(|scope| scope.is_same_node(Some(root)))
}

#[cfg(target_arch = "wasm32")]
fn focus_scope_focus_first(
    root: &web_sys::Element,
    document: &web_sys::Document,
) -> Result<(), ()> {
    let elements = focus_scope_tabbable_elements(root)?;
    if elements.is_empty() {
        if let Some(element) = root.dyn_ref::<web_sys::HtmlElement>() {
            let _ = element.focus();
        }
        return Ok(());
    }
    let first = elements.into_iter().next().or_else(|| {
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
    let elements = focus_scope_tabbable_elements(root)?;
    let count = elements.len();
    if count == 0 {
        return Ok(());
    }
    let active = document.active_element();
    let mut current_index = None;
    if let Some(active) = active {
        for index in 0..count {
            if active.is_same_node(Some(&elements[index])) {
                current_index = Some(index);
                break;
            }
        }
    }
    let next_index = focus_scope_target_index(current_index, count, shift);
    if let Some(next) = elements.get(next_index) {
        let _ = next.focus();
    }
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn focus_scope_tabbable_elements(root: &web_sys::Element) -> Result<Vec<web_sys::HtmlElement>, ()> {
    let list = root
        .query_selector_all(FOCUSABLE_SELECTOR)
        .map_err(|_| ())?;
    let mut elements = Vec::new();

    for index in 0..list.length() {
        let Some(element) = list
            .item(index)
            .and_then(|node| node.dyn_into::<web_sys::Element>().ok())
        else {
            continue;
        };
        if !focus_scope_candidate_is_tabbable(root, &element) {
            continue;
        }
        if let Ok(element) = element.dyn_into::<web_sys::HtmlElement>() {
            elements.push(element);
        }
    }

    Ok(elements)
}

#[cfg(target_arch = "wasm32")]
fn focus_scope_candidate_is_tabbable(root: &web_sys::Element, element: &web_sys::Element) -> bool {
    focus_scope_candidate_belongs_to_scope(root, element)
        && !focus_scope_candidate_is_disabled(element)
        && !focus_scope_candidate_has_negative_tabindex(element)
        && !focus_scope_candidate_has_hidden_ancestor(root, element)
        && focus_scope_candidate_has_layout_box(element)
}

#[cfg(target_arch = "wasm32")]
fn focus_scope_candidate_belongs_to_scope(
    root: &web_sys::Element,
    element: &web_sys::Element,
) -> bool {
    let closest_scope = element.closest(FOCUS_SCOPE_MARKER_SELECTOR).ok().flatten();

    closest_scope
        .as_ref()
        .is_some_and(|scope| scope.is_same_node(Some(root)))
}

#[cfg(target_arch = "wasm32")]
fn focus_scope_candidate_is_disabled(element: &web_sys::Element) -> bool {
    element.matches(":disabled").unwrap_or(false) || element.has_attribute("disabled")
}

#[cfg(target_arch = "wasm32")]
fn focus_scope_candidate_has_negative_tabindex(element: &web_sys::Element) -> bool {
    element
        .get_attribute("tabindex")
        .as_deref()
        .and_then(|value| value.trim().parse::<i32>().ok())
        .is_some_and(|value| value < 0)
}

#[cfg(target_arch = "wasm32")]
fn focus_scope_candidate_has_hidden_ancestor(
    root: &web_sys::Element,
    element: &web_sys::Element,
) -> bool {
    let mut current = Some(element.clone());

    while let Some(candidate) = current {
        if candidate.has_attribute("hidden")
            || candidate.has_attribute("inert")
            || candidate
                .get_attribute("aria-hidden")
                .is_some_and(|value| value.eq_ignore_ascii_case("true"))
            || focus_scope_element_has_non_visible_style(&candidate)
        {
            return true;
        }

        if candidate.is_same_node(Some(root)) {
            return false;
        }

        current = candidate.parent_element();
    }

    true
}

#[cfg(target_arch = "wasm32")]
fn focus_scope_element_has_non_visible_style(element: &web_sys::Element) -> bool {
    let Some(window) = web_sys::window() else {
        return false;
    };
    let Ok(Some(style)) = window.get_computed_style(element) else {
        return false;
    };
    let display = style.get_property_value("display").unwrap_or_default();
    let visibility = style.get_property_value("visibility").unwrap_or_default();

    display == "none" || matches!(visibility.as_str(), "hidden" | "collapse")
}

#[cfg(target_arch = "wasm32")]
fn focus_scope_candidate_has_layout_box(element: &web_sys::Element) -> bool {
    element
        .dyn_ref::<web_sys::HtmlElement>()
        .is_none_or(|element| element.offset_width() > 0 || element.offset_height() > 0)
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
