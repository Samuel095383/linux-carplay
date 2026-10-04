use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Idle,
    Discovering,
    Pairing,
    Negotiating,
    Authenticating,
    ChannelSetup,
    Streaming,
    Reconnecting,
    Stopping,
    Stopped,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionEvent {
    Start,
    DeviceDiscovered,
    PairingComplete,
    NegotiationSucceeded,
    AuthenticationSucceeded,
    ChannelsReady,
    LinkLost,
    ReconnectSucceeded,
    ReconnectFailed,
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
            (SessionState::Discovering, SessionEvent::DeviceDiscovered) => SessionState::Pairing,
            (SessionState::Pairing, SessionEvent::PairingComplete) => SessionState::Negotiating,
            (SessionState::Negotiating, SessionEvent::NegotiationSucceeded) => {
                SessionState::Authenticating
            }
            (SessionState::Authenticating, SessionEvent::AuthenticationSucceeded) => {
                SessionState::ChannelSetup
            }
            (SessionState::ChannelSetup, SessionEvent::ChannelsReady) => SessionState::Streaming,
            (SessionState::Streaming, SessionEvent::LinkLost) => SessionState::Reconnecting,
            (SessionState::Reconnecting, SessionEvent::ReconnectSucceeded) => SessionState::Streaming,
            (SessionState::Reconnecting, SessionEvent::ReconnectFailed) => SessionState::Stopping,
            (SessionState::Streaming, SessionEvent::Stop)
            | (SessionState::Discovering, SessionEvent::Stop)
            | (SessionState::Pairing, SessionEvent::Stop)
            | (SessionState::Negotiating, SessionEvent::Stop)
            | (SessionState::Authenticating, SessionEvent::Stop)
            | (SessionState::ChannelSetup, SessionEvent::Stop)
            | (SessionState::Reconnecting, SessionEvent::Stop) => SessionState::Stopping,
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
        assert_eq!(session.transition(SessionEvent::Start), Ok(SessionState::Discovering));
        assert_eq!(
            session.transition(SessionEvent::DeviceDiscovered),
            Ok(SessionState::Pairing)
        );
        assert_eq!(
            session.transition(SessionEvent::PairingComplete),
            Ok(SessionState::Negotiating)
        );
        assert_eq!(
            session.transition(SessionEvent::NegotiationSucceeded),
            Ok(SessionState::Authenticating)
        );
        assert_eq!(
            session.transition(SessionEvent::AuthenticationSucceeded),
            Ok(SessionState::ChannelSetup)
        );
        assert_eq!(
            session.transition(SessionEvent::ChannelsReady),
            Ok(SessionState::Streaming)
        );
    }

    #[test]
    fn reconnect_flow_returns_to_streaming() {
        let mut session = Session::new();
        for event in [
            SessionEvent::Start,
            SessionEvent::DeviceDiscovered,
            SessionEvent::PairingComplete,
            SessionEvent::NegotiationSucceeded,
            SessionEvent::AuthenticationSucceeded,
            SessionEvent::ChannelsReady,
        ] {
            session.transition(event).expect("transition to streaming");
        }

        assert_eq!(session.transition(SessionEvent::LinkLost), Ok(SessionState::Reconnecting));
        assert_eq!(
            session.transition(SessionEvent::ReconnectSucceeded),
            Ok(SessionState::Streaming)
        );
    }

    #[test]
    fn reconnect_failure_moves_to_stopping() {
        let mut session = Session::new();
        for event in [
            SessionEvent::Start,
            SessionEvent::DeviceDiscovered,
            SessionEvent::PairingComplete,
            SessionEvent::NegotiationSucceeded,
            SessionEvent::AuthenticationSucceeded,
            SessionEvent::ChannelsReady,
            SessionEvent::LinkLost,
        ] {
            session.transition(event).expect("transition to reconnecting");
        }

        assert_eq!(
            session.transition(SessionEvent::ReconnectFailed),
            Ok(SessionState::Stopping)
        );
    }

    #[test]
    fn invalid_transition_is_rejected() {
        let mut session = Session::new();
        let result = session.transition(SessionEvent::AuthenticationSucceeded);
        assert!(result.is_err());
    }

    #[test]
    fn error_event_moves_to_failed_from_any_state() {
        let mut session = Session::new();
        assert_eq!(session.transition(SessionEvent::Error), Ok(SessionState::Failed));
    }
}
