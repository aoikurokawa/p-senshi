use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{rent::Rent, Sysvar},
    AccountView, Address,
};
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::instructions::Transfer;

use crate::{
    error::SenshiError,
    state::{
        entry::Entry,
        season::{Season, SeasonStatus},
    },
};

/// Enters a player into an open season with the given validator roster.
///
/// Creates the [`Entry`] PDA, transfers the entry fee from the player's token
/// account to the season vault, and increments the season's `total_entries`
/// and `prize_pool`.
///
/// # Accounts
///
/// 0. `[writable]` Season PDA.
/// 1. `[writable]` Entry PDA (derived from `["entry", epoch_start, player]`).
/// 2. `[signer, writable]` Player.
/// 3. `[writable]` Player's JitoSOL token account.
/// 4. `[writable]` Season vault token account.
/// 5. `[]` Token program.
/// 6. `[]` System program.
///
/// # Instruction Data (after tag byte)
///
/// | Offset | Size     | Field       |
/// |--------|----------|-------------|
/// | 0      | 8        | epoch_start |
/// | 8      | N * 32   | validators  |
pub fn process_enter_season(
    program_id: &Address,
    accounts: &[AccountView],
    data: &[u8],
) -> Result<(), ProgramError> {
    let [season_view, entry_view, player_view, player_token_view, vault_view, token_program_view, system_program_view] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !player_view.is_signer() {
        pinocchio_log::log!("player is not a signer");
        return Err(ProgramError::MissingRequiredSignature);
    }

    if system_program_view.address().ne(&pinocchio_system::id()) {
        pinocchio_log::log!("Account is not the system program");
        return Err(ProgramError::IncorrectProgramId);
    }

    if token_program_view.address().ne(&pinocchio_token::id()) {
        pinocchio_log::log!("Account is not the token program");
        return Err(ProgramError::IncorrectProgramId);
    }

    // Parse instruction data
    if data.len() < 8 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let epoch_start = u64::from_le_bytes(data[0..8].try_into().unwrap());
    let validators_data = &data[8..];

    // Load season
    let season_data = unsafe { season_view.borrow_unchecked_mut() };
    if season_data[0..8] != *Season::DISCRIMINATOR {
        pinocchio_log::log!("Invalid season discriminator");
        return Err(ProgramError::InvalidAccountData);
    }
    let season = unsafe { Season::load_mut_unchecked(&mut season_data[8..])? };

    // Verify season PDA
    let (season_pubkey, _, _) = Season::find_program_address(program_id, epoch_start);
    if season_pubkey.ne(season_view.address()) {
        pinocchio_log::log!("Season account is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }

    // Season must be open
    if season.status != SeasonStatus::Open as u8 {
        pinocchio_log::log!("Season is not open");
        return Err(SenshiError::SeasonNotOpen.into());
    }

    // Validate roster size
    let roster_size = season.roster_size as usize;
    if validators_data.len() != roster_size * 32 {
        pinocchio_log::log!("Invalid roster size");
        return Err(SenshiError::InvalidRosterSize.into());
    }

    // Parse validators
    let mut validators: [Address; 10] = core::array::from_fn(|_| Address::default());
    for (i, validator) in validators.iter_mut().enumerate().take(roster_size) {
        let start = i * 32;
        let bytes: [u8; 32] = validators_data[start..start + 32].try_into().unwrap();
        *validator = Address::from(bytes).clone();
    }

    // Check for duplicate validators
    for i in 0..roster_size {
        for j in (i + 1)..roster_size {
            if validators[i] == validators[j] {
                pinocchio_log::log!("Duplicate validator in roster");
                return Err(SenshiError::DuplicateValidator.into());
            }
        }
    }

    // Verify vault matches season
    if vault_view.address().ne(&season.vault) {
        pinocchio_log::log!("Vault does not match season vault");
        return Err(ProgramError::InvalidAccountData);
    }

    // Transfer entry fee from player's token account to vault
    Transfer {
        from: player_token_view,
        to: vault_view,
        authority: player_view,
        amount: season.entry_fee,
    }
    .invoke()?;

    // Create Entry PDA
    let (entry_pubkey, entry_bump, mut entry_seeds) =
        Entry::find_program_address(program_id, epoch_start, player_view.address());
    entry_seeds.push(vec![entry_bump]);
    if entry_pubkey.ne(entry_view.address()) {
        pinocchio_log::log!("Entry account is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }

    let rent = Rent::get()?;
    let space = 8usize
        .checked_add(Entry::LEN)
        .ok_or(SenshiError::ArithmeticError)?;

    let seeds: Vec<Seed> = entry_seeds
        .iter()
        .map(|seed| Seed::from(seed.as_slice()))
        .collect();
    let signers = [Signer::from(seeds.as_slice())];

    pinocchio_log::log!("Creating Entry account");
    CreateAccount {
        from: player_view,
        to: entry_view,
        lamports: rent.minimum_balance_unchecked(space),
        space: space as u64,
        owner: program_id,
    }
    .invoke_signed(&signers)?;

    // Initialize entry
    let entry = unsafe {
        let entry_data = entry_view.borrow_unchecked_mut();
        entry_data[0..8].copy_from_slice(Entry::DISCRIMINATOR);
        Entry::load_mut_unchecked(&mut entry_data[8..])?
    };

    entry.season_id = epoch_start;
    entry.player = player_view.address().clone();
    entry.validators = validators;
    entry.has_score = 0;
    entry.score = 0;
    entry.has_reward = 0;
    entry.reward = 0;
    entry.claimed = 0;
    entry.bump = entry_bump;
    entry.reserved = [0u8; 64];

    // Update season
    season.total_entries = season
        .total_entries
        .checked_add(1)
        .ok_or(SenshiError::ArithmeticError)?;
    season.prize_pool = season
        .prize_pool
        .checked_add(season.entry_fee)
        .ok_or(SenshiError::ArithmeticError)?;

    Ok(())
}
