use leptos::ev::{AnimationEvent, TransitionEvent};
use leptos::html;
use leptos::prelude::*;
use std::sync::Arc;
#[cfg(target_arch = "wasm32")]
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresenceState {
    Mounted,
    Exiting,
    Unmounted,
}

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

#[component]
pub fn Presence(
    #[prop(into)] present: Signal<bool>,
    #[prop(optional)] on_exit_complete: Option<Callback<()>>,
    children: ChildrenFn,
) -> impl IntoView {
    let node_ref = NodeRef::<html::Div>::new();
    let state = RwSignal::new(if present.get_untracked() {
        PresenceState::Mounted
    } else {
        PresenceState::Unmounted
    });
    #[cfg(target_arch = "wasm32")]
    let root_element = Arc::new(Mutex::new(
        None::<send_wrapper::SendWrapper<web_sys::Element>>,
    ));
    #[cfg(target_arch = "wasm32")]
    let exit_timeout = Arc::new(Mutex::new(None::<PresenceExitTimeout>));

    let on_exit_complete = on_exit_complete.clone();
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;

        let root_element_handle = Arc::clone(&root_element);
        let exit_timeout_handle = Arc::clone(&exit_timeout);
        node_ref.on_load(move |root| {
            if let Ok(element) = root.dyn_into::<web_sys::Element>() {
                *root_element_handle
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner()) =
                    Some(send_wrapper::SendWrapper::new(element));
            }
            let root_element_handle = Arc::clone(&root_element_handle);
            let exit_timeout_handle = Arc::clone(&exit_timeout_handle);
            on_cleanup(move || {
                *root_element_handle
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner()) = None;
                presence_clear_exit_timeout(&exit_timeout_handle);
            });
        });
    }

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

    Effect::new({
        let end_handler = Arc::clone(&end_handler);
        #[cfg(target_arch = "wasm32")]
        let root_element = Arc::clone(&root_element);
        #[cfg(target_arch = "wasm32")]
        let exit_timeout = Arc::clone(&exit_timeout);

        move || {
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

            #[cfg(target_arch = "wasm32")]
            let exit_timeout_ms = root_element
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .as_ref()
                .map(|root| presence_exit_timeout_ms(&*root))
                .unwrap_or(0);
            #[cfg(not(target_arch = "wasm32"))]
            let exit_timeout_ms = 0;

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

    let render = move || -> AnyView {
        if state.get() == PresenceState::Unmounted {
            ().into_any()
        } else {
            let transition_end = {
                let end_handler = Arc::clone(&end_handler);
                move |event: TransitionEvent| {
                    if presence_should_complete_exit(
                        state.get_untracked(),
                        present.get_untracked(),
                        presence_event_matches_current_target(
                            event.target(),
                            event.current_target(),
                        ),
                    ) {
                        end_handler();
                    }
                }
            };
            let animation_end = {
                let end_handler = Arc::clone(&end_handler);
                move |event: AnimationEvent| {
                    if presence_should_complete_exit(
                        state.get_untracked(),
                        present.get_untracked(),
                        presence_event_matches_current_target(
                            event.target(),
                            event.current_target(),
                        ),
                    ) {
                        end_handler();
                    }
                }
            };
            view! {
                <div
                    node_ref=node_ref
                    data-state=move || if present.get() { "open" } else { "closed" }
                    on:transitionend=transition_end
                    on:animationend=animation_end
                >
                    {children()}
                </div>
            }
            .into_any()
        }
    };

    view! { {render} }
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
