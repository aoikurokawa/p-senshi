use pinocchio::{
    error::ProgramError,
    sysvars::{clock::Clock, Sysvar},
    AccountView, Address,
};

use crate::{
    error::SenshiError,
    state::season::{Season, SeasonStatus},
};

/// Locks the season once the target epoch has begun, preventing new entries.
///
/// Only the season authority can invoke this. The season must be in `Open`
/// status and the current clock epoch must be >= `epoch_start`.
///
/// # Accounts
///
/// 0. `[writable]` Season PDA.
/// 1. `[signer]` Authority.
///
/// # Instruction Data (after tag byte)
///
/// | Offset | Size | Field       |
/// |--------|------|-------------|
/// | 0      | 8    | epoch_start |
pub fn process_lock_season(
    program_id: &Address,
    accounts: &[AccountView],
    data: &[u8],
) -> Result<(), ProgramError> {
    let [season_view, authority_view] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !authority_view.is_signer() {
        pinocchio_log::log!("authority is not a signer");
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Parse epoch_start from instruction data
    if data.len() < 8 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let epoch_start = u64::from_le_bytes(data[0..8].try_into().unwrap());

    // Verify season PDA
    let (season_pubkey, _, _) = Season::find_program_address(program_id, epoch_start);
    if season_pubkey.ne(season_view.address()) {
        pinocchio_log::log!("Season account is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }

    // Load season
    let season_data = unsafe { season_view.borrow_unchecked_mut() };
    if season_data[0..8] != *Season::DISCRIMINATOR {
        pinocchio_log::log!("Invalid season discriminator");
        return Err(ProgramError::InvalidAccountData);
    }
    let season = unsafe { Season::load_mut_unchecked(&mut season_data[8..])? };

    // Verify authority
    if authority_view.address().ne(&season.authority) {
        pinocchio_log::log!("Authority does not match season authority");
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Season must be Open
    if season.status != SeasonStatus::Open as u8 {
        pinocchio_log::log!("Season is not open");
        return Err(SenshiError::InvalidTransition.into());
    }

    // Current epoch must be >= epoch_start
    let clock = Clock::get()?;
    if clock.epoch < season.epoch_start {
        pinocchio_log::log!("Target epoch has not been reached");
        return Err(SenshiError::EpochNotReached.into());
    }

    season.status = SeasonStatus::Locked as u8;

    Ok(())
}
