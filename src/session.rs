use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Idle,
    Discovering,
    Negotiating,
    Streaming,
    Stopping,
    Stopped,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionEvent {
    Start,
    DeviceDiscovered,
    NegotiationSucceeded,
    Stop,
    TeardownComplete,
    Error,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SessionError {
    #[error("invalid event {event:?} for state {state:?}")]
    InvalidTransition {
        state: SessionState,
        event: SessionEvent,
    },
}

#[derive(Debug)]
pub struct Session {
    state: SessionState,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            state: SessionState::Idle,
        }
    }
}

impl Session {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn state(&self) -> SessionState {
        self.state
    }

    pub fn transition(&mut self, event: SessionEvent) -> Result<SessionState, SessionError> {
        let next = match (self.state, event) {
            (SessionState::Idle, SessionEvent::Start) => SessionState::Discovering,
            (SessionState::Discovering, SessionEvent::DeviceDiscovered) => {
                SessionState::Negotiating
            }
            (SessionState::Negotiating, SessionEvent::NegotiationSucceeded) => {
                SessionState::Streaming
            }
            (SessionState::Streaming, SessionEvent::Stop)
            | (SessionState::Negotiating, SessionEvent::Stop)
            | (SessionState::Discovering, SessionEvent::Stop) => SessionState::Stopping,
            (SessionState::Stopping, SessionEvent::TeardownComplete) => SessionState::Stopped,
            (SessionState::Stopped, SessionEvent::Start) => SessionState::Discovering,
            (_, SessionEvent::Error) => SessionState::Failed,
            (state, event) => {
                return Err(SessionError::InvalidTransition { state, event });
            }
        };

        self.state = next;
        Ok(next)
    }
}

#[cfg(test)]
mod tests {
    use super::{Session, SessionEvent, SessionState};

    #[test]
    fn happy_path_transitions_to_streaming() {
        let mut session = Session::new();
        assert_eq!(
            session.transition(SessionEvent::Start),
            Ok(SessionState::Discovering)
        );
        assert_eq!(
            session.transition(SessionEvent::DeviceDiscovered),
            Ok(SessionState::Negotiating)
        );
        assert_eq!(
            session.transition(SessionEvent::NegotiationSucceeded),
            Ok(SessionState::Streaming)
        );
    }

    #[test]
    fn stop_flow_transitions_to_stopped() {
        let mut session = Session::new();
        session.transition(SessionEvent::Start).expect("start");
        session
            .transition(SessionEvent::DeviceDiscovered)
            .expect("device discovered");
        session.transition(SessionEvent::Stop).expect("stop");
        assert_eq!(session.state(), SessionState::Stopping);
        session
            .transition(SessionEvent::TeardownComplete)
            .expect("teardown complete");
        assert_eq!(session.state(), SessionState::Stopped);
    }

    #[test]
    fn invalid_transition_is_rejected() {
        let mut session = Session::new();
        let result = session.transition(SessionEvent::NegotiationSucceeded);
        assert!(result.is_err());
    }

    #[test]
    fn error_event_moves_to_failed_from_any_state() {
        let mut session = Session::new();
        assert_eq!(
            session.transition(SessionEvent::Error),
            Ok(SessionState::Failed)
        );
    }
}
