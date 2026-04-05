use pinocchio::{error::ProgramError, AccountView, Address};

use crate::{
    error::SenshiError,
    state::{
        entry::Entry,
        season::{Season, SeasonStatus},
    },
};

/// Submits a score for a single entry in a locked or scoring season.
///
/// The authority calls this once per entry. The season transitions
/// to `Scoring` on the first call.
///
/// # Accounts
///
/// 0. `[writable]` Season PDA.
/// 1. `[signer]` Authority.
/// 2. `[]` Vote account.
/// 3. `[writable]` Entry PDA.
///
/// # Instruction Data (after tag byte)
///
/// | Offset | Size | Field       |
/// |--------|------|-------------|
/// | 0      | 8    | epoch_start |
/// | 8      | 8    | score       |
pub fn process_submit_scores(
    program_id: &Address,
    accounts: &[AccountView],
    epoch_start: u64,
    score: u64,
) -> Result<(), ProgramError> {
    let [season_view, authority_view, vote_account_view, entry_view] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !authority_view.is_signer() {
        pinocchio_log::log!("authority is not a signer");
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Verify season PDA
    let (season_pubkey, _, _) =
        Season::find_program_address(program_id, vote_account_view.address(), epoch_start);
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

    // Load entry and write score
    let entry_data = unsafe { entry_view.borrow_unchecked_mut() };
    if entry_data[0..8] != *Entry::DISCRIMINATOR {
        pinocchio_log::log!("Invalid entry discriminator");
        return Err(ProgramError::InvalidAccountData);
    }
    let entry = unsafe { Entry::load_mut_unchecked(&mut entry_data[8..])? };

    entry.has_score = 1;
    entry.score = score;

    Ok(())
}
