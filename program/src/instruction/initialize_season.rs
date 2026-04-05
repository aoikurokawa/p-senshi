use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{rent::Rent, Sysvar},
    AccountView, Address,
};
use pinocchio_system::instructions::CreateAccount;

use crate::{error::SenshiError, state::season::Season};

pub fn process_initialize_season(
    program_id: &Address,
    accounts: &[AccountView],
) -> Result<(), ProgramError> {
    let [season_view, payer_view, system_program_view] = accounts else {
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

    let (season_pubkey, season_bump, mut season_seeds) = Season::find_program_address(program_id);
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

    Ok(())
}
