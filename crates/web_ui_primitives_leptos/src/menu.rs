//! Menu behavior composition for accessible overlays.

use leptos::html;
use leptos::prelude::*;

use crate::{
    DismissibleBranch, DismissibleEscapeKeyDownEvent, DismissibleFocusOutsideEvent,
    DismissibleLayerOptions, DismissiblePointerDownOutsideEvent, DismissibleReason,
    FocusScopeOptions, PresenceBinding, use_dismissible_layer_with_node_ref,
    use_focus_scope_with_node_ref, use_presence_with_node_ref,
};

#[derive(Clone)]
/// Options for [`use_menu_layer`].
pub struct MenuLayerOptions {
    pub open: Signal<bool>,
    pub on_dismiss: Option<Callback<DismissibleReason>>,
    pub on_escape_key_down: Option<Callback<DismissibleEscapeKeyDownEvent>>,
    pub on_pointer_down_outside: Option<Callback<DismissiblePointerDownOutsideEvent>>,
    pub on_focus_outside: Option<Callback<DismissibleFocusOutsideEvent>>,
    pub disable_pointer_down_outside_dismiss: bool,
    pub branches: Vec<DismissibleBranch>,
    pub auto_focus: bool,
    pub return_focus: bool,
    pub on_mount_auto_focus: Option<Callback<()>>,
    pub on_unmount_auto_focus: Option<Callback<()>>,
}

impl MenuLayerOptions {
    /// Creates menu layer options for an open signal.
    pub fn new(open: impl Into<Signal<bool>>) -> Self {
        Self {
            open: open.into(),
            on_dismiss: None,
            on_escape_key_down: None,
            on_pointer_down_outside: None,
            on_focus_outside: None,
            disable_pointer_down_outside_dismiss: false,
            branches: Vec::new(),
            auto_focus: true,
            return_focus: true,
            on_mount_auto_focus: None,
            on_unmount_auto_focus: None,
        }
    }
}

#[derive(Clone)]
/// Handle returned by [`use_menu_layer`].
pub struct MenuLayerBinding<E>
where
    E: html::ElementType,
{
    node_ref: NodeRef<E>,
    presence: PresenceBinding<E>,
}

impl<E> MenuLayerBinding<E>
where
    E: html::ElementType,
{
    /// Returns the [`NodeRef`] that should be attached to the menu content element.
    pub fn node_ref(&self) -> NodeRef<E> {
        self.node_ref
    }

    /// Returns `true` while the menu surface should be rendered.
    pub fn is_rendered(&self) -> bool {
        self.presence.is_rendered()
    }

    /// Returns the canonical data-state value for the attached menu element.
    pub fn data_state(&self) -> &'static str {
        self.presence.data_state()
    }

    /// Returns the transition-end handler for the menu element.
    pub fn transition_end_handler(&self) -> Callback<leptos::ev::TransitionEvent> {
        self.presence.transition_end_handler()
    }

    /// Returns the animation-end handler for the menu element.
    pub fn animation_end_handler(&self) -> Callback<leptos::ev::AnimationEvent> {
        self.presence.animation_end_handler()
    }
}

#[cfg(not(target_arch = "wasm32"))]
/// Creates a wrapper-free menu layer binding.
pub fn use_menu_layer<E>(options: MenuLayerOptions) -> MenuLayerBinding<E>
where
    E: html::ElementType,
    E::Output: Clone + 'static,
{
    use_menu_layer_with_node_ref(NodeRef::<E>::new(), options)
}

#[cfg(not(target_arch = "wasm32"))]
/// Creates a wrapper-free menu layer binding from an existing [`NodeRef`].
pub fn use_menu_layer_with_node_ref<E>(
    node_ref: NodeRef<E>,
    options: MenuLayerOptions,
) -> MenuLayerBinding<E>
where
    E: html::ElementType,
    E::Output: Clone + 'static,
{
    attach_menu_layer(node_ref, options)
}

#[cfg(target_arch = "wasm32")]
/// Creates a wrapper-free menu layer binding.
pub fn use_menu_layer<E>(options: MenuLayerOptions) -> MenuLayerBinding<E>
where
    E: html::ElementType,
    E::Output: wasm_bindgen::JsCast + Clone + 'static,
{
    use_menu_layer_with_node_ref(NodeRef::<E>::new(), options)
}

#[cfg(target_arch = "wasm32")]
/// Creates a wrapper-free menu layer binding from an existing [`NodeRef`].
pub fn use_menu_layer_with_node_ref<E>(
    node_ref: NodeRef<E>,
    options: MenuLayerOptions,
) -> MenuLayerBinding<E>
where
    E: html::ElementType,
    E::Output: wasm_bindgen::JsCast + Clone + 'static,
{
    attach_menu_layer(node_ref, options)
}

#[cfg(not(target_arch = "wasm32"))]
fn attach_menu_layer<E>(node_ref: NodeRef<E>, options: MenuLayerOptions) -> MenuLayerBinding<E>
where
    E: html::ElementType,
    E::Output: Clone + 'static,
{
    let open = options.open;
    let presence = use_presence_with_node_ref(node_ref, open, None);
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
            trapped: false,
            auto_focus: options.auto_focus,
            return_focus: options.return_focus,
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
    MenuLayerBinding { node_ref, presence }
}

#[cfg(target_arch = "wasm32")]
fn attach_menu_layer<E>(node_ref: NodeRef<E>, options: MenuLayerOptions) -> MenuLayerBinding<E>
where
    E: html::ElementType,
    E::Output: wasm_bindgen::JsCast + Clone + 'static,
{
    let open = options.open;
    let presence = use_presence_with_node_ref(node_ref, open, None);
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
            trapped: false,
            auto_focus: options.auto_focus,
            return_focus: options.return_focus,
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
    MenuLayerBinding { node_ref, presence }
}
