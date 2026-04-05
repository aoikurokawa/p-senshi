use pinocchio::error::ProgramError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SenshiError {
    #[error("Encountered an arithmetic under/overflow error.")]
    ArithmeticError,

    #[error("Season is not open for entries.")]
    SeasonNotOpen,

    #[error("Roster size does not match the season's required roster size.")]
    InvalidRosterSize,

    #[error("Duplicate validator in roster.")]
    DuplicateValidator,
}

impl From<SenshiError> for ProgramError {
    fn from(value: SenshiError) -> Self {
        Self::Custom(value as u32)
    }
}
