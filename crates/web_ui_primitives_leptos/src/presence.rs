//! Presence primitive for exit-aware mounting and unmounting.

use leptos::ev::{AnimationEvent, TransitionEvent};
use leptos::html;
use leptos::prelude::*;
use std::sync::Arc;
#[cfg(target_arch = "wasm32")]
use std::sync::Mutex;

/// The browser lifecycle contract implemented by this crate.
pub const PRESENCE_ABI_VERSION: u32 = 2;

#[cfg(target_arch = "wasm32")]
const PRESENCE_HARD_CAP_MS: f64 = 30_000.0;
#[cfg(target_arch = "wasm32")]
const PRESENCE_TRANSITION_GRACE_MS: f64 = 50.0;

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
    transition_cancel: Callback<TransitionEvent>,
    animation_end: Callback<AnimationEvent>,
    animation_cancel: Callback<AnimationEvent>,
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

    /// Returns the transition-cancel handler for the attached element.
    pub fn transition_cancel_handler(&self) -> Callback<TransitionEvent> {
        self.transition_cancel
    }

    /// Returns the animation-end handler for the attached element.
    pub fn animation_end_handler(&self) -> Callback<AnimationEvent> {
        self.animation_end
    }

    /// Returns the animation-cancel handler for the attached element.
    pub fn animation_cancel_handler(&self) -> Callback<AnimationEvent> {
        self.animation_cancel
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

#[cfg(target_arch = "wasm32")]
fn presence_event_matches_current_target<T: PartialEq>(
    target: Option<T>,
    current_target: Option<T>,
) -> bool {
    matches!(
        (target, current_target),
        (Some(target), Some(current_target)) if target == current_target
    )
}

#[cfg(any(test, target_arch = "wasm32"))]
#[derive(Debug, Clone, PartialEq, Eq)]
enum PresenceTrackKey {
    Transition(String),
    TransitionAll,
    Animation { name: String, occurrence: usize },
    UnknownTransition(usize),
    UnknownAnimation(usize),
}

#[cfg(any(test, target_arch = "wasm32"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PresencePlayState {
    Running,
    Paused,
}

#[cfg(any(test, target_arch = "wasm32"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PresenceDeadlineClass {
    FiniteTransition,
    EventOnlyAnimation,
    ConservativeUnknown,
}

#[cfg(any(test, target_arch = "wasm32"))]
#[derive(Debug, Clone, PartialEq)]
struct PresenceTrackDescriptor {
    key: PresenceTrackKey,
    winning_slot: usize,
    duration_ms: Option<f64>,
    delay_ms: Option<f64>,
    iteration_count: Option<f64>,
    infinite_iterations: bool,
    play_state: Option<PresencePlayState>,
    deadline_class: PresenceDeadlineClass,
    parse_error: Option<&'static str>,
}

#[cfg(any(test, target_arch = "wasm32"))]
impl PresenceTrackDescriptor {
    fn transition_total_ms(&self) -> Option<f64> {
        if self.deadline_class != PresenceDeadlineClass::FiniteTransition {
            return None;
        }
        self.duration_ms
            .zip(self.delay_ms)
            .map(|(duration, delay)| duration + delay)
    }
}

#[cfg(any(test, target_arch = "wasm32"))]
#[derive(Debug, Clone, PartialEq)]
struct PresenceTimingSignature {
    raw: [String; 8],
    tracks: Vec<PresenceTrackDescriptor>,
}

#[cfg(any(test, target_arch = "wasm32"))]
fn presence_split_css_list(value: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut start = 0;
    let mut quote = None;
    let mut escaped = false;
    let mut depth = 0_u32;

    for (index, character) in value.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if character == '\\' {
            escaped = true;
            continue;
        }
        if let Some(active_quote) = quote {
            if character == active_quote {
                quote = None;
            }
            continue;
        }
        match character {
            '\'' | '"' => quote = Some(character),
            '(' => depth = depth.saturating_add(1),
            ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                values.push(value[start..index].trim().to_owned());
                start = index + character.len_utf8();
            }
            _ => {}
        }
    }
    values.push(value[start..].trim().to_owned());
    values
}

#[cfg(any(test, target_arch = "wasm32"))]
fn presence_cyclic_value(values: &[String], index: usize) -> Option<&str> {
    (!values.is_empty()).then(|| values[index % values.len()].as_str())
}

#[cfg(any(test, target_arch = "wasm32"))]
fn presence_parse_time_ms(value: &str, allow_negative: bool) -> Result<f64, &'static str> {
    let value = value.trim();
    let parsed = if let Some(value) = value.strip_suffix("ms") {
        value.trim().parse::<f64>().map_err(|_| "invalid-time")?
    } else if let Some(value) = value.strip_suffix('s') {
        value.trim().parse::<f64>().map_err(|_| "invalid-time")? * 1000.0
    } else {
        return Err("invalid-time-unit");
    };
    if !parsed.is_finite() {
        return Err("non-finite-time");
    }
    if !allow_negative && parsed < 0.0 {
        return Err("negative-duration");
    }
    Ok(parsed)
}

#[cfg(any(test, target_arch = "wasm32"))]
fn presence_parse_iteration_count(value: &str) -> Result<(Option<f64>, bool), &'static str> {
    if value.trim().eq_ignore_ascii_case("infinite") {
        return Ok((None, true));
    }
    let parsed = value
        .trim()
        .parse::<f64>()
        .map_err(|_| "invalid-iteration-count")?;
    if !parsed.is_finite() {
        return Err("non-finite-iteration-count");
    }
    if parsed < 0.0 {
        return Err("negative-iteration-count");
    }
    Ok((Some(parsed), false))
}

#[cfg(any(test, target_arch = "wasm32"))]
fn presence_parse_play_state(value: &str) -> Result<PresencePlayState, &'static str> {
    match value.trim().to_ascii_lowercase().as_str() {
        "running" => Ok(PresencePlayState::Running),
        "paused" => Ok(PresencePlayState::Paused),
        _ => Err("invalid-play-state"),
    }
}

#[cfg(any(test, target_arch = "wasm32"))]
fn presence_unknown_transition(slot: usize, error: &'static str) -> PresenceTrackDescriptor {
    PresenceTrackDescriptor {
        key: PresenceTrackKey::UnknownTransition(slot),
        winning_slot: slot,
        duration_ms: None,
        delay_ms: None,
        iteration_count: Some(1.0),
        infinite_iterations: false,
        play_state: None,
        deadline_class: PresenceDeadlineClass::ConservativeUnknown,
        parse_error: Some(error),
    }
}

#[cfg(any(test, target_arch = "wasm32"))]
fn presence_unknown_animation(slot: usize, error: &'static str) -> PresenceTrackDescriptor {
    PresenceTrackDescriptor {
        key: PresenceTrackKey::UnknownAnimation(slot),
        winning_slot: slot,
        duration_ms: None,
        delay_ms: None,
        iteration_count: None,
        infinite_iterations: false,
        play_state: None,
        deadline_class: PresenceDeadlineClass::ConservativeUnknown,
        parse_error: Some(error),
    }
}

#[cfg(any(test, target_arch = "wasm32"))]
fn presence_transition_tracks(raw: &[String; 8]) -> Vec<PresenceTrackDescriptor> {
    use std::collections::BTreeMap;

    let properties = presence_split_css_list(&raw[0]);
    let durations = presence_split_css_list(&raw[1]);
    let delays = presence_split_css_list(&raw[2]);
    let mut last_explicit = BTreeMap::<String, usize>::new();
    let mut last_all = None;

    for (slot, property) in properties.iter().enumerate() {
        let property = property.trim().to_ascii_lowercase();
        match property.as_str() {
            "none" => {}
            "all" => last_all = Some(slot),
            _ => {
                last_explicit.insert(property, slot);
            }
        }
    }

    let mut tracks = Vec::new();
    for (slot, property) in properties.iter().enumerate() {
        let property = property.trim().to_ascii_lowercase();
        let retained = match property.as_str() {
            "none" => false,
            "all" => last_all == Some(slot),
            _ => {
                last_explicit.get(&property).copied() == Some(slot)
                    && last_all.is_none_or(|all_slot| slot > all_slot)
            }
        };
        if !retained {
            continue;
        }

        let Some(duration) = presence_cyclic_value(&durations, slot) else {
            tracks.push(presence_unknown_transition(slot, "missing-duration"));
            continue;
        };
        let Some(delay) = presence_cyclic_value(&delays, slot) else {
            tracks.push(presence_unknown_transition(slot, "missing-delay"));
            continue;
        };
        let duration = match presence_parse_time_ms(duration, false) {
            Ok(duration) => duration,
            Err(error) => {
                tracks.push(presence_unknown_transition(slot, error));
                continue;
            }
        };
        let delay = match presence_parse_time_ms(delay, true) {
            Ok(delay) => delay,
            Err(error) => {
                tracks.push(presence_unknown_transition(slot, error));
                continue;
            }
        };
        if duration + delay <= 0.0 {
            continue;
        }
        tracks.push(PresenceTrackDescriptor {
            key: if property == "all" {
                PresenceTrackKey::TransitionAll
            } else {
                PresenceTrackKey::Transition(property)
            },
            winning_slot: slot,
            duration_ms: Some(duration),
            delay_ms: Some(delay),
            iteration_count: Some(1.0),
            infinite_iterations: false,
            play_state: None,
            deadline_class: PresenceDeadlineClass::FiniteTransition,
            parse_error: None,
        });
    }
    tracks
}

#[cfg(any(test, target_arch = "wasm32"))]
fn presence_animation_tracks(raw: &[String; 8]) -> Vec<PresenceTrackDescriptor> {
    use std::collections::BTreeMap;

    let names = presence_split_css_list(&raw[3]);
    let durations = presence_split_css_list(&raw[4]);
    let delays = presence_split_css_list(&raw[5]);
    let iterations = presence_split_css_list(&raw[6]);
    let play_states = presence_split_css_list(&raw[7]);
    let mut occurrences = BTreeMap::<String, usize>::new();
    let mut tracks = Vec::new();

    for (slot, name) in names.iter().enumerate() {
        let name = name.trim().to_owned();
        if name.eq_ignore_ascii_case("none") {
            continue;
        }
        let occurrence = occurrences.entry(name.clone()).or_default();
        let key = PresenceTrackKey::Animation {
            name: name.clone(),
            occurrence: *occurrence,
        };
        *occurrence += 1;

        let Some(duration) = presence_cyclic_value(&durations, slot) else {
            tracks.push(presence_unknown_animation(slot, "missing-duration"));
            continue;
        };
        let Some(delay) = presence_cyclic_value(&delays, slot) else {
            tracks.push(presence_unknown_animation(slot, "missing-delay"));
            continue;
        };
        let Some(iteration_count) = presence_cyclic_value(&iterations, slot) else {
            tracks.push(presence_unknown_animation(slot, "missing-iteration-count"));
            continue;
        };
        let Some(play_state) = presence_cyclic_value(&play_states, slot) else {
            tracks.push(presence_unknown_animation(slot, "missing-play-state"));
            continue;
        };

        let duration = match presence_parse_time_ms(duration, false) {
            Ok(duration) => duration,
            Err(error) => {
                tracks.push(presence_unknown_animation(slot, error));
                continue;
            }
        };
        let delay = match presence_parse_time_ms(delay, true) {
            Ok(delay) => delay,
            Err(error) => {
                tracks.push(presence_unknown_animation(slot, error));
                continue;
            }
        };
        let (iteration_count, infinite_iterations) =
            match presence_parse_iteration_count(iteration_count) {
                Ok(iteration_count) => iteration_count,
                Err(error) => {
                    tracks.push(presence_unknown_animation(slot, error));
                    continue;
                }
            };
        let play_state = match presence_parse_play_state(play_state) {
            Ok(play_state) => play_state,
            Err(error) => {
                tracks.push(presence_unknown_animation(slot, error));
                continue;
            }
        };
        let active_interval = if infinite_iterations {
            f64::INFINITY
        } else {
            delay + duration * iteration_count.unwrap_or_default()
        };
        if active_interval <= 0.0 {
            continue;
        }
        tracks.push(PresenceTrackDescriptor {
            key,
            winning_slot: slot,
            duration_ms: Some(duration),
            delay_ms: Some(delay),
            iteration_count,
            infinite_iterations,
            play_state: Some(play_state),
            deadline_class: PresenceDeadlineClass::EventOnlyAnimation,
            parse_error: None,
        });
    }
    tracks
}

#[cfg(any(test, target_arch = "wasm32"))]
fn presence_timing_signature(raw: [String; 8]) -> PresenceTimingSignature {
    let mut tracks = presence_transition_tracks(&raw);
    tracks.extend(presence_animation_tracks(&raw));
    PresenceTimingSignature { raw, tracks }
}

#[cfg(any(test, target_arch = "wasm32"))]
fn presence_pending_tracks(
    signature: &PresenceTimingSignature,
    elapsed_ms: f64,
) -> Vec<PresenceTrackDescriptor> {
    signature
        .tracks
        .iter()
        .filter(|track| {
            track
                .transition_total_ms()
                .is_none_or(|total| total > elapsed_ms)
        })
        .cloned()
        .collect()
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
/// Conditionally mounts content while allowing every root exit track to finish.
///
/// On wasm, completion follows Presence ABI v2 and observes transition and
/// animation end/cancel events plus computed-style changes. Zero-motion exits
/// still complete asynchronously. On non-wasm targets, timing is immediate.
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
                let transition_cancel = presence.transition_cancel_handler();
                let animation_end = presence.animation_end_handler();
                let animation_cancel = presence.animation_cancel_handler();

                view! {
                    <div
                        node_ref=node_ref
                        data-state=move || data_state.data_state()
                        on:transitionend=move |event| transition_end.run(event)
                        on:transitioncancel=move |event| transition_cancel.run(event)
                        on:animationend=move |event| animation_end.run(event)
                        on:animationcancel=move |event| animation_cancel.run(event)
                    >
                        {children()}
                    </div>
                }
                .into_any()
            }
        }}
    }
}

#[cfg(not(target_arch = "wasm32"))]
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
    let complete = Arc::new(move || {
        let current = state.get_untracked();
        let next = presence_state_next(current, present.get_untracked(), true);
        if next != current {
            state.set(next);
            if next == PresenceState::Unmounted
                && let Some(callback) = on_exit_complete.as_ref()
            {
                callback.run(());
            }
        }
    });
    let effect = RenderEffect::new({
        let complete = Arc::clone(&complete);
        move |_| {
            let current = state.get();
            if present.get() {
                if current != PresenceState::Mounted {
                    state.set(PresenceState::Mounted);
                }
            } else if current == PresenceState::Mounted {
                complete();
            }
        }
    });
    on_cleanup(move || drop(effect));

    let transition_end = {
        let complete = Arc::clone(&complete);
        Callback::new(move |_| complete())
    };
    let transition_cancel = transition_end;
    let animation_end = {
        let complete = Arc::clone(&complete);
        Callback::new(move |_| complete())
    };
    let animation_cancel = animation_end;

    PresenceBinding {
        node_ref,
        state,
        present,
        transition_end,
        transition_cancel,
        animation_end,
        animation_cancel,
    }
}

#[cfg(target_arch = "wasm32")]
struct PresenceBrowserHandle {
    id: i32,
    callback: send_wrapper::SendWrapper<wasm_bindgen::closure::Closure<dyn FnMut()>>,
}

#[cfg(target_arch = "wasm32")]
struct PresenceCloseCycle {
    generation: u64,
    close_start_ms: f64,
    signature: Option<PresenceTimingSignature>,
    pending: Vec<PresenceTrackDescriptor>,
    frame: Option<PresenceBrowserHandle>,
    finite_timeout: Option<PresenceBrowserHandle>,
    hard_timeout: Option<PresenceBrowserHandle>,
}

#[cfg(target_arch = "wasm32")]
#[derive(Default)]
struct PresenceBrowserState {
    generation: u64,
    cycle: Option<PresenceCloseCycle>,
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Copy, PartialEq, Eq)]
enum PresenceSamplePhase {
    InitialMicrotask,
    Frame,
    FiniteTimeout,
    FinalMicrotask,
}

#[cfg(target_arch = "wasm32")]
struct PresenceBrowserRuntime<E>
where
    E: html::ElementType,
{
    node_ref: NodeRef<E>,
    state: RwSignal<PresenceState>,
    present: Signal<bool>,
    on_exit_complete: Option<Callback<()>>,
    browser: Arc<Mutex<PresenceBrowserState>>,
}

#[cfg(target_arch = "wasm32")]
impl<E> PresenceBrowserRuntime<E>
where
    E: html::ElementType,
    E::Output: wasm_bindgen::JsCast + Clone + 'static,
{
    fn now_ms() -> f64 {
        web_sys::window()
            .and_then(|window| window.performance())
            .map(|performance| performance.now())
            .unwrap_or_else(js_sys::Date::now)
    }

    fn cancel_handle(handle: Option<PresenceBrowserHandle>, frame: bool) {
        let Some(handle) = handle else {
            return;
        };
        if let Some(window) = web_sys::window() {
            if frame {
                let _ = window.cancel_animation_frame(handle.id);
            } else {
                window.clear_timeout_with_handle(handle.id);
            }
        }
        drop(handle.callback);
    }

    fn cancel_cycle(cycle: &mut PresenceCloseCycle) {
        Self::cancel_handle(cycle.frame.take(), true);
        Self::cancel_handle(cycle.finite_timeout.take(), false);
        Self::cancel_handle(cycle.hard_timeout.take(), false);
    }

    fn begin_close(self: &Arc<Self>) {
        let generation = {
            let mut browser = self
                .browser
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if let Some(mut old_cycle) = browser.cycle.take() {
                Self::cancel_cycle(&mut old_cycle);
            }
            browser.generation = browser.generation.wrapping_add(1);
            let generation = browser.generation;
            browser.cycle = Some(PresenceCloseCycle {
                generation,
                close_start_ms: Self::now_ms(),
                signature: None,
                pending: Vec::new(),
                frame: None,
                finite_timeout: None,
                hard_timeout: None,
            });
            generation
        };
        self.state.set(PresenceState::Exiting);
        self.schedule_hard_timeout(generation);
        self.queue_sample_microtask(generation, PresenceSamplePhase::InitialMicrotask);
    }

    fn reopen(&self) {
        let mut browser = self
            .browser
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        browser.generation = browser.generation.wrapping_add(1);
        if let Some(mut cycle) = browser.cycle.take() {
            Self::cancel_cycle(&mut cycle);
        }
        drop(browser);
        if self.state.get_untracked() != PresenceState::Mounted {
            self.state.set(PresenceState::Mounted);
        }
    }

    fn queue_sample_microtask(self: &Arc<Self>, generation: u64, phase: PresenceSamplePhase) {
        let runtime = Arc::clone(self);
        wasm_bindgen_futures::spawn_local(async move {
            let _ = wasm_bindgen_futures::JsFuture::from(js_sys::Promise::resolve(
                &wasm_bindgen::JsValue::UNDEFINED,
            ))
            .await;
            runtime.process_sample(generation, phase);
        });
    }

    fn schedule_hard_timeout(self: &Arc<Self>, generation: u64) {
        use wasm_bindgen::JsCast;

        let Some(window) = web_sys::window() else {
            self.finish(generation);
            return;
        };
        let runtime = Arc::clone(self);
        let callback = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
            runtime.take_timeout(generation, false);
            runtime.finish(generation);
        }) as Box<dyn FnMut()>);
        let Ok(id) = window.set_timeout_with_callback_and_timeout_and_arguments_0(
            callback.as_ref().unchecked_ref(),
            PRESENCE_HARD_CAP_MS as i32,
        ) else {
            self.finish(generation);
            return;
        };
        let handle = PresenceBrowserHandle {
            id,
            callback: send_wrapper::SendWrapper::new(callback),
        };
        let mut browser = self
            .browser
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(cycle) = browser
            .cycle
            .as_mut()
            .filter(|cycle| cycle.generation == generation)
        {
            cycle.hard_timeout = Some(handle);
        } else {
            drop(browser);
            Self::cancel_handle(Some(handle), false);
        }
    }

    fn schedule_finite_timeout(self: &Arc<Self>, generation: u64) {
        use wasm_bindgen::JsCast;

        let (delay_ms, previous) = {
            let mut browser = self
                .browser
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let Some(cycle) = browser
                .cycle
                .as_mut()
                .filter(|cycle| cycle.generation == generation)
            else {
                return;
            };
            let previous = cycle.finite_timeout.take();
            let only_finite_transitions = !cycle.pending.is_empty()
                && cycle
                    .pending
                    .iter()
                    .all(|track| track.deadline_class == PresenceDeadlineClass::FiniteTransition);
            let delay_ms = only_finite_transitions.then(|| {
                let max_total = cycle
                    .pending
                    .iter()
                    .filter_map(PresenceTrackDescriptor::transition_total_ms)
                    .fold(0.0, f64::max);
                (cycle.close_start_ms + max_total + PRESENCE_TRANSITION_GRACE_MS - Self::now_ms())
                    .clamp(0.0, PRESENCE_HARD_CAP_MS)
            });
            (delay_ms, previous)
        };
        Self::cancel_handle(previous, false);
        let Some(delay_ms) = delay_ms else {
            return;
        };
        let Some(window) = web_sys::window() else {
            self.process_sample(generation, PresenceSamplePhase::FiniteTimeout);
            return;
        };
        let runtime = Arc::clone(self);
        let callback = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
            runtime.take_timeout(generation, true);
            runtime.process_sample(generation, PresenceSamplePhase::FiniteTimeout);
        }) as Box<dyn FnMut()>);
        let Ok(id) = window.set_timeout_with_callback_and_timeout_and_arguments_0(
            callback.as_ref().unchecked_ref(),
            delay_ms.ceil().min(i32::MAX as f64) as i32,
        ) else {
            self.process_sample(generation, PresenceSamplePhase::FiniteTimeout);
            return;
        };
        let handle = PresenceBrowserHandle {
            id,
            callback: send_wrapper::SendWrapper::new(callback),
        };
        let mut browser = self
            .browser
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(cycle) = browser
            .cycle
            .as_mut()
            .filter(|cycle| cycle.generation == generation)
        {
            cycle.finite_timeout = Some(handle);
        } else {
            drop(browser);
            Self::cancel_handle(Some(handle), false);
        }
    }

    fn take_timeout(&self, generation: u64, finite: bool) {
        let handle = {
            let mut browser = self
                .browser
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let Some(cycle) = browser
                .cycle
                .as_mut()
                .filter(|cycle| cycle.generation == generation)
            else {
                return;
            };
            if finite {
                cycle.finite_timeout.take()
            } else {
                cycle.hard_timeout.take()
            }
        };
        drop(handle);
    }

    fn schedule_frame(self: &Arc<Self>, generation: u64) {
        use wasm_bindgen::JsCast;

        {
            let browser = self
                .browser
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let Some(cycle) = browser
                .cycle
                .as_ref()
                .filter(|cycle| cycle.generation == generation)
            else {
                return;
            };
            if cycle.frame.is_some() {
                return;
            }
        }
        let Some(window) = web_sys::window() else {
            self.queue_sample_microtask(generation, PresenceSamplePhase::FinalMicrotask);
            return;
        };
        let runtime = Arc::clone(self);
        let callback = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
            let old_frame = runtime.take_frame(generation);
            runtime.process_sample(generation, PresenceSamplePhase::Frame);
            drop(old_frame);
        }) as Box<dyn FnMut()>);
        let Ok(id) = window.request_animation_frame(callback.as_ref().unchecked_ref()) else {
            self.queue_sample_microtask(generation, PresenceSamplePhase::FinalMicrotask);
            return;
        };
        let handle = PresenceBrowserHandle {
            id,
            callback: send_wrapper::SendWrapper::new(callback),
        };
        let mut browser = self
            .browser
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(cycle) = browser
            .cycle
            .as_mut()
            .filter(|cycle| cycle.generation == generation)
        {
            cycle.frame = Some(handle);
        } else {
            drop(browser);
            Self::cancel_handle(Some(handle), true);
        }
    }

    fn take_frame(&self, generation: u64) -> Option<PresenceBrowserHandle> {
        let mut browser = self
            .browser
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        browser
            .cycle
            .as_mut()
            .filter(|cycle| cycle.generation == generation)
            .and_then(|cycle| cycle.frame.take())
    }

    fn sample_signature(&self) -> PresenceTimingSignature {
        use wasm_bindgen::JsCast;

        let empty = || {
            presence_timing_signature([
                "none".into(),
                "0s".into(),
                "0s".into(),
                "none".into(),
                "0s".into(),
                "0s".into(),
                "1".into(),
                "running".into(),
            ])
        };
        let Some(element) = self
            .node_ref
            .get_untracked()
            .and_then(|element| element.dyn_into::<web_sys::Element>().ok())
        else {
            return empty();
        };
        let Some(style) =
            web_sys::window().and_then(|window| window.get_computed_style(&element).ok().flatten())
        else {
            return empty();
        };
        let property = |name| style.get_property_value(name).unwrap_or_default();
        presence_timing_signature([
            property("transition-property"),
            property("transition-duration"),
            property("transition-delay"),
            property("animation-name"),
            property("animation-duration"),
            property("animation-delay"),
            property("animation-iteration-count"),
            property("animation-play-state"),
        ])
    }

    fn process_sample(self: &Arc<Self>, generation: u64, phase: PresenceSamplePhase) {
        let signature = self.sample_signature();
        let now = Self::now_ms();
        let mut should_finish = false;
        let mut should_schedule_frame = false;
        let mut should_schedule_finite_timeout = false;
        {
            let mut browser = self
                .browser
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let Some(cycle) = browser
                .cycle
                .as_mut()
                .filter(|cycle| cycle.generation == generation)
            else {
                return;
            };
            let elapsed = (now - cycle.close_start_ms).max(0.0);
            if elapsed >= PRESENCE_HARD_CAP_MS {
                should_finish = true;
            } else {
                let changed = cycle.signature.as_ref() != Some(&signature);
                if changed {
                    cycle.pending = presence_pending_tracks(&signature, elapsed);
                    cycle.signature = Some(signature);
                    should_schedule_finite_timeout = true;
                } else if matches!(
                    phase,
                    PresenceSamplePhase::Frame | PresenceSamplePhase::FiniteTimeout
                ) {
                    cycle.pending.retain(|track| {
                        track
                            .transition_total_ms()
                            .is_none_or(|total| elapsed < total + PRESENCE_TRANSITION_GRACE_MS)
                    });
                }

                if phase == PresenceSamplePhase::FinalMicrotask
                    && !changed
                    && cycle.pending.is_empty()
                {
                    should_finish = true;
                } else if phase == PresenceSamplePhase::Frame
                    && !changed
                    && cycle.pending.is_empty()
                {
                    drop(browser);
                    self.queue_sample_microtask(generation, PresenceSamplePhase::FinalMicrotask);
                    return;
                } else {
                    should_schedule_frame = true;
                }
            }
        }
        if should_finish {
            self.finish(generation);
            return;
        }
        if should_schedule_finite_timeout {
            self.schedule_finite_timeout(generation);
        }
        if should_schedule_frame {
            self.schedule_frame(generation);
        }
    }

    fn handle_transition_event(self: &Arc<Self>, event: TransitionEvent) {
        if !presence_event_matches_current_target(event.target(), event.current_target())
            || !event.pseudo_element().is_empty()
        {
            return;
        }
        let property = event.property_name().trim().to_ascii_lowercase();
        let generation = {
            let mut browser = self
                .browser
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let Some(cycle) = browser.cycle.as_mut() else {
                return;
            };
            if event.time_stamp() < cycle.close_start_ms || cycle.signature.is_none() {
                return;
            }
            let Some(index) = cycle.pending.iter().position(
                |track| matches!(&track.key, PresenceTrackKey::Transition(key) if key == &property),
            ) else {
                return;
            };
            cycle.pending.remove(index);
            cycle.generation
        };
        self.schedule_frame(generation);
    }

    fn handle_animation_event(self: &Arc<Self>, event: AnimationEvent) {
        if !presence_event_matches_current_target(event.target(), event.current_target())
            || !event.pseudo_element().is_empty()
        {
            return;
        }
        let name = event.animation_name();
        let generation = {
            let mut browser = self
                .browser
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let Some(cycle) = browser.cycle.as_mut() else {
                return;
            };
            if event.time_stamp() < cycle.close_start_ms || cycle.signature.is_none() {
                return;
            }
            let Some(index) = cycle.pending.iter().position(|track| {
                matches!(&track.key, PresenceTrackKey::Animation { name: key, .. } if key == &name)
            }) else {
                return;
            };
            cycle.pending.remove(index);
            cycle.generation
        };
        self.schedule_frame(generation);
    }

    fn finish(&self, generation: u64) {
        let mut cycle = {
            let mut browser = self
                .browser
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let Some(cycle) = browser
                .cycle
                .as_ref()
                .filter(|cycle| cycle.generation == generation)
            else {
                return;
            };
            let _ = cycle;
            browser.generation = browser.generation.wrapping_add(1);
            browser.cycle.take().expect("checked close cycle")
        };
        Self::cancel_cycle(&mut cycle);
        if !self.present.get_untracked() && self.state.get_untracked() == PresenceState::Exiting {
            self.state.set(PresenceState::Unmounted);
            if let Some(callback) = self.on_exit_complete.as_ref() {
                callback.run(());
            }
        }
    }

    fn cleanup(&self) {
        let mut browser = self
            .browser
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        browser.generation = browser.generation.wrapping_add(1);
        if let Some(mut cycle) = browser.cycle.take() {
            Self::cancel_cycle(&mut cycle);
        }
    }
}

#[cfg(target_arch = "wasm32")]
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
    let runtime = Arc::new(PresenceBrowserRuntime {
        node_ref,
        state,
        present,
        on_exit_complete,
        browser: Arc::new(Mutex::new(PresenceBrowserState::default())),
    });
    let effect = RenderEffect::new({
        let runtime = Arc::clone(&runtime);
        move |_| {
            let current = runtime.state.get();
            if runtime.present.get() {
                if current != PresenceState::Mounted {
                    runtime.reopen();
                }
            } else if current == PresenceState::Mounted {
                runtime.begin_close();
            }
        }
    });
    on_cleanup({
        let runtime = Arc::clone(&runtime);
        move || {
            runtime.cleanup();
            drop(effect);
        }
    });

    let transition_end = {
        let runtime = Arc::clone(&runtime);
        Callback::new(move |event| runtime.handle_transition_event(event))
    };
    let transition_cancel = {
        let runtime = Arc::clone(&runtime);
        Callback::new(move |event| runtime.handle_transition_event(event))
    };
    let animation_end = {
        let runtime = Arc::clone(&runtime);
        Callback::new(move |event| runtime.handle_animation_event(event))
    };
    let animation_cancel = {
        let runtime = Arc::clone(&runtime);
        Callback::new(move |event| runtime.handle_animation_event(event))
    };

    PresenceBinding {
        node_ref,
        state,
        present,
        transition_end,
        transition_cancel,
        animation_end,
        animation_cancel,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        PRESENCE_ABI_VERSION, PresenceDeadlineClass, PresenceState, PresenceTrackKey,
        presence_pending_tracks, presence_state_next, presence_timing_signature,
    };

    fn signature(values: [&str; 8]) -> super::PresenceTimingSignature {
        presence_timing_signature(values.map(str::to_owned))
    }

    #[test]
    fn presence_exports_abi_v2() {
        assert_eq!(PRESENCE_ABI_VERSION, 2);
    }

    #[test]
    fn presence_state_handles_close_completion_and_reopen() {
        assert_eq!(
            presence_state_next(PresenceState::Mounted, false, false),
            PresenceState::Exiting
        );
        assert_eq!(
            presence_state_next(PresenceState::Exiting, false, true),
            PresenceState::Unmounted
        );
        assert_eq!(
            presence_state_next(PresenceState::Unmounted, true, false),
            PresenceState::Mounted
        );
    }

    #[test]
    fn transition_lists_expand_cyclically_and_keep_last_explicit_slot() {
        let signature = signature([
            "opacity, transform, opacity",
            "100ms, 200ms",
            "10ms",
            "none",
            "0s",
            "0s",
            "1",
            "running",
        ]);
        assert_eq!(signature.tracks.len(), 2);
        assert_eq!(signature.tracks[0].winning_slot, 1);
        assert_eq!(signature.tracks[0].duration_ms, Some(200.0));
        assert_eq!(signature.tracks[1].winning_slot, 2);
        assert_eq!(signature.tracks[1].duration_ms, Some(100.0));
    }

    #[test]
    fn transition_all_overlap_order_is_normalized() {
        let all_then_opacity = signature([
            "all, opacity",
            "100ms, 200ms",
            "0s",
            "none",
            "0s",
            "0s",
            "1",
            "running",
        ]);
        assert_eq!(all_then_opacity.tracks.len(), 2);
        assert!(matches!(
            all_then_opacity.tracks[0].key,
            PresenceTrackKey::TransitionAll
        ));
        assert!(matches!(
            &all_then_opacity.tracks[1].key,
            PresenceTrackKey::Transition(property) if property == "opacity"
        ));

        let opacity_then_all = signature([
            "opacity, all",
            "100ms, 200ms",
            "0s",
            "none",
            "0s",
            "0s",
            "1",
            "running",
        ]);
        assert_eq!(opacity_then_all.tracks.len(), 1);
        assert!(matches!(
            opacity_then_all.tracks[0].key,
            PresenceTrackKey::TransitionAll
        ));
    }

    #[test]
    fn animation_lists_expand_and_duplicate_names_keep_occurrences() {
        let signature = signature([
            "none",
            "0s",
            "0s",
            "fade, fade, slide",
            "100ms, 200ms",
            "0s",
            "1, 2",
            "running, paused",
        ]);
        assert_eq!(signature.tracks.len(), 3);
        assert!(matches!(
            &signature.tracks[0].key,
            PresenceTrackKey::Animation { name, occurrence: 0 } if name == "fade"
        ));
        assert!(matches!(
            &signature.tracks[1].key,
            PresenceTrackKey::Animation { name, occurrence: 1 } if name == "fade"
        ));
        assert_eq!(signature.tracks[1].duration_ms, Some(200.0));
        assert_eq!(signature.tracks[1].iteration_count, Some(2.0));
        assert_eq!(
            signature.tracks[1].play_state,
            Some(super::PresencePlayState::Paused)
        );
    }

    #[test]
    fn invalid_and_infinite_animation_timing_is_conservative() {
        let invalid = signature(["none", "0s", "0s", "fade", "wat", "0s", "1", "running"]);
        assert_eq!(
            invalid.tracks[0].deadline_class,
            PresenceDeadlineClass::ConservativeUnknown
        );
        let infinite = signature([
            "none", "0s", "0s", "fade", "1ms", "0s", "infinite", "paused",
        ]);
        assert!(infinite.tracks[0].infinite_iterations);
        assert_eq!(
            infinite.tracks[0].deadline_class,
            PresenceDeadlineClass::EventOnlyAnimation
        );
    }

    #[test]
    fn zero_duration_with_positive_delay_remains_pending() {
        let signature = signature(["opacity", "0s", "50ms", "none", "0s", "0s", "1", "running"]);
        assert_eq!(presence_pending_tracks(&signature, 10.0).len(), 1);
        assert!(presence_pending_tracks(&signature, 60.0).is_empty());
    }

    #[test]
    fn exact_raw_signature_changes_are_observable() {
        let left = signature(["opacity", "1s", "0s", "none", "0s", "0s", "1", "running"]);
        let right = signature([
            "opacity", "1000ms", "0s", "none", "0s", "0s", "1", "running",
        ]);
        assert_ne!(left, right);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn native_presence_state_is_isolated_by_request_owner() {
        use leptos::html;
        use leptos::prelude::{Owner, RwSignal};

        std::thread::scope(|scope| {
            let requests = (0..16)
                .map(|index| {
                    scope.spawn(move || {
                        Owner::new().with(|| {
                            let present = RwSignal::new(index % 2 == 0);
                            let binding = super::use_presence::<html::Div>(present, None);
                            assert_eq!(binding.is_rendered(), index % 2 == 0);
                        });
                    })
                })
                .collect::<Vec<_>>();

            for request in requests {
                request.join().expect("request owner");
            }
        });
    }
}
