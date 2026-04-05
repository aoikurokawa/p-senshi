use pinocchio::{entrypoint, error::ProgramError, AccountView, Address, ProgramResult};

use crate::instruction::{
    claim_reward::process_claim_reward, enter_season::process_enter_season,
    initialize_config::process_initialize_config, initialize_season::process_initialize_season,
    lock_season::process_lock_season, settle_season::process_settle_season,
    submit_scores::process_submit_scores, SenshiInstruction,
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
    if program_id.ne(&Address::new_from_array(id())) {
        return Err(ProgramError::IncorrectProgramId);
    }

    let instruction = SenshiInstruction::unpack(instruction_data)?;

    match instruction {
        SenshiInstruction::InitializeConfig => {
            pinocchio_log::log!("Instruction: InitializeConfig");
            process_initialize_config(program_id, accounts)
        }
        SenshiInstruction::InitializeSeason {
            entry_fee,
            roster_size,
            epoch_start,
            epoch_end,
        } => {
            pinocchio_log::log!("Instruction: InitializeSeason");
            process_initialize_season(
                program_id,
                accounts,
                entry_fee,
                roster_size,
                epoch_start,
                epoch_end,
            )
        }
        SenshiInstruction::EnterSeason { epoch_start } => {
            pinocchio_log::log!("Instruction: EnterSeason");
            process_enter_season(program_id, accounts, epoch_start)
        }
        SenshiInstruction::LockSeason { epoch_start } => {
            pinocchio_log::log!("Instruction: LockSeason");
            process_lock_season(program_id, accounts, epoch_start)
        }
        SenshiInstruction::SubmitScores {
            epoch_start,
            score,
        } => {
            pinocchio_log::log!("Instruction: SubmitScores");
            process_submit_scores(program_id, accounts, epoch_start, score)
        }
        SenshiInstruction::SettleSeason { epoch_start } => {
            pinocchio_log::log!("Instruction: SettleSeason");
            process_settle_season(program_id, accounts, epoch_start)
        }
        SenshiInstruction::ClaimReward { epoch_start } => {
            pinocchio_log::log!("Instruction: ClaimReward");
            process_claim_reward(program_id, accounts, epoch_start)
        }
    }
}
