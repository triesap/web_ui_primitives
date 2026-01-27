use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transition<S, E> {
    pub from: S,
    pub event: E,
    pub to: S,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionResult<S> {
    pub previous: S,
    pub next: S,
}

#[derive(Debug, Clone)]
pub struct StateMachine<S, E> {
    state: S,
    transitions: Vec<Transition<S, E>>,
}

impl<S, E> StateMachine<S, E> {
    pub fn new(initial: S) -> Self {
        Self {
            state: initial,
            transitions: Vec::new(),
        }
    }

    pub fn state(&self) -> &S {
        &self.state
    }

    pub fn set_state(&mut self, state: S) {
        self.state = state;
    }

    pub fn transitions(&self) -> &[Transition<S, E>] {
        &self.transitions
    }
}

impl<S: PartialEq, E: PartialEq> StateMachine<S, E> {
    pub fn add_transition(&mut self, from: S, event: E, to: S) {
        self.transitions.push(Transition { from, event, to });
    }

    pub fn can_transition(&self, event: &E) -> bool {
        self.transitions
            .iter()
            .any(|transition| transition.from == self.state && &transition.event == event)
    }
}

impl<S: PartialEq + Clone, E: PartialEq> StateMachine<S, E> {
    pub fn trigger(&mut self, event: &E) -> Option<TransitionResult<S>> {
        let transition = self.transitions.iter().find(|transition| {
            transition.from == self.state && &transition.event == event
        })?;

        let previous = core::mem::replace(&mut self.state, transition.to.clone());
        Some(TransitionResult {
            previous,
            next: self.state.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{StateMachine, TransitionResult};

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum DoorState {
        Open,
        Closed,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum DoorEvent {
        Open,
        Close,
    }

    #[test]
    fn state_machine_transitions() {
        let mut machine = StateMachine::new(DoorState::Closed);
        machine.add_transition(DoorState::Closed, DoorEvent::Open, DoorState::Open);
        machine.add_transition(DoorState::Open, DoorEvent::Close, DoorState::Closed);

        let result = machine.trigger(&DoorEvent::Open);
        assert_eq!(
            result,
            Some(TransitionResult {
                previous: DoorState::Closed,
                next: DoorState::Open,
            })
        );

        assert!(machine.can_transition(&DoorEvent::Close));
    }

    #[test]
    fn state_machine_rejects_invalid_transition() {
        let mut machine = StateMachine::new(DoorState::Closed);
        machine.add_transition(DoorState::Closed, DoorEvent::Open, DoorState::Open);

        assert!(machine.trigger(&DoorEvent::Close).is_none());
        assert!(!machine.can_transition(&DoorEvent::Close));
    }
}
