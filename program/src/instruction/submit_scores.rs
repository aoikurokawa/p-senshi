use pinocchio::{error::ProgramError, AccountView, Address};

use crate::{
    error::SenshiError,
    state::{
        entry::Entry,
        season::{Season, SeasonStatus},
    },
};

/// Submits scores for a batch of entries in a locked or scoring season.
///
/// The authority calls this (potentially multiple times) with batches of
/// entry accounts passed via remaining accounts. The season transitions
/// to `Scoring` on the first call.
///
/// # Accounts
///
/// 0. `[writable]` Season PDA.
/// 1. `[signer]` Authority.
/// 2. .N. `[writable]` Entry PDAs (one per score).
///
/// # Instruction Data (after tag byte)
///
/// | Offset | Size    | Field       |
/// |--------|---------|-------------|
/// | 0      | 8       | epoch_start |
/// | 8      | 4       | count (N)   |
/// | 12     | N * 8   | scores      |
pub fn process_submit_scores(
    program_id: &Address,
    accounts: &[AccountView],
    data: &[u8],
) -> Result<(), ProgramError> {
    if accounts.len() < 2 {
        return Err(ProgramError::NotEnoughAccountKeys);
    }

    let season_view = &accounts[0];
    let authority_view = &accounts[1];
    let entry_views = &accounts[2..];

    if !authority_view.is_signer() {
        pinocchio_log::log!("authority is not a signer");
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Parse instruction data
    if data.len() < 12 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let epoch_start = u64::from_le_bytes(data[0..8].try_into().unwrap());
    let count = u32::from_le_bytes(data[8..12].try_into().unwrap()) as usize;

    if data.len() < 12 + count * 8 {
        return Err(ProgramError::InvalidInstructionData);
    }

    // Entry accounts must match count
    if entry_views.len() != count {
        pinocchio_log::log!("Entry account count does not match score count");
        return Err(SenshiError::LengthMismatch.into());
    }

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

    // Season must be Locked or Scoring
    if season.status != SeasonStatus::Locked as u8 && season.status != SeasonStatus::Scoring as u8 {
        pinocchio_log::log!("Season is not in Locked or Scoring status");
        return Err(SenshiError::InvalidTransition.into());
    }

    // Transition to Scoring
    season.status = SeasonStatus::Scoring as u8;

    // Write scores to each entry
    for i in 0..count {
        let score = u64::from_le_bytes(data[12 + i * 8..20 + i * 8].try_into().unwrap());

        let entry_data = unsafe { entry_views[i].borrow_unchecked_mut() };
        if entry_data[0..8] != *Entry::DISCRIMINATOR {
            pinocchio_log::log!("Invalid entry discriminator");
            return Err(ProgramError::InvalidAccountData);
        }
        let entry = unsafe { Entry::load_mut_unchecked(&mut entry_data[8..])? };

        // Verify this entry belongs to this season
        if entry.season_id != epoch_start {
            pinocchio_log::log!("Entry does not belong to this season");
            return Err(ProgramError::InvalidAccountData);
        }

        entry.has_score = 1;
        entry.score = score;
    }

    Ok(())
}
