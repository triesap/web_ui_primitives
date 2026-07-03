//! Menu behavior composition for accessible overlays.

use leptos::html;
use leptos::prelude::*;
#[cfg(target_arch = "wasm32")]
use std::rc::Rc;
use web_ui_primitives_core::{PlacementAlign, PlacementSide};
#[cfg(target_arch = "wasm32")]
use web_ui_primitives_core::{PlacementOptions, PlacementRect, PlacementSize, place_layer};

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

#[derive(Clone)]
/// Options for [`use_menu_placement_with_node_refs`].
pub struct MenuPlacementOptions {
    pub open: Signal<bool>,
    pub side: PlacementSide,
    pub align: PlacementAlign,
    pub spacing: f64,
    pub viewport_padding: f64,
}

impl MenuPlacementOptions {
    /// Creates menu placement options for an open signal.
    pub fn new(open: impl Into<Signal<bool>>, side: PlacementSide, align: PlacementAlign) -> Self {
        Self {
            open: open.into(),
            side,
            align,
            spacing: 4.0,
            viewport_padding: 8.0,
        }
    }

    /// Sets the gap between the trigger and menu content.
    pub fn spacing(mut self, spacing: f64) -> Self {
        self.spacing = spacing;
        self
    }

    /// Sets the viewport padding used while flipping and shifting content.
    pub fn viewport_padding(mut self, viewport_padding: f64) -> Self {
        self.viewport_padding = viewport_padding;
        self
    }
}

#[derive(Clone)]
/// Handle returned by [`use_menu_placement_with_node_refs`].
pub struct MenuPlacementBinding {
    style: RwSignal<String>,
    side: RwSignal<PlacementSide>,
    align: PlacementAlign,
}

impl MenuPlacementBinding {
    /// Returns inline CSS positioning for the menu content.
    pub fn style(&self) -> String {
        self.style.get()
    }

    /// Returns the resolved placement side.
    pub fn side(&self) -> PlacementSide {
        self.side.get()
    }

    /// Returns the requested placement alignment.
    pub fn align(&self) -> PlacementAlign {
        self.align
    }

    /// Returns the resolved placement side as a stable data attribute value.
    pub fn data_side(&self) -> &'static str {
        placement_side_data_value(self.side())
    }

    /// Returns the requested placement alignment as a stable data attribute value.
    pub fn data_align(&self) -> &'static str {
        placement_align_data_value(self.align())
    }
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

#[cfg(not(target_arch = "wasm32"))]
/// Creates a menu placement binding from trigger and content [`NodeRef`] values.
pub fn use_menu_placement_with_node_refs<T, C>(
    _trigger_ref: NodeRef<T>,
    _content_ref: NodeRef<C>,
    options: MenuPlacementOptions,
) -> MenuPlacementBinding
where
    T: html::ElementType,
    T::Output: Clone + 'static,
    C: html::ElementType,
    C::Output: Clone + 'static,
{
    MenuPlacementBinding {
        style: RwSignal::new(String::new()),
        side: RwSignal::new(options.side),
        align: options.align,
    }
}

#[cfg(target_arch = "wasm32")]
/// Creates a menu placement binding from trigger and content [`NodeRef`] values.
pub fn use_menu_placement_with_node_refs<T, C>(
    trigger_ref: NodeRef<T>,
    content_ref: NodeRef<C>,
    options: MenuPlacementOptions,
) -> MenuPlacementBinding
where
    T: html::ElementType,
    T::Output: wasm_bindgen::JsCast + Clone + 'static,
    C: html::ElementType,
    C::Output: wasm_bindgen::JsCast + Clone + 'static,
{
    use send_wrapper::SendWrapper;
    use wasm_bindgen::JsCast;
    use wasm_bindgen::closure::Closure;

    let binding = MenuPlacementBinding {
        style: RwSignal::new(String::new()),
        side: RwSignal::new(options.side),
        align: options.align,
    };
    let style = binding.style;
    let side = binding.side;
    let align = binding.align;

    content_ref.on_load(move |content| {
        let Ok(content) = content.dyn_into::<web_sys::Element>() else {
            return;
        };
        let trigger_ref = trigger_ref;
        let options = options.clone();
        let content = SendWrapper::new(content);
        let update_options = options.clone();
        let update = Rc::new(move || {
            update_menu_placement(
                trigger_ref,
                content.clone(),
                update_options.clone(),
                style,
                side,
                align,
            );
        });

        let update_for_effect = Rc::clone(&update);
        let effect_options = options.clone();
        Effect::new(move || {
            let _ = effect_options.open.get();
            update_for_effect();
        });

        let Some(window) = web_sys::window() else {
            return;
        };
        let resize_update = Rc::clone(&update);
        let resize = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            resize_update();
        }) as Box<dyn FnMut(_)>);
        let scroll_update = Rc::clone(&update);
        let scroll = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            scroll_update();
        }) as Box<dyn FnMut(_)>);
        let _ = window.add_event_listener_with_callback("resize", resize.as_ref().unchecked_ref());
        let _ = window.add_event_listener_with_callback("scroll", scroll.as_ref().unchecked_ref());
        let cleanup_window = SendWrapper::new(window);
        let cleanup_resize = SendWrapper::new(resize);
        let cleanup_scroll = SendWrapper::new(scroll);

        on_cleanup(move || {
            let window = cleanup_window.take();
            let resize = cleanup_resize.take();
            let scroll = cleanup_scroll.take();
            let _ = window
                .remove_event_listener_with_callback("resize", resize.as_ref().unchecked_ref());
            let _ = window
                .remove_event_listener_with_callback("scroll", scroll.as_ref().unchecked_ref());
        });
    });

    binding
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

pub fn placement_side_data_value(side: PlacementSide) -> &'static str {
    match side {
        PlacementSide::Top => "top",
        PlacementSide::Right => "right",
        PlacementSide::Bottom => "bottom",
        PlacementSide::Left => "left",
    }
}

pub fn placement_align_data_value(align: PlacementAlign) -> &'static str {
    match align {
        PlacementAlign::Start => "start",
        PlacementAlign::Center => "center",
        PlacementAlign::End => "end",
    }
}

#[cfg(target_arch = "wasm32")]
fn update_menu_placement<T>(
    trigger_ref: NodeRef<T>,
    content: send_wrapper::SendWrapper<web_sys::Element>,
    options: MenuPlacementOptions,
    style: RwSignal<String>,
    side: RwSignal<PlacementSide>,
    align: PlacementAlign,
) where
    T: html::ElementType,
    T::Output: wasm_bindgen::JsCast + Clone + 'static,
{
    use wasm_bindgen::JsCast;

    if !options.open.get_untracked() {
        style.set(String::new());
        side.set(options.side);
        return;
    }

    let Some(trigger) = trigger_ref
        .get()
        .and_then(|trigger| trigger.dyn_into::<web_sys::Element>().ok())
    else {
        return;
    };
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(viewport_width) = window.inner_width().ok().and_then(|value| value.as_f64()) else {
        return;
    };
    let Some(viewport_height) = window.inner_height().ok().and_then(|value| value.as_f64()) else {
        return;
    };

    let trigger_rect = trigger.get_bounding_client_rect();
    let content_rect = content.get_bounding_client_rect();
    let placement = place_layer(
        PlacementRect {
            x: trigger_rect.x(),
            y: trigger_rect.y(),
            width: trigger_rect.width(),
            height: trigger_rect.height(),
        },
        PlacementSize {
            width: content_rect.width(),
            height: content_rect.height(),
        },
        PlacementSize {
            width: viewport_width,
            height: viewport_height,
        },
        PlacementOptions::new(options.side, align)
            .spacing(options.spacing)
            .viewport_padding(options.viewport_padding),
    );

    style.set(format!(
        "left:{:.3}px;top:{:.3}px;max-width:{:.3}px;max-height:{:.3}px;",
        placement.x, placement.y, placement.max_width, placement.max_height,
    ));
    side.set(placement.side);
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
