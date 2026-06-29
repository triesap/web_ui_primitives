//! Dialog behavior composition for accessible overlays.

#[cfg(target_arch = "wasm32")]
use std::sync::{Arc, Mutex};

use leptos::html;
use leptos::prelude::*;

use crate::{
    DismissibleBranch, DismissibleEscapeKeyDownEvent, DismissibleFocusOutsideEvent,
    DismissibleLayerOptions, DismissiblePointerDownOutsideEvent, DismissibleReason,
    FocusScopeOptions, ModalError, PresenceBinding, ScrollLockError,
    use_dismissible_layer_with_node_ref, use_focus_scope_with_node_ref, use_presence_with_node_ref,
};

/// Runtime error emitted while applying dialog side effects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialogLayerError {
    Modal(ModalError),
    ScrollLock(ScrollLockError),
}

#[derive(Clone)]
/// Options for [`use_dialog_layer`].
pub struct DialogLayerOptions {
    pub open: Signal<bool>,
    pub modal: bool,
    pub on_dismiss: Option<Callback<DismissibleReason>>,
    pub on_escape_key_down: Option<Callback<DismissibleEscapeKeyDownEvent>>,
    pub on_pointer_down_outside: Option<Callback<DismissiblePointerDownOutsideEvent>>,
    pub on_focus_outside: Option<Callback<DismissibleFocusOutsideEvent>>,
    pub disable_pointer_down_outside_dismiss: bool,
    pub branches: Vec<DismissibleBranch>,
    pub on_mount_auto_focus: Option<Callback<()>>,
    pub on_unmount_auto_focus: Option<Callback<()>>,
    pub on_error: Option<Callback<DialogLayerError>>,
}

impl DialogLayerOptions {
    /// Creates dialog layer options for an open signal.
    pub fn new(open: impl Into<Signal<bool>>) -> Self {
        Self {
            open: open.into(),
            modal: true,
            on_dismiss: None,
            on_escape_key_down: None,
            on_pointer_down_outside: None,
            on_focus_outside: None,
            disable_pointer_down_outside_dismiss: false,
            branches: Vec::new(),
            on_mount_auto_focus: None,
            on_unmount_auto_focus: None,
            on_error: None,
        }
    }
}

#[derive(Clone)]
/// Handle returned by [`use_dialog_layer`].
pub struct DialogLayerBinding<E>
where
    E: html::ElementType,
{
    node_ref: NodeRef<E>,
    presence: PresenceBinding<E>,
}

impl<E> DialogLayerBinding<E>
where
    E: html::ElementType,
{
    /// Returns the [`NodeRef`] that should be attached to the dialog content element.
    pub fn node_ref(&self) -> NodeRef<E> {
        self.node_ref
    }

    /// Returns `true` while the dialog surface should be rendered.
    pub fn is_rendered(&self) -> bool {
        self.presence.is_rendered()
    }

    /// Returns the canonical data-state value for the attached dialog element.
    pub fn data_state(&self) -> &'static str {
        self.presence.data_state()
    }

    /// Returns the transition-end handler for the dialog element.
    pub fn transition_end_handler(&self) -> Callback<leptos::ev::TransitionEvent> {
        self.presence.transition_end_handler()
    }

    /// Returns the animation-end handler for the dialog element.
    pub fn animation_end_handler(&self) -> Callback<leptos::ev::AnimationEvent> {
        self.presence.animation_end_handler()
    }
}

#[cfg(not(target_arch = "wasm32"))]
/// Creates a wrapper-free dialog layer binding.
pub fn use_dialog_layer<E>(options: DialogLayerOptions) -> DialogLayerBinding<E>
where
    E: html::ElementType,
    E::Output: Clone + 'static,
{
    use_dialog_layer_with_node_ref(NodeRef::<E>::new(), options)
}

#[cfg(not(target_arch = "wasm32"))]
/// Creates a wrapper-free dialog layer binding from an existing [`NodeRef`].
pub fn use_dialog_layer_with_node_ref<E>(
    node_ref: NodeRef<E>,
    options: DialogLayerOptions,
) -> DialogLayerBinding<E>
where
    E: html::ElementType,
    E::Output: Clone + 'static,
{
    let open = options.open;
    let presence = use_presence_with_node_ref(node_ref, open, None);
    attach_dialog_modal_layer(node_ref, open, options.modal, options.on_error);
    let on_dismiss = options.on_dismiss.map(|callback| {
        Callback::new(move |reason| {
            if open.get_untracked() {
                callback.run(reason);
            }
        })
    });
    let _ = use_focus_scope_with_node_ref(
        node_ref,
        FocusScopeOptions {
            active: Some(open),
            trapped: options.modal,
            auto_focus: true,
            return_focus: true,
            on_mount_auto_focus: options.on_mount_auto_focus,
            on_unmount_auto_focus: options.on_unmount_auto_focus,
        },
    );
    let _ = use_dismissible_layer_with_node_ref(
        node_ref,
        DismissibleLayerOptions {
            active: Some(open),
            on_dismiss,
            on_escape_key_down: options.on_escape_key_down,
            on_pointer_down_outside: options.on_pointer_down_outside,
            on_focus_outside: options.on_focus_outside,
            disable_pointer_down_outside_dismiss: options.disable_pointer_down_outside_dismiss,
            branches: options.branches,
        },
    );
    DialogLayerBinding { node_ref, presence }
}

#[cfg(target_arch = "wasm32")]
/// Creates a wrapper-free dialog layer binding.
pub fn use_dialog_layer<E>(options: DialogLayerOptions) -> DialogLayerBinding<E>
where
    E: html::ElementType,
    E::Output: wasm_bindgen::JsCast + Clone + 'static,
{
    use_dialog_layer_with_node_ref(NodeRef::<E>::new(), options)
}

#[cfg(target_arch = "wasm32")]
/// Creates a wrapper-free dialog layer binding from an existing [`NodeRef`].
pub fn use_dialog_layer_with_node_ref<E>(
    node_ref: NodeRef<E>,
    options: DialogLayerOptions,
) -> DialogLayerBinding<E>
where
    E: html::ElementType,
    E::Output: wasm_bindgen::JsCast + Clone + 'static,
{
    let open = options.open;
    let presence = use_presence_with_node_ref(node_ref, open, None);
    attach_dialog_modal_layer(node_ref, open, options.modal, options.on_error);
    let on_dismiss = options.on_dismiss.map(|callback| {
        Callback::new(move |reason| {
            if open.get_untracked() {
                callback.run(reason);
            }
        })
    });
    let _ = use_focus_scope_with_node_ref(
        node_ref,
        FocusScopeOptions {
            active: Some(open),
            trapped: options.modal,
            auto_focus: true,
            return_focus: true,
            on_mount_auto_focus: options.on_mount_auto_focus,
            on_unmount_auto_focus: options.on_unmount_auto_focus,
        },
    );
    let _ = use_dismissible_layer_with_node_ref(
        node_ref,
        DismissibleLayerOptions {
            active: Some(open),
            on_dismiss,
            on_escape_key_down: options.on_escape_key_down,
            on_pointer_down_outside: options.on_pointer_down_outside,
            on_focus_outside: options.on_focus_outside,
            disable_pointer_down_outside_dismiss: options.disable_pointer_down_outside_dismiss,
            branches: options.branches,
        },
    );
    DialogLayerBinding { node_ref, presence }
}

#[cfg(target_arch = "wasm32")]
struct DialogLayerGuards {
    _modal: Option<crate::ModalGuard>,
    _scroll: Option<crate::ScrollLockGuard>,
}

#[cfg(target_arch = "wasm32")]
fn attach_dialog_modal_layer<E>(
    node_ref: NodeRef<E>,
    open: Signal<bool>,
    modal: bool,
    on_error: Option<Callback<DialogLayerError>>,
) where
    E: html::ElementType,
    E::Output: wasm_bindgen::JsCast + Clone + 'static,
{
    use wasm_bindgen::JsCast;

    use crate::{modal_hide_siblings, scroll_lock_acquire};

    let guards = Arc::new(Mutex::new(None::<DialogLayerGuards>));
    let effect_guards = Arc::clone(&guards);
    let effect = RenderEffect::new(move |_| {
        if !modal || !open.get() {
            effect_guards
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .take();
            return;
        }
        if effect_guards
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .is_some()
        {
            return;
        }
        let Some(root) = node_ref
            .get()
            .and_then(|root| root.dyn_into::<web_sys::Element>().ok())
        else {
            return;
        };

        let scroll = match scroll_lock_acquire() {
            Ok(guard) => Some(guard),
            Err(error) => {
                if let Some(callback) = on_error.as_ref() {
                    callback.run(DialogLayerError::ScrollLock(error));
                }
                None
            }
        };
        let modal_guard = match modal_hide_siblings(&root) {
            Ok(guard) => Some(guard),
            Err(error) => {
                if let Some(callback) = on_error.as_ref() {
                    callback.run(DialogLayerError::Modal(error));
                }
                None
            }
        };

        if modal_guard.is_some() || scroll.is_some() {
            *effect_guards
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(DialogLayerGuards {
                _modal: modal_guard,
                _scroll: scroll,
            });
        }
    });

    on_cleanup(move || {
        drop(effect);
        guards
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take();
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn attach_dialog_modal_layer<E>(
    node_ref: NodeRef<E>,
    open: Signal<bool>,
    modal: bool,
    on_error: Option<Callback<DialogLayerError>>,
) where
    E: html::ElementType,
{
    let _ = node_ref;
    let _ = open;
    let _ = modal;
    let _ = on_error;
}
