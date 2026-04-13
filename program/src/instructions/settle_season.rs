use pinocchio::{
    error::ProgramError,
    sysvars::{clock::Clock, Sysvar},
    AccountView, Address,
};

use crate::{
    error::SenshiError,
    states::season::{Season, SeasonStatus},
};

/// Settles the season after scoring is complete and the epoch window has ended.
///
/// Updates the prize pool to the current vault balance and transitions the
/// season to `Settled`, enabling reward claims.
///
/// # Accounts
///
/// 0. `[writable]` Season PDA.
/// 1. `[signer]` Authority.
/// 2. `[]` Vote account.
/// 3. `[]` Vault token account.
///
/// # Instruction Data (after tag byte)
///
/// | Offset | Size | Field       |
/// |--------|------|-------------|
/// | 0      | 8    | epoch_start |
pub fn process_settle_season(
    program_id: &Address,
    accounts: &[AccountView],
    epoch_start: u64,
) -> Result<(), ProgramError> {
    let [season_view, authority_view, vote_account_view, vault_view] = accounts else {
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

    // Verify vault matches season
    if vault_view.address().ne(&season.vault) {
        pinocchio_log::log!("Vault does not match season vault");
        return Err(ProgramError::InvalidAccountData);
    }

    // Season must be Scoring
    if season.status != SeasonStatus::Scoring as u8 {
        pinocchio_log::log!("Season is not in Scoring status");
        return Err(SenshiError::InvalidTransition.into());
    }

    // Current epoch must be past epoch_end
    let clock = Clock::get()?;
    if clock.epoch <= season.epoch_end {
        pinocchio_log::log!("Season epoch has not ended");
        return Err(SenshiError::EpochNotEnded.into());
    }

    // Update prize pool from vault balance (includes accrued yield)
    let vault_data = vault_view.try_borrow()?;
    // SPL Token account: amount is at offset 64, 8 bytes little-endian
    let vault_amount = u64::from_le_bytes(vault_data[64..72].try_into().unwrap());
    season.prize_pool = vault_amount;

    season.status = SeasonStatus::Settled as u8;

    Ok(())
}
