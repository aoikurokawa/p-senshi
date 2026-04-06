use pinocchio::error::ProgramError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SenshiError {
    #[error("Encountered an arithmetic under/overflow error.")]
    ArithmeticError,

    #[error("Season is not open for entries.")]
    SeasonNotOpen,

    #[error("Invalid season status transition.")]
    InvalidTransition,

    #[error("Target epoch has not been reached yet.")]
    EpochNotReached,

    #[error("Season epoch has not ended yet.")]
    EpochNotEnded,

    #[error("Season is not settled.")]
    NotSettled,

    #[error("Reward has already been claimed.")]
    AlreadyClaimed,

    #[error("No reward assigned to this entry.")]
    NoReward,
}

impl From<SenshiError> for ProgramError {
    fn from(value: SenshiError) -> Self {
        Self::Custom(value as u32)
    }
}
