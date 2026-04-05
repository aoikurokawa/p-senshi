use pinocchio::error::ProgramError;

pub mod enter_season;
pub mod initialize_config;
pub mod initialize_season;
pub mod lock_season;
pub mod settle_season;
pub mod submit_scores;

#[derive(Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub enum SenshiInstruction {
    /// Initialize config
    InitializeConfig,

    /// Initialize season
    InitializeSeason,

    /// Enter a season
    EnterSeason,

    /// Lock a season
    LockSeason,

    /// Submit scores for entries
    SubmitScores,

    /// Settle a season
    SettleSeason,
}

impl SenshiInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, _rest) = input
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;

        Ok(match tag {
            0 => SenshiInstruction::InitializeConfig,
            1 => SenshiInstruction::InitializeSeason,
            2 => SenshiInstruction::EnterSeason,
            3 => SenshiInstruction::LockSeason,
            4 => SenshiInstruction::SubmitScores,
            5 => SenshiInstruction::SettleSeason,
            _ => return Err(ProgramError::InvalidInstructionData),
        })
    }
}
