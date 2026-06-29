#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

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
/// Branch root that should be treated as inside a dismissible layer.
pub type DismissibleBranch = web_sys::Element;

#[cfg(not(target_arch = "wasm32"))]
/// Branch root that should be treated as inside a dismissible layer.
pub type DismissibleBranch = ();

#[derive(Clone)]
/// Cancellable dismissible interaction event.
pub struct DismissibleEvent<E> {
    event: E,
    default_prevented: Arc<AtomicBool>,
}

impl<E> DismissibleEvent<E> {
    /// Creates a dismissible event from a DOM event.
    pub fn new(event: E) -> Self {
        Self {
            event,
            default_prevented: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Returns the underlying DOM event.
    pub fn event(&self) -> &E {
        &self.event
    }

    /// Returns `true` when dismissing default behavior has been prevented.
    pub fn default_prevented(&self) -> bool {
        self.default_prevented.load(Ordering::SeqCst)
    }
}

impl DismissibleEvent<PointerEvent> {
    /// Prevents the dismissible default action and the underlying DOM default.
    pub fn prevent_default(&self) {
        self.event.prevent_default();
        self.default_prevented.store(true, Ordering::SeqCst);
    }
}

impl DismissibleEvent<FocusEvent> {
    /// Prevents the dismissible default action and the underlying DOM default.
    pub fn prevent_default(&self) {
        self.event.prevent_default();
        self.default_prevented.store(true, Ordering::SeqCst);
    }
}

impl DismissibleEvent<KeyboardEvent> {
    /// Prevents the dismissible default action and the underlying DOM default.
    pub fn prevent_default(&self) {
        self.event.prevent_default();
        self.default_prevented.store(true, Ordering::SeqCst);
    }
}

/// Cancellable pointer-down-outside event.
pub type DismissiblePointerDownOutsideEvent = DismissibleEvent<PointerEvent>;

/// Cancellable focus-outside event.
pub type DismissibleFocusOutsideEvent = DismissibleEvent<FocusEvent>;

/// Cancellable escape-key event.
pub type DismissibleEscapeKeyDownEvent = DismissibleEvent<KeyboardEvent>;

#[derive(Clone, Default)]
/// Options for [`use_dismissible_layer`].
pub struct DismissibleLayerOptions {
    pub active: Option<Signal<bool>>,
    pub on_dismiss: Option<Callback<DismissibleReason>>,
    pub on_escape_key_down: Option<Callback<DismissibleEscapeKeyDownEvent>>,
    pub on_pointer_down_outside: Option<Callback<DismissiblePointerDownOutsideEvent>>,
    pub on_focus_outside: Option<Callback<DismissibleFocusOutsideEvent>>,
    pub disable_pointer_down_outside_dismiss: bool,
    pub branches: Vec<DismissibleBranch>,
}

#[derive(Clone)]
/// Handle returned by [`use_dismissible_layer`].
pub struct DismissibleLayerBinding<E>
where
    E: html::ElementType,
{
    node_ref: NodeRef<E>,
}

impl<E> DismissibleLayerBinding<E>
where
    E: html::ElementType,
{
    /// Returns the [`NodeRef`] that should be attached to the layer element.
    pub fn node_ref(&self) -> NodeRef<E> {
        self.node_ref
    }
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
/// Creates a wrapper-free dismissible layer binding.
pub fn use_dismissible_layer<E>(options: DismissibleLayerOptions) -> DismissibleLayerBinding<E>
where
    E: html::ElementType,
    E::Output: 'static,
{
    use_dismissible_layer_with_node_ref(NodeRef::<E>::new(), options)
}

#[cfg(not(target_arch = "wasm32"))]
/// Creates a wrapper-free dismissible layer binding from an existing [`NodeRef`].
pub fn use_dismissible_layer_with_node_ref<E>(
    node_ref: NodeRef<E>,
    options: DismissibleLayerOptions,
) -> DismissibleLayerBinding<E>
where
    E: html::ElementType,
    E::Output: 'static,
{
    let _ = options;
    DismissibleLayerBinding { node_ref }
}

#[cfg(target_arch = "wasm32")]
/// Creates a wrapper-free dismissible layer binding.
pub fn use_dismissible_layer<E>(options: DismissibleLayerOptions) -> DismissibleLayerBinding<E>
where
    E: html::ElementType,
    E::Output: wasm_bindgen::JsCast + Clone + 'static,
{
    use_dismissible_layer_with_node_ref(NodeRef::<E>::new(), options)
}

#[cfg(target_arch = "wasm32")]
/// Creates a wrapper-free dismissible layer binding from an existing [`NodeRef`].
pub fn use_dismissible_layer_with_node_ref<E>(
    node_ref: NodeRef<E>,
    options: DismissibleLayerOptions,
) -> DismissibleLayerBinding<E>
where
    E: html::ElementType,
    E::Output: wasm_bindgen::JsCast + Clone + 'static,
{
    attach_dismissible_layer(node_ref, options);
    DismissibleLayerBinding { node_ref }
}

#[component]
/// Headless layer that reports escape, outside pointer, and outside focus
/// dismissal requests.
///
/// `disable_pointer_down_outside_dismiss` suppresses pointer-down-outside
/// dismissal handling. This prop does not mutate CSS `pointer-events`.
pub fn DismissibleLayer(
    #[prop(optional)] on_dismiss: Option<Callback<DismissibleReason>>,
    #[prop(optional)] on_escape_key_down: Option<Callback<DismissibleEscapeKeyDownEvent>>,
    #[prop(optional)] on_pointer_down_outside: Option<Callback<DismissiblePointerDownOutsideEvent>>,
    #[prop(optional)] on_focus_outside: Option<Callback<DismissibleFocusOutsideEvent>>,
    #[prop(optional)] disable_pointer_down_outside_dismiss: bool,
    #[prop(optional)] branches: Vec<DismissibleBranch>,
    children: ChildrenFn,
) -> impl IntoView {
    let layer = use_dismissible_layer::<html::Div>(DismissibleLayerOptions {
        active: None,
        on_dismiss,
        on_escape_key_down,
        on_pointer_down_outside,
        on_focus_outside,
        disable_pointer_down_outside_dismiss,
        branches,
    });

    view! {
        <div node_ref=layer.node_ref()>
            {children()}
        </div>
    }
}

#[cfg(target_arch = "wasm32")]
fn attach_dismissible_layer<E>(node_ref: NodeRef<E>, options: DismissibleLayerOptions)
where
    E: html::ElementType,
    E::Output: wasm_bindgen::JsCast + Clone + 'static,
{
    use send_wrapper::SendWrapper;
    use wasm_bindgen::JsCast;
    use wasm_bindgen::closure::Closure;

    let layer_id = dismissible_layer_register();
    let registered = Arc::new(AtomicBool::new(true));
    let disable_pointer_down_outside_dismiss =
        pointer_down_outside_dismiss_disabled(options.disable_pointer_down_outside_dismiss);
    let document = web_sys::window().and_then(|window| window.document());
    let active = options.active.unwrap_or_else(|| Signal::derive(|| true));
    let branches = options.branches;

    let registration_registered = Arc::clone(&registered);
    let registration_effect = RenderEffect::new(move |_| {
        let is_active = active.get();
        let is_registered = registration_registered.load(Ordering::SeqCst);
        match (is_active, is_registered) {
            (true, false) => {
                dismissible_state_with(|state| {
                    if !state.layers.contains(&layer_id) {
                        state.layers.push(layer_id);
                    }
                });
                registration_registered.store(true, Ordering::SeqCst);
            }
            (false, true) => {
                dismissible_layer_unregister(layer_id);
                registration_registered.store(false, Ordering::SeqCst);
            }
            _ => {}
        }
    });

    if let Some(document) = document {
        if !disable_pointer_down_outside_dismiss {
            let on_dismiss = options.on_dismiss.clone();
            let on_pointer_down_outside = options.on_pointer_down_outside.clone();
            let pointer_active = active;
            let pointer_node_ref = node_ref;
            let pointer_branches = branches.clone();
            let handler = Closure::wrap(Box::new(move |event: web_sys::PointerEvent| {
                if !pointer_active.get_untracked() {
                    return;
                }
                if !dismissible_layer_is_topmost(layer_id) {
                    return;
                }
                let is_inside = dismissible_event_target_is_inside(
                    event.target(),
                    pointer_node_ref,
                    &pointer_branches,
                );
                if dismissible_is_outside(is_inside) {
                    let outside_event = DismissibleEvent::new(event.clone());
                    if let Some(callback) = on_pointer_down_outside.as_ref() {
                        callback.run(outside_event.clone());
                    }
                    if outside_event.default_prevented() || event.default_prevented() {
                        return;
                    }
                    if let Some(callback) = on_dismiss.as_ref() {
                        callback.run(DismissibleReason::PointerDownOutside);
                    }
                }
            }) as Box<dyn FnMut(_)>);
            let _ = document
                .add_event_listener_with_callback("pointerdown", handler.as_ref().unchecked_ref());
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

        let on_dismiss = options.on_dismiss.clone();
        let on_focus_outside = options.on_focus_outside.clone();
        let focus_active = active;
        let focus_node_ref = node_ref;
        let focus_branches = branches.clone();
        let focus_handler = Closure::wrap(Box::new(move |event: web_sys::FocusEvent| {
            if !focus_active.get_untracked() {
                return;
            }
            if !dismissible_layer_is_topmost(layer_id) {
                return;
            }
            let is_inside =
                dismissible_event_target_is_inside(event.target(), focus_node_ref, &focus_branches);
            if dismissible_is_outside(is_inside) {
                let outside_event = DismissibleEvent::new(event.clone());
                if let Some(callback) = on_focus_outside.as_ref() {
                    callback.run(outside_event.clone());
                }
                if outside_event.default_prevented() || event.default_prevented() {
                    return;
                }
                if let Some(callback) = on_dismiss.as_ref() {
                    callback.run(DismissibleReason::FocusOutside);
                }
            }
        }) as Box<dyn FnMut(_)>);
        let _ = document
            .add_event_listener_with_callback("focusin", focus_handler.as_ref().unchecked_ref());
        let cleanup_doc = SendWrapper::new(document.clone());
        let cleanup_handler = SendWrapper::new(focus_handler);
        on_cleanup(move || {
            let document = cleanup_doc.take();
            let handler = cleanup_handler.take();
            let _ = document
                .remove_event_listener_with_callback("focusin", handler.as_ref().unchecked_ref());
        });

        let on_dismiss = options.on_dismiss;
        let on_escape_key_down = options.on_escape_key_down;
        let key_active = active;
        let key_node_ref = node_ref;
        let key_branches = branches;
        let key_handler = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
            if !key_active.get_untracked() {
                return;
            }
            if !dismissible_layer_is_topmost(layer_id) {
                return;
            }
            if !dismissible_event_target_is_inside(event.target(), key_node_ref, &key_branches) {
                return;
            }
            if !dismissible_is_escape(&event.key()) {
                return;
            }
            let escape_event = DismissibleEvent::new(event.clone());
            if let Some(callback) = on_escape_key_down.as_ref() {
                callback.run(escape_event.clone());
            }
            if escape_event.default_prevented() || event.default_prevented() {
                return;
            }
            if let Some(callback) = on_dismiss.as_ref() {
                callback.run(DismissibleReason::Escape);
            }
        }) as Box<dyn FnMut(_)>);
        let _ = document
            .add_event_listener_with_callback("keydown", key_handler.as_ref().unchecked_ref());
        let cleanup_doc = SendWrapper::new(document);
        let cleanup_handler = SendWrapper::new(key_handler);
        on_cleanup(move || {
            let document = cleanup_doc.take();
            let handler = cleanup_handler.take();
            let _ = document
                .remove_event_listener_with_callback("keydown", handler.as_ref().unchecked_ref());
        });
    }

    on_cleanup(move || {
        drop(registration_effect);
        if registered.swap(false, Ordering::SeqCst) {
            dismissible_layer_unregister(layer_id);
        }
    });
}

#[cfg(target_arch = "wasm32")]
fn dismissible_event_target_is_inside<E>(
    target: Option<web_sys::EventTarget>,
    node_ref: NodeRef<E>,
    branches: &[DismissibleBranch],
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

    let Some(target) = target.as_ref() else {
        return false;
    };
    if root
        .as_ref()
        .is_some_and(|root| root.contains(Some(target)))
    {
        return true;
    }
    branches.iter().any(|branch| branch.contains(Some(target)))
}

#[cfg_attr(not(any(test, target_arch = "wasm32")), allow(dead_code))]
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
