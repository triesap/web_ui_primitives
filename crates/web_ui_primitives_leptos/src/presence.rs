//! Presence primitive for exit-aware mounting and unmounting.

use leptos::ev::{AnimationEvent, TransitionEvent};
use leptos::html;
use leptos::prelude::*;
use std::sync::Arc;
#[cfg(target_arch = "wasm32")]
use std::sync::Mutex;

/// Lifecycle states used by [`Presence`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresenceState {
    Mounted,
    Exiting,
    Unmounted,
}

/// Computes the next presence state from visibility and exit-completion inputs.
pub fn presence_state_next(
    current: PresenceState,
    present: bool,
    exit_complete: bool,
) -> PresenceState {
    match (current, present, exit_complete) {
        (_, true, _) => PresenceState::Mounted,
        (PresenceState::Mounted, false, true) => PresenceState::Unmounted,
        (PresenceState::Mounted, false, false) => PresenceState::Exiting,
        (PresenceState::Exiting, false, true) => PresenceState::Unmounted,
        (PresenceState::Exiting, false, false) => PresenceState::Exiting,
        (PresenceState::Unmounted, false, _) => PresenceState::Unmounted,
    }
}

#[derive(Clone)]
/// Handle returned by [`use_presence`].
pub struct PresenceBinding<E>
where
    E: html::ElementType,
{
    node_ref: NodeRef<E>,
    state: RwSignal<PresenceState>,
    present: Signal<bool>,
    transition_end: Callback<TransitionEvent>,
    animation_end: Callback<AnimationEvent>,
}

impl<E> PresenceBinding<E>
where
    E: html::ElementType,
{
    /// Returns the [`NodeRef`] that should be attached to the present element.
    pub fn node_ref(&self) -> NodeRef<E> {
        self.node_ref
    }

    /// Returns the current presence state.
    pub fn state(&self) -> PresenceState {
        self.state.get()
    }

    /// Returns `true` while the element should be rendered.
    pub fn is_rendered(&self) -> bool {
        self.state() != PresenceState::Unmounted
    }

    /// Returns the canonical data-state value for the attached element.
    pub fn data_state(&self) -> &'static str {
        if self.present.get() { "open" } else { "closed" }
    }

    /// Returns the transition-end handler for the attached element.
    pub fn transition_end_handler(&self) -> Callback<TransitionEvent> {
        self.transition_end
    }

    /// Returns the animation-end handler for the attached element.
    pub fn animation_end_handler(&self) -> Callback<AnimationEvent> {
        self.animation_end
    }
}

#[cfg(target_arch = "wasm32")]
trait PresenceElementOutput: wasm_bindgen::JsCast + Clone + 'static {}

#[cfg(target_arch = "wasm32")]
impl<T> PresenceElementOutput for T where T: wasm_bindgen::JsCast + Clone + 'static {}

#[cfg(not(target_arch = "wasm32"))]
trait PresenceElementOutput: Clone + 'static {}

#[cfg(not(target_arch = "wasm32"))]
impl<T> PresenceElementOutput for T where T: Clone + 'static {}

fn presence_should_complete_exit(
    current: PresenceState,
    present: bool,
    event_target_matches_current_target: bool,
) -> bool {
    current == PresenceState::Exiting && !present && event_target_matches_current_target
}

fn presence_event_matches_current_target<T: PartialEq>(
    target: Option<T>,
    current_target: Option<T>,
) -> bool {
    matches!(
        (target, current_target),
        (Some(target), Some(current_target)) if target == current_target
    )
}

#[cfg_attr(not(any(test, target_arch = "wasm32")), allow(dead_code))]
fn presence_parse_time_millis(value: &str) -> Option<i64> {
    let value = value.trim();
    if let Some(value) = value.strip_suffix("ms") {
        value
            .trim()
            .parse::<f64>()
            .ok()
            .map(|time| time.round() as i64)
    } else if let Some(value) = value.strip_suffix('s') {
        value
            .trim()
            .parse::<f64>()
            .ok()
            .map(|time| (time * 1000.0).round() as i64)
    } else {
        None
    }
}

#[cfg_attr(not(any(test, target_arch = "wasm32")), allow(dead_code))]
fn presence_parse_time_list(value: &str) -> Vec<i64> {
    value
        .split(',')
        .filter_map(presence_parse_time_millis)
        .collect()
}

#[cfg_attr(not(any(test, target_arch = "wasm32")), allow(dead_code))]
fn presence_list_value(values: &[i64], index: usize) -> i64 {
    values
        .get(index)
        .copied()
        .or_else(|| values.last().copied())
        .unwrap_or(0)
}

#[cfg_attr(not(any(test, target_arch = "wasm32")), allow(dead_code))]
fn presence_max_total_time_millis(durations: &str, delays: &str) -> i64 {
    let durations = presence_parse_time_list(durations);
    let delays = presence_parse_time_list(delays);
    let count = durations.len().max(delays.len());
    let mut max_total = 0_i64;

    for index in 0..count {
        let duration = presence_list_value(&durations, index);
        if duration <= 0 {
            continue;
        }
        let delay = presence_list_value(&delays, index);
        max_total = max_total.max((duration + delay).max(0));
    }

    max_total
}

#[cfg_attr(not(any(test, target_arch = "wasm32")), allow(dead_code))]
fn presence_exit_timeout_ms_values(
    transition_duration: &str,
    transition_delay: &str,
    animation_duration: &str,
    animation_delay: &str,
) -> u32 {
    let max_total = presence_max_total_time_millis(transition_duration, transition_delay).max(
        presence_max_total_time_millis(animation_duration, animation_delay),
    );
    max_total.max(0) as u32
}

#[cfg(target_arch = "wasm32")]
struct PresenceExitTimeout {
    id: i32,
    _callback: send_wrapper::SendWrapper<wasm_bindgen::closure::Closure<dyn FnMut()>>,
}

#[cfg(target_arch = "wasm32")]
fn presence_clear_exit_timeout(exit_timeout: &Arc<Mutex<Option<PresenceExitTimeout>>>) {
    let mut exit_timeout = exit_timeout
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(timeout) = exit_timeout.take() {
        if let Some(window) = web_sys::window() {
            window.clear_timeout_with_handle(timeout.id);
        }
        drop(timeout);
    }
}

#[cfg(target_arch = "wasm32")]
fn presence_schedule_exit_timeout(
    exit_timeout: &Arc<Mutex<Option<PresenceExitTimeout>>>,
    delay_ms: u32,
    callback: Arc<dyn Fn() + Send + Sync>,
) {
    use wasm_bindgen::JsCast;

    presence_clear_exit_timeout(exit_timeout);

    let Some(window) = web_sys::window() else {
        callback();
        return;
    };

    let timeout_callback = {
        let callback = Arc::clone(&callback);
        wasm_bindgen::closure::Closure::wrap(Box::new(move || {
            callback();
        }) as Box<dyn FnMut()>)
    };

    let Ok(id) = window.set_timeout_with_callback_and_timeout_and_arguments_0(
        timeout_callback.as_ref().unchecked_ref(),
        delay_ms.min(i32::MAX as u32) as i32,
    ) else {
        callback();
        return;
    };

    *exit_timeout
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(PresenceExitTimeout {
        id,
        _callback: send_wrapper::SendWrapper::new(timeout_callback),
    });
}

#[cfg(target_arch = "wasm32")]
fn presence_exit_timeout_ms(root: &web_sys::Element) -> u32 {
    let Some(window) = web_sys::window() else {
        return 0;
    };
    let Ok(Some(style)) = window.get_computed_style(root) else {
        return 0;
    };

    let transition_duration = style
        .get_property_value("transition-duration")
        .unwrap_or_default();
    let transition_delay = style
        .get_property_value("transition-delay")
        .unwrap_or_default();
    let animation_duration = style
        .get_property_value("animation-duration")
        .unwrap_or_default();
    let animation_delay = style
        .get_property_value("animation-delay")
        .unwrap_or_default();

    presence_exit_timeout_ms_values(
        &transition_duration,
        &transition_delay,
        &animation_duration,
        &animation_delay,
    )
}

#[cfg(not(target_arch = "wasm32"))]
/// Creates a wrapper-free presence binding.
pub fn use_presence<E>(
    present: impl Into<Signal<bool>>,
    on_exit_complete: Option<Callback<()>>,
) -> PresenceBinding<E>
where
    E: html::ElementType,
    E::Output: Clone + 'static,
{
    use_presence_with_node_ref(NodeRef::<E>::new(), present, on_exit_complete)
}

#[cfg(not(target_arch = "wasm32"))]
/// Creates a wrapper-free presence binding from an existing [`NodeRef`].
pub fn use_presence_with_node_ref<E>(
    node_ref: NodeRef<E>,
    present: impl Into<Signal<bool>>,
    on_exit_complete: Option<Callback<()>>,
) -> PresenceBinding<E>
where
    E: html::ElementType,
    E::Output: Clone + 'static,
{
    create_presence_binding(node_ref, present.into(), on_exit_complete)
}

#[cfg(target_arch = "wasm32")]
/// Creates a wrapper-free presence binding.
pub fn use_presence<E>(
    present: impl Into<Signal<bool>>,
    on_exit_complete: Option<Callback<()>>,
) -> PresenceBinding<E>
where
    E: html::ElementType,
    E::Output: wasm_bindgen::JsCast + Clone + 'static,
{
    use_presence_with_node_ref(NodeRef::<E>::new(), present, on_exit_complete)
}

#[cfg(target_arch = "wasm32")]
/// Creates a wrapper-free presence binding from an existing [`NodeRef`].
pub fn use_presence_with_node_ref<E>(
    node_ref: NodeRef<E>,
    present: impl Into<Signal<bool>>,
    on_exit_complete: Option<Callback<()>>,
) -> PresenceBinding<E>
where
    E: html::ElementType,
    E::Output: wasm_bindgen::JsCast + Clone + 'static,
{
    create_presence_binding(node_ref, present.into(), on_exit_complete)
}

#[component]
/// Conditionally mounts content while allowing exit transitions to finish.
///
/// On wasm, this waits for matching `transitionend` or `animationend` events,
/// with a computed-style timeout fallback when exit motion is present.
///
/// On non-wasm targets, exit timing is treated as immediate, so content
/// unmounts as soon as `present` becomes `false`.
pub fn Presence(
    #[prop(into)] present: Signal<bool>,
    #[prop(optional)] on_exit_complete: Option<Callback<()>>,
    children: ChildrenFn,
) -> impl IntoView {
    let presence = use_presence::<html::Div>(present, on_exit_complete);

    view! {
        {move || -> AnyView {
            if !presence.is_rendered() {
                ().into_any()
            } else {
                let node_ref = presence.node_ref();
                let data_state = presence.clone();
                let transition_end = presence.transition_end_handler();
                let animation_end = presence.animation_end_handler();

                view! {
                    <div
                        node_ref=node_ref
                        data-state=move || data_state.data_state()
                        on:transitionend=move |event| transition_end.run(event)
                        on:animationend=move |event| animation_end.run(event)
                    >
                        {children()}
                    </div>
                }
                .into_any()
            }
        }}
    }
}

fn create_presence_binding<E>(
    node_ref: NodeRef<E>,
    present: Signal<bool>,
    on_exit_complete: Option<Callback<()>>,
) -> PresenceBinding<E>
where
    E: html::ElementType,
    E::Output: PresenceElementOutput,
{
    let state = RwSignal::new(if present.get_untracked() {
        PresenceState::Mounted
    } else {
        PresenceState::Unmounted
    });
    #[cfg(target_arch = "wasm32")]
    let exit_timeout = Arc::new(Mutex::new(None::<PresenceExitTimeout>));

    let on_exit_complete = on_exit_complete.clone();

    #[cfg(target_arch = "wasm32")]
    let exit_timeout_for_end_handler = Arc::clone(&exit_timeout);
    let end_handler: Arc<dyn Fn() + Send + Sync> = Arc::new(move || {
        #[cfg(target_arch = "wasm32")]
        presence_clear_exit_timeout(&exit_timeout_for_end_handler);

        let current_state = state.get_untracked();
        let next = presence_state_next(current_state, present.get_untracked(), true);
        if next != current_state {
            state.set(next);
            if next == PresenceState::Unmounted {
                if let Some(callback) = on_exit_complete.as_ref() {
                    callback.run(());
                }
            }
        }
    });

    let effect = RenderEffect::new({
        let end_handler = Arc::clone(&end_handler);
        #[cfg(target_arch = "wasm32")]
        let exit_timeout = Arc::clone(&exit_timeout);

        move |_| {
            let current = state.get();
            let is_present = present.get();

            if is_present {
                #[cfg(target_arch = "wasm32")]
                presence_clear_exit_timeout(&exit_timeout);

                let next = presence_state_next(current, true, false);
                if next != current {
                    state.set(next);
                }
                return;
            }

            if current != PresenceState::Mounted {
                return;
            }

            let exit_timeout_ms = presence_node_ref_exit_timeout_ms(node_ref);

            if exit_timeout_ms == 0 {
                end_handler();
                return;
            }

            let next = presence_state_next(current, false, false);
            if next != current {
                state.set(next);
            }

            #[cfg(target_arch = "wasm32")]
            presence_schedule_exit_timeout(
                &exit_timeout,
                exit_timeout_ms,
                Arc::clone(&end_handler),
            );
        }
    });

    #[cfg(target_arch = "wasm32")]
    {
        let exit_timeout = Arc::clone(&exit_timeout);
        on_cleanup(move || {
            presence_clear_exit_timeout(&exit_timeout);
            drop(effect);
        });
    }
    #[cfg(not(target_arch = "wasm32"))]
    on_cleanup(move || {
        drop(effect);
    });

    let transition_end = {
        let end_handler = Arc::clone(&end_handler);
        Callback::new(move |event: TransitionEvent| {
            if presence_should_complete_exit(
                state.get_untracked(),
                present.get_untracked(),
                presence_event_matches_current_target(event.target(), event.current_target()),
            ) {
                end_handler();
            }
        })
    };
    let animation_end = {
        let end_handler = Arc::clone(&end_handler);
        Callback::new(move |event: AnimationEvent| {
            if presence_should_complete_exit(
                state.get_untracked(),
                present.get_untracked(),
                presence_event_matches_current_target(event.target(), event.current_target()),
            ) {
                end_handler();
            }
        })
    };

    PresenceBinding {
        node_ref,
        state,
        present,
        transition_end,
        animation_end,
    }
}

#[cfg(target_arch = "wasm32")]
fn presence_node_ref_exit_timeout_ms<E>(node_ref: NodeRef<E>) -> u32
where
    E: html::ElementType,
    E::Output: wasm_bindgen::JsCast + Clone + 'static,
{
    use wasm_bindgen::JsCast;

    node_ref
        .get_untracked()
        .and_then(|root| root.dyn_into::<web_sys::Element>().ok())
        .as_ref()
        .map(presence_exit_timeout_ms)
        .unwrap_or(0)
}

#[cfg(not(target_arch = "wasm32"))]
fn presence_node_ref_exit_timeout_ms<E>(node_ref: NodeRef<E>) -> u32
where
    E: html::ElementType,
{
    let _ = node_ref;
    0
}

#[cfg(test)]
mod tests {
    use super::{
        PresenceState, presence_event_matches_current_target, presence_exit_timeout_ms_values,
        presence_should_complete_exit, presence_state_next,
    };

    #[test]
    fn presence_state_moves_to_exiting_on_close() {
        let next = presence_state_next(PresenceState::Mounted, false, false);
        assert_eq!(next, PresenceState::Exiting);
    }

    #[test]
    fn presence_state_unmounts_after_exit() {
        let next = presence_state_next(PresenceState::Exiting, false, true);
        assert_eq!(next, PresenceState::Unmounted);
    }

    #[test]
    fn presence_state_mounts_when_present() {
        let next = presence_state_next(PresenceState::Unmounted, true, false);
        assert_eq!(next, PresenceState::Mounted);
    }

    #[test]
    fn presence_only_completes_exit_for_self_events() {
        assert!(presence_should_complete_exit(
            PresenceState::Exiting,
            false,
            true,
        ));
        assert!(!presence_should_complete_exit(
            PresenceState::Exiting,
            false,
            false,
        ));
        assert!(!presence_should_complete_exit(
            PresenceState::Mounted,
            false,
            true,
        ));
    }

    #[test]
    fn presence_event_match_requires_same_target() {
        assert!(presence_event_matches_current_target(Some(1), Some(1)));
        assert!(!presence_event_matches_current_target(Some(1), Some(2)));
        assert!(!presence_event_matches_current_target::<u8>(None, Some(1)));
    }

    #[test]
    fn presence_exit_timeout_ms_handles_css_lists() {
        assert_eq!(presence_exit_timeout_ms_values("0s", "0s", "0s", "0s"), 0);
        assert_eq!(
            presence_exit_timeout_ms_values("150ms, 75ms", "50ms", "0s", "0s"),
            200,
        );
        assert_eq!(
            presence_exit_timeout_ms_values("0s", "0s", "0.2s", "0.1s"),
            300,
        );
    }
}
