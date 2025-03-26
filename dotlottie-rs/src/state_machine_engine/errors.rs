#[derive(Debug, thiserror::Error)]
pub enum StateMachineError {
    #[error("Failed to parse JSON state machine definition")]
    ParsingError { reason: String },
}
