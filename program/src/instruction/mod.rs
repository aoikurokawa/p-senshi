use pinocchio::error::ProgramError;

pub mod initialize_config;
pub mod initialize_season;

#[derive(Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub enum SenshiInstruction {
    /// Initialize config
    InitializeConfig,

    /// Initialize season
    InitializeSeason,
}

impl SenshiInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, _rest) = input
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;

        Ok(match tag {
            0 => SenshiInstruction::InitializeConfig,
            1 => SenshiInstruction::InitializeSeason,
            _ => return Err(ProgramError::InvalidInstructionData),
        })
    }
}
