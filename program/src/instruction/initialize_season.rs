use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{rent::Rent, Sysvar},
    AccountView, Address,
};
use pinocchio_system::instructions::CreateAccount;

use crate::{
    error::SenshiError,
    state::season::{Season, SeasonStatus},
};

/// Creates the global [`Season`] PDA and populates it with the provided parameters.
///
/// # Accounts
///
/// 0. `[writable]` Season PDA (derived from `["season", validator, epoch]`).
/// 1. `[signer, writable]` Payer — becomes the season authority.
/// 2. `[]` Vote account.
/// 3. `[]` Vault token account.
/// 4. `[]` System program.
///
/// # Instruction Data (after tag byte)
///
/// | Offset | Size | Field        |
/// |--------|------|--------------|
/// | 0      | 8    | entry_fee    |
/// | 8      | 8    | epoch_start  |
/// | 16     | 8    | epoch_end    |
pub fn process_initialize_season(
    program_id: &Address,
    accounts: &[AccountView],
    entry_fee: u64,
    epoch_start: u64,
    epoch_end: u64,
) -> Result<(), ProgramError> {
    let [season_view, payer_view, vote_account_view, vault_view, system_program_view] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !payer_view.is_signer() {
        pinocchio_log::log!("payer_view is not a signer");
        return Err(ProgramError::MissingRequiredSignature);
    }

    if system_program_view.address().ne(&pinocchio_system::id()) {
        pinocchio_log::log!("Account is not the system program");
        return Err(ProgramError::IncorrectProgramId);
    }

    let rent = Rent::get()?;
    let space = 8usize
        .checked_add(Season::LEN)
        .ok_or(SenshiError::ArithmeticError)?;

    let (season_pubkey, season_bump, mut season_seeds) =
        Season::find_program_address(program_id, vote_account_view.address(), epoch_start);
    season_seeds.push(vec![season_bump]);
    if season_pubkey.ne(season_view.address()) {
        pinocchio_log::log!("Season account is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }

    let seeds: Vec<Seed> = season_seeds
        .iter()
        .map(|seed| Seed::from(seed.as_slice()))
        .collect();
    let signers = [Signer::from(seeds.as_slice())];

    pinocchio_log::log!("Initializing Season at address");
    CreateAccount {
        from: payer_view,
        to: season_view,
        lamports: rent.minimum_balance_unchecked(space),
        space: space as u64,
        owner: program_id,
    }
    .invoke_signed(&signers)?;

    let season = unsafe {
        let season_data = season_view.borrow_unchecked_mut();
        season_data[0..8].copy_from_slice(Season::DISCRIMINATOR);
        Season::load_mut_unchecked(&mut season_data[8..])?
    };

    season.authority = payer_view.address().clone();
    season.vault = vault_view.address().clone();
    season.entry_fee = entry_fee;
    season.status = SeasonStatus::Open as u8;
    season.epoch_start = epoch_start;
    season.epoch_end = epoch_end;
    season.total_entries = 0;
    season.prize_pool = 0;
    season.bump = season_bump;
    season.reserved = [0u8; 128];

    Ok(())
}
