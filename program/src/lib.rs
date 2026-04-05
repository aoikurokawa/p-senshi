use pinocchio::{entrypoint, AccountView, Address, ProgramResult};

use crate::instruction::{
    enter_season::process_enter_season, initialize_config::process_initialize_config,
    initialize_season::process_initialize_season, lock_season::process_lock_season,
    settle_season::process_settle_season, submit_scores::process_submit_scores,
    SenshiInstruction,
};

pub mod error;
pub mod instruction;
pub mod state;

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
        SenshiInstruction::InitializeSeason => {
            pinocchio_log::log!("Instruction: InitializeSeason");
            process_initialize_season(program_id, accounts, &instruction_data[1..])
        }
        SenshiInstruction::EnterSeason => {
            pinocchio_log::log!("Instruction: EnterSeason");
            process_enter_season(program_id, accounts, &instruction_data[1..])
        }
        SenshiInstruction::LockSeason => {
            pinocchio_log::log!("Instruction: LockSeason");
            process_lock_season(program_id, accounts, &instruction_data[1..])
        }
        SenshiInstruction::SubmitScores => {
            pinocchio_log::log!("Instruction: SubmitScores");
            process_submit_scores(program_id, accounts, &instruction_data[1..])
        }
        SenshiInstruction::SettleSeason => {
            pinocchio_log::log!("Instruction: SettleSeason");
            process_settle_season(program_id, accounts, &instruction_data[1..])
        }
    }
}
