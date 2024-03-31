// program logic

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

use crate::{error::RoshamboError, instruction::RoshamboInstruction, state::Game};

pub struct Processor;
impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = RoshamboInstruction::unpack(instruction_data)?;

        match instruction {
            RoshamboInstruction::NewGame {} => {
                msg!("Instruction: NewGame");
                Self::process_new_game(accounts, program_id)
            }
            RoshamboInstruction::EndGame { result } => {
                msg!("Instruction: EndGame");
                Self::process_end_game(accounts, result, program_id)
            }
        }
    }

    fn process_new_game(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let game_creator = next_account_info(account_info_iter)?;
        if !game_creator.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let temp_token_account = next_account_info(account_info_iter)?;

        // Game Account (store game info data) -> Make sure fee exempt
        let game_account = next_account_info(account_info_iter)?;
        let rent = Rent::get()?;
        if !rent.is_exempt(game_account.lamports(), game_account.data_len()) {
            return Err(RoshamboError::NotRentExempt.into());
        }

        // Check if this game account is already initialize
        let mut game_info = Game::unpack_unchecked(&game_account.try_borrow_data()?)?;
        if game_info.is_initialized() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        // Update game account with new game data
        game_info.is_initialized = true;
        game_info.game_creator_pubkey = *game_creator.key;
        game_info.temp_token_account_pubkey = *temp_token_account.key;
        Game::pack(game_info, &mut game_account.try_borrow_mut_data()?)?;

        // just need 1 PDA that can own N temporary token accounts
        let (pda, _bump_seed) = Pubkey::find_program_address(&[b"roshambo"], program_id);

        // CPI call token program transfer owner ship (user space) from game_creator to PDA
        let token_program = next_account_info(account_info_iter)?;
        let owner_change_ix = spl_token::instruction::set_authority(
            token_program.key,
            temp_token_account.key,
            Some(&pda),
            spl_token::instruction::AuthorityType::AccountOwner,
            game_creator.key,
            &[&game_creator.key],
        )?;

        msg!("Calling the token program to transfer token account ownership...");
        invoke(
            &owner_change_ix,
            &[
                temp_token_account.clone(),
                game_creator.clone(),
                token_program.clone(),
            ],
        )?;

        Ok(())
    }

    fn process_end_game(
        accounts: &[AccountInfo],
        result: u8,
        program_id: &Pubkey,
    ) -> ProgramResult {
        Ok(())
    }
}
