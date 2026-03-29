use pinocchio::{entrypoint, AccountView, Address, ProgramResult};

use crate::instruction::{initialize_config::process_initialize_config, SenshiInstruction};

pub mod error;
pub mod instruction;

entrypoint!(process_instruction);

pinocchio_pubkey::declare_id!("3vgVYgJxqFKF2cFYHV4GPBUnLynCJYmKizq9DRmZmTUf");

fn process_instruction(
    program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    // if *program_id != id() {
    //     return Err(ProgramError::IncorrectProgramId);
    // }

    let instruction = SenshiInstruction::unpack(instruction_data)?;

    match instruction {
        SenshiInstruction::InitializeConfig => {
            pinocchio_log::log!("Instruction: InitializeConfig");
            process_initialize_config(program_id, accounts)
        }
    }
}
