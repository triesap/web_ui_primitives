use leptos::ev::{AnimationEvent, TransitionEvent};
use leptos::prelude::*;
use std::sync::Arc;

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

#[component]
pub fn Presence(
    #[prop(into)] present: Signal<bool>,
    #[prop(optional)] on_exit_complete: Option<Callback<()>>,
    children: ChildrenFn,
) -> impl IntoView {
    let state = RwSignal::new(if present.get_untracked() {
        PresenceState::Mounted
    } else {
        PresenceState::Unmounted
    });

    Effect::new(move || {
        let next = presence_state_next(state.get(), present.get(), false);
        if next != state.get() {
            state.set(next);
        }
    });

    let on_exit_complete = on_exit_complete.clone();
    let end_handler = Arc::new(move || {
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

    let render = move || -> AnyView {
        if state.get() == PresenceState::Unmounted {
            ().into_any()
        } else {
            let transition_end = {
                let end_handler = Arc::clone(&end_handler);
                move |_event: TransitionEvent| {
                    end_handler();
                }
            };
            let animation_end = {
                let end_handler = Arc::clone(&end_handler);
                move |_event: AnimationEvent| {
                    end_handler();
                }
            };
            view! {
                <div
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
    use super::{presence_state_next, PresenceState};

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
}
