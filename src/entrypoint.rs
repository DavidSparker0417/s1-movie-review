use solana_program::{
    account_info::AccountInfo, entrypoint,
    entrypoint::ProgramResult, msg, pubkey::Pubkey,
};

use crate::processor;

entrypoint!(process_instruction);

fn process_instruction(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    msg!("[DAVID] Program started.");
    processor::process_instruction(program_id, accounts, data);
    Ok(())
}
