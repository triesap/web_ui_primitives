//! Menu behavior composition for accessible overlays.

use leptos::html;
use leptos::prelude::*;
#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
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

/// Attribute that keys a content element to its strict placement rule.
pub const STRICT_PLACEMENT_ATTRIBUTE: &str = "data-web-ui-placement-id";
/// Attribute that identifies the owned strict placement stylesheet.
pub const STRICT_PLACEMENT_STYLESHEET_ATTRIBUTE: &str = "data-web-ui-placement-stylesheet";

/// Error returned while configuring or publishing a strict placement sink.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlacementSinkError {
    InvalidId,
    InvalidNonce,
    MissingAuthorization,
    DocumentUnavailable,
    HeadUnavailable,
    StylesheetUnavailable,
    DuplicateId,
}

/// Validated stable component ID used by the strict placement adapter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlacementStyleId(String);

impl PlacementStyleId {
    pub fn new(value: impl Into<String>) -> Result<Self, PlacementSinkError> {
        let value = value.into();
        let mut chars = value.chars();
        let valid = value.len() <= 128
            && chars
                .next()
                .is_some_and(|character| character.is_ascii_alphanumeric())
            && chars.all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '-' | '_')
            });
        valid
            .then_some(Self(value))
            .ok_or(PlacementSinkError::InvalidId)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Validated CSP nonce applied to an owned strict placement stylesheet.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlacementStyleNonce(String);

impl PlacementStyleNonce {
    pub fn new(value: impl Into<String>) -> Result<Self, PlacementSinkError> {
        let value = value.into();
        let valid = !value.is_empty()
            && value.len() <= 512
            && value.chars().all(|character| {
                character.is_ascii_alphanumeric()
                    || matches!(character, '+' | '/' | '-' | '_' | '=')
            });
        valid
            .then_some(Self(value))
            .ok_or(PlacementSinkError::InvalidNonce)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Strict stylesheet configuration for one stable component ID.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StrictPlacementSink {
    id: PlacementStyleId,
    nonce: Option<PlacementStyleNonce>,
}

impl StrictPlacementSink {
    pub fn new(id: PlacementStyleId) -> Self {
        Self { id, nonce: None }
    }

    pub fn authorized(mut self, nonce: PlacementStyleNonce) -> Self {
        self.nonce = Some(nonce);
        self
    }

    pub fn id(&self) -> &PlacementStyleId {
        &self.id
    }

    pub fn nonce(&self) -> Option<&PlacementStyleNonce> {
        self.nonce.as_ref()
    }
}

/// Publication adapter selected for dynamic placement coordinates.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum PlacementSink {
    #[default]
    InlineStyle,
    StrictStylesheet(StrictPlacementSink),
}

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
    pub sink: PlacementSink,
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
            sink: PlacementSink::InlineStyle,
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

    /// Selects the publication adapter for the computed placement.
    pub fn sink(mut self, sink: PlacementSink) -> Self {
        self.sink = sink;
        self
    }
}

#[derive(Clone)]
/// Handle returned by [`use_menu_placement_with_node_refs`].
pub struct MenuPlacementBinding {
    style: RwSignal<String>,
    side: RwSignal<PlacementSide>,
    align: PlacementAlign,
    sink: PlacementSink,
    error: RwSignal<Option<PlacementSinkError>>,
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone)]
struct MenuPlacementPublication {
    style: RwSignal<String>,
    side: RwSignal<PlacementSide>,
    align: PlacementAlign,
    strict_state: Rc<RefCell<Option<StrictPlacementDomState>>>,
    error: RwSignal<Option<PlacementSinkError>>,
}

#[cfg_attr(not(any(test, target_arch = "wasm32")), allow(dead_code))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MenuPlacementUpdateAction {
    Clear,
    Retain,
    AnimationFrame,
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

    /// Returns the stable strict-placement ID, when that adapter is selected.
    pub fn strict_id(&self) -> Option<&PlacementStyleId> {
        match &self.sink {
            PlacementSink::InlineStyle => None,
            PlacementSink::StrictStylesheet(sink) => Some(sink.id()),
        }
    }

    /// Returns the latest strict placement publication error.
    pub fn error(&self) -> Option<PlacementSinkError> {
        self.error.get()
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

#[cfg_attr(not(any(test, target_arch = "wasm32")), allow(dead_code))]
fn menu_placement_update_action(
    open: bool,
    trigger_loaded: bool,
    content_loaded: bool,
) -> MenuPlacementUpdateAction {
    if !content_loaded {
        return MenuPlacementUpdateAction::Clear;
    }

    if open && trigger_loaded {
        return MenuPlacementUpdateAction::AnimationFrame;
    }

    MenuPlacementUpdateAction::Retain
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
    let sink = options.sink;
    MenuPlacementBinding {
        style: RwSignal::new(String::new()),
        side: RwSignal::new(options.side),
        align: options.align,
        sink,
        error: RwSignal::new(None),
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
    use wasm_bindgen::JsCast;
    use wasm_bindgen::closure::Closure;

    let sink = options.sink.clone();
    let binding = MenuPlacementBinding {
        style: RwSignal::new(String::new()),
        side: RwSignal::new(options.side),
        align: options.align,
        sink: sink.clone(),
        error: RwSignal::new(None),
    };
    let strict_state = Rc::new(RefCell::new(None::<StrictPlacementDomState>));
    let publication = MenuPlacementPublication {
        style: binding.style,
        side: binding.side,
        align: binding.align,
        strict_state: Rc::clone(&strict_state),
        error: binding.error,
    };
    let update_options = options.clone();
    let update_publication = publication.clone();
    let update: Rc<dyn Fn()> = Rc::new(move || {
        update_menu_placement(
            trigger_ref,
            content_ref,
            update_options.clone(),
            update_publication.clone(),
        );
    });

    let update_for_effect = Rc::clone(&update);
    let effect_options = options.clone();
    let effect_trigger_ref = trigger_ref;
    let effect_content_ref = content_ref;
    let effect_strict_state = Rc::clone(&strict_state);
    Effect::new(move || {
        let open = effect_options.open.get();
        let trigger_loaded = effect_trigger_ref.get().is_some();
        let content_loaded = effect_content_ref.get().is_some();

        match menu_placement_update_action(open, trigger_loaded, content_loaded) {
            MenuPlacementUpdateAction::Clear => {
                clear_menu_placement(
                    publication.style,
                    publication.side,
                    effect_options.side,
                    &effect_strict_state,
                    publication.error,
                );
            }
            MenuPlacementUpdateAction::Retain => {}
            MenuPlacementUpdateAction::AnimationFrame => {
                request_menu_placement_update(Rc::clone(&update_for_effect));
            }
        }
    });

    let Some(window) = web_sys::window() else {
        return binding;
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
    let cleanup_window = send_wrapper::SendWrapper::new(window);
    let cleanup_resize = send_wrapper::SendWrapper::new(resize);
    let cleanup_scroll = send_wrapper::SendWrapper::new(scroll);
    let cleanup_strict_state = send_wrapper::SendWrapper::new(Rc::clone(&strict_state));

    on_cleanup(move || {
        let window = cleanup_window.take();
        let resize = cleanup_resize.take();
        let scroll = cleanup_scroll.take();
        let _ =
            window.remove_event_listener_with_callback("resize", resize.as_ref().unchecked_ref());
        let _ =
            window.remove_event_listener_with_callback("scroll", scroll.as_ref().unchecked_ref());
        clear_strict_placement(&cleanup_strict_state.take());
    });

    binding
}

#[cfg(target_arch = "wasm32")]
fn request_menu_placement_update(update: Rc<dyn Fn()>) {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::closure::Closure;

    let Some(window) = web_sys::window() else {
        update();
        return;
    };
    let fallback_update = Rc::clone(&update);
    let callback = Closure::once_into_js(move || {
        update();
    });
    if window
        .request_animation_frame(callback.unchecked_ref())
        .is_err()
    {
        fallback_update();
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

    /// Returns the transition-cancel handler for the menu content element.
    pub fn transition_cancel_handler(&self) -> Callback<leptos::ev::TransitionEvent> {
        self.presence.transition_cancel_handler()
    }

    /// Returns the animation-end handler for the menu element.
    pub fn animation_end_handler(&self) -> Callback<leptos::ev::AnimationEvent> {
        self.presence.animation_end_handler()
    }

    /// Returns the animation-cancel handler for the menu content element.
    pub fn animation_cancel_handler(&self) -> Callback<leptos::ev::AnimationEvent> {
        self.presence.animation_cancel_handler()
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
struct StrictPlacementDomState {
    id: PlacementStyleId,
    content: web_sys::Element,
    stylesheet: web_sys::Element,
}

#[cfg(target_arch = "wasm32")]
fn clear_menu_placement(
    style: RwSignal<String>,
    side: RwSignal<PlacementSide>,
    requested_side: PlacementSide,
    strict_state: &Rc<RefCell<Option<StrictPlacementDomState>>>,
    error: RwSignal<Option<PlacementSinkError>>,
) {
    style.set(String::new());
    side.set(requested_side);
    error.set(None);
    clear_strict_placement(strict_state);
}

#[cfg(target_arch = "wasm32")]
fn update_menu_placement<T, C>(
    trigger_ref: NodeRef<T>,
    content_ref: NodeRef<C>,
    options: MenuPlacementOptions,
    publication: MenuPlacementPublication,
) where
    T: html::ElementType,
    T::Output: wasm_bindgen::JsCast + Clone + 'static,
    C: html::ElementType,
    C::Output: wasm_bindgen::JsCast + Clone + 'static,
{
    use wasm_bindgen::JsCast;

    if !options.open.get_untracked() {
        return;
    }

    let Some(trigger) = trigger_ref
        .get_untracked()
        .and_then(|trigger| trigger.dyn_into::<web_sys::Element>().ok())
    else {
        return;
    };
    let Some(content) = content_ref
        .get_untracked()
        .and_then(|content| content.dyn_into::<web_sys::Element>().ok())
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
        PlacementOptions::new(options.side, publication.align)
            .spacing(options.spacing)
            .viewport_padding(options.viewport_padding),
    );

    match &options.sink {
        PlacementSink::InlineStyle => {
            clear_strict_placement(&publication.strict_state);
            publication.error.set(None);
            publication.style.set(format!(
                "left:{:.3}px;top:{:.3}px;max-width:{:.3}px;max-height:{:.3}px;",
                placement.x, placement.y, placement.max_width, placement.max_height,
            ));
        }
        PlacementSink::StrictStylesheet(sink) => {
            publication.style.set(String::new());
            if let Err(sink_error) =
                publish_strict_placement(&content, sink, placement, &publication.strict_state)
            {
                clear_strict_placement(&publication.strict_state);
                publication.error.set(Some(sink_error));
                return;
            }
            publication.error.set(None);
        }
    }
    publication.side.set(placement.side);
}

#[cfg(target_arch = "wasm32")]
fn publish_strict_placement(
    content: &web_sys::Element,
    sink: &StrictPlacementSink,
    placement: web_ui_primitives_core::Placement,
    state: &Rc<RefCell<Option<StrictPlacementDomState>>>,
) -> Result<(), PlacementSinkError> {
    let nonce = sink
        .nonce()
        .ok_or(PlacementSinkError::MissingAuthorization)?;
    let document = web_sys::window()
        .and_then(|window| window.document())
        .ok_or(PlacementSinkError::DocumentUnavailable)?;
    let head = document
        .query_selector("head")
        .map_err(|_| PlacementSinkError::HeadUnavailable)?
        .ok_or(PlacementSinkError::HeadUnavailable)?;

    let mut state = state.borrow_mut();
    if state.is_none() {
        let selector = format!(
            "style[{STRICT_PLACEMENT_STYLESHEET_ATTRIBUTE}=\"{}\"]",
            sink.id().as_str()
        );
        if document
            .query_selector(&selector)
            .map_err(|_| PlacementSinkError::StylesheetUnavailable)?
            .is_some()
        {
            return Err(PlacementSinkError::DuplicateId);
        }
        let stylesheet = document
            .create_element("style")
            .map_err(|_| PlacementSinkError::StylesheetUnavailable)?;
        stylesheet
            .set_attribute(STRICT_PLACEMENT_STYLESHEET_ATTRIBUTE, sink.id().as_str())
            .map_err(|_| PlacementSinkError::StylesheetUnavailable)?;
        stylesheet
            .set_attribute("nonce", nonce.as_str())
            .map_err(|_| PlacementSinkError::StylesheetUnavailable)?;
        head.append_child(&stylesheet)
            .map_err(|_| PlacementSinkError::StylesheetUnavailable)?;
        *state = Some(StrictPlacementDomState {
            id: sink.id().clone(),
            content: content.clone(),
            stylesheet,
        });
    }

    let Some(state) = state.as_mut() else {
        return Err(PlacementSinkError::StylesheetUnavailable);
    };
    if state.id != *sink.id() {
        return Err(PlacementSinkError::DuplicateId);
    }
    if state.content != *content {
        remove_owned_placement_attribute(&state.content, &state.id);
        state.content = content.clone();
    }
    content
        .set_attribute(STRICT_PLACEMENT_ATTRIBUTE, sink.id().as_str())
        .map_err(|_| PlacementSinkError::StylesheetUnavailable)?;
    state.stylesheet.set_text_content(Some(&format!(
        "[{STRICT_PLACEMENT_ATTRIBUTE}=\"{}\"]{{position:fixed;left:{:.3}px;top:{:.3}px;\
         max-width:{:.3}px;max-height:{:.3}px;}}",
        sink.id().as_str(),
        placement.x,
        placement.y,
        placement.max_width,
        placement.max_height,
    )));
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn clear_strict_placement(state: &Rc<RefCell<Option<StrictPlacementDomState>>>) {
    let Some(state) = state.borrow_mut().take() else {
        return;
    };
    remove_owned_placement_attribute(&state.content, &state.id);
    if let Some(parent) = state.stylesheet.parent_node() {
        let _ = parent.remove_child(&state.stylesheet);
    }
}

#[cfg(target_arch = "wasm32")]
fn remove_owned_placement_attribute(content: &web_sys::Element, id: &PlacementStyleId) {
    if content.get_attribute(STRICT_PLACEMENT_ATTRIBUTE).as_deref() == Some(id.as_str()) {
        let _ = content.remove_attribute(STRICT_PLACEMENT_ATTRIBUTE);
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

#[cfg(test)]
mod tests {
    use super::{
        MenuPlacementUpdateAction, PlacementSinkError, PlacementStyleId, PlacementStyleNonce,
        StrictPlacementSink, menu_placement_update_action,
    };

    #[test]
    fn menu_placement_update_action_tracks_open_and_loaded_refs() {
        assert_eq!(
            menu_placement_update_action(false, false, false),
            MenuPlacementUpdateAction::Clear
        );
        assert_eq!(
            menu_placement_update_action(false, false, true),
            MenuPlacementUpdateAction::Retain
        );
        assert_eq!(
            menu_placement_update_action(false, true, true),
            MenuPlacementUpdateAction::Retain
        );
        assert_eq!(
            menu_placement_update_action(true, false, false),
            MenuPlacementUpdateAction::Clear
        );
        assert_eq!(
            menu_placement_update_action(true, true, false),
            MenuPlacementUpdateAction::Clear
        );
        assert_eq!(
            menu_placement_update_action(true, false, true),
            MenuPlacementUpdateAction::Retain
        );
        assert_eq!(
            menu_placement_update_action(true, true, true),
            MenuPlacementUpdateAction::AnimationFrame
        );
    }

    #[test]
    fn strict_placement_identifiers_reject_selector_fragments() {
        assert_eq!(
            PlacementStyleId::new("menu-account_1")
                .expect("stable ID")
                .as_str(),
            "menu-account_1"
        );
        for invalid in [
            "",
            "-leading",
            "menu account",
            "menu\"]{display:none}",
            "menu/account",
        ] {
            assert_eq!(
                PlacementStyleId::new(invalid),
                Err(PlacementSinkError::InvalidId)
            );
        }
    }

    #[test]
    fn strict_placement_nonces_reject_csp_fragments() {
        assert_eq!(
            PlacementStyleNonce::new("abcDEF012-_+/=")
                .expect("nonce")
                .as_str(),
            "abcDEF012-_+/="
        );
        for invalid in ["", "nonce value", "'unsafe-inline'", "nonce;style-src"] {
            assert_eq!(
                PlacementStyleNonce::new(invalid),
                Err(PlacementSinkError::InvalidNonce)
            );
        }
    }

    #[test]
    fn strict_placement_requires_explicit_authorization() {
        let id = PlacementStyleId::new("menu-account").expect("stable ID");
        let unauthorized = StrictPlacementSink::new(id.clone());
        assert_eq!(unauthorized.id(), &id);
        assert!(unauthorized.nonce().is_none());

        let nonce = PlacementStyleNonce::new("nonce123").expect("nonce");
        let authorized = unauthorized.authorized(nonce.clone());
        assert_eq!(authorized.nonce(), Some(&nonce));
    }
}
