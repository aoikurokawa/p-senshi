use pinocchio::{error::ProgramError, AccountView, Address};

pub fn process_initialize_config(
    _program_id: &Address,
    _accounts: &[AccountView],
) -> Result<(), ProgramError> {
    Ok(())
}
