use pinocchio::error::ProgramError;

pub mod claim_reward;
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
    InitializeSeason {
        /// Entry fee
        entry_fee: u64,

        /// Roster size
        roster_size: u8,

        /// Epoch start
        epoch_start: u64,

        /// Epoch end
        epoch_end: u64,
    },

    /// Enter a season
    EnterSeason {
        /// Epoch start
        epoch_start: u64,
    },

    /// Lock a season
    LockSeason {
        /// Epoch start
        epoch_start: u64,
    },

    /// Submit score for a single entry
    SubmitScores {
        /// Epoch start
        epoch_start: u64,

        /// Score
        score: u64,
    },

    /// Settle a season
    SettleSeason {
        /// Epoch start
        epoch_start: u64,
    },

    /// Claim reward
    ClaimReward {
        /// Epoch start
        epoch_start: u64,
    },
}

impl SenshiInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;

        Ok(match tag {
            0 => SenshiInstruction::InitializeConfig,
            1 => {
                // Parse instruction data: entry_fee (8) + roster_size (1) + epoch_start (8) + epoch_end (8) + vault (32) = 57
                if rest.len() < 57 {
                    return Err(ProgramError::InvalidInstructionData);
                }

                let entry_fee = u64::from_le_bytes(rest[0..8].try_into().unwrap());
                let roster_size = rest[8];
                let epoch_start = u64::from_le_bytes(rest[9..17].try_into().unwrap());
                let epoch_end = u64::from_le_bytes(rest[17..25].try_into().unwrap());

                SenshiInstruction::InitializeSeason {
                    entry_fee,
                    roster_size,
                    epoch_start,
                    epoch_end,
                }
            }
            2 => {
                // Parse instruction data
                if rest.len() < 8 {
                    return Err(ProgramError::InvalidInstructionData);
                }
                let epoch_start = u64::from_le_bytes(rest[0..8].try_into().unwrap());

                SenshiInstruction::EnterSeason { epoch_start }
            }
            3 => {
                // Parse epoch_start from instruction data
                if rest.len() < 8 {
                    return Err(ProgramError::InvalidInstructionData);
                }
                let epoch_start = u64::from_le_bytes(rest[0..8].try_into().unwrap());

                SenshiInstruction::LockSeason { epoch_start }
            }
            4 => {
                if rest.len() < 16 {
                    return Err(ProgramError::InvalidInstructionData);
                }
                let epoch_start = u64::from_le_bytes(rest[0..8].try_into().unwrap());
                let score = u64::from_le_bytes(rest[8..16].try_into().unwrap());

                SenshiInstruction::SubmitScores { epoch_start, score }
            }
            5 => {
                // Parse epoch_start from instruction data
                if rest.len() < 8 {
                    return Err(ProgramError::InvalidInstructionData);
                }
                let epoch_start = u64::from_le_bytes(rest[0..8].try_into().unwrap());

                SenshiInstruction::SettleSeason { epoch_start }
            }
            6 => {
                // Parse epoch_start
                if rest.len() < 8 {
                    return Err(ProgramError::InvalidInstructionData);
                }
                let epoch_start = u64::from_le_bytes(rest[0..8].try_into().unwrap());

                SenshiInstruction::ClaimReward { epoch_start }
            }
            _ => return Err(ProgramError::InvalidInstructionData),
        })
    }
}
