use pinocchio::error::ProgramError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SenshiError {
    #[error("Encountered an arithmetic under/overflow error.")]
    ArithmeticError,
}

impl From<SenshiError> for ProgramError {
    fn from(value: SenshiError) -> Self {
        Self::Custom(value as u32)
    }
}
