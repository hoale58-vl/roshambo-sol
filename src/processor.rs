// program logic

use crate::{error::RoshamboError, instruction::RoshamboInstruction, state::Game};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_option::COption,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};
use spl_token::state::Account as TokenAccount;

pub struct Processor;
impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = RoshamboInstruction::unpack(instruction_data)?;

        match instruction {
            RoshamboInstruction::NewGame { amount } => {
                msg!("Instruction: NewGame");
                Self::process_new_game(accounts, amount, program_id)
            }
            RoshamboInstruction::ClaimReward {
                host_seed,
                public_seed,
            } => {
                msg!("Instruction: Claim");
                Self::process_claim(accounts, host_seed, public_seed, program_id)
            }
        }
    }

    fn process_new_game(
        accounts: &[AccountInfo],
        amount: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let game_creator = next_account_info(account_info_iter)?;
        if !game_creator.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let creator_token_account = next_account_info(account_info_iter)?;

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
        game_info.bet_amount = amount;
        game_info.game_creator_pubkey = *game_creator.key;
        game_info.result = COption::None;
        Game::pack(game_info, &mut game_account.try_borrow_mut_data()?)?;

        let house_token_account = next_account_info(account_info_iter)?;
        // verify house token account's authority is PDA
        let house_token_account_info =
            TokenAccount::unpack(&house_token_account.try_borrow_data()?)?;
        // just need 1 PDA that can own N temporary token accounts
        let (pda, _nonce) = Pubkey::find_program_address(&[b"roshambo"], program_id);
        if pda != house_token_account_info.owner {
            return Err(ProgramError::InvalidAccountData);
        }

        // CPI call token program transfer bet amount to house PDA
        let token_program = next_account_info(account_info_iter)?;
        let owner_change_ix = spl_token::instruction::transfer(
            token_program.key,
            creator_token_account.key,
            house_token_account.key,
            &game_creator.key,
            &[&game_creator.key],
            amount,
        )?;

        msg!("Calling the token program to transfer token to house token account...");
        invoke(
            &owner_change_ix,
            &[
                creator_token_account.clone(),
                house_token_account.clone(),
                game_creator.clone(),
                token_program.clone(),
            ],
        )?;

        Ok(())
    }

    fn process_claim(
        accounts: &[AccountInfo],
        host_seed: u64,
        public_seed: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        // / 0. `[signer]` The account of the person owned the game
        // / 1. `[signer]` The account of the house verify the result of this game
        // / 3. `[writable]` The game account, it will hold all necessary info about the game (close after this and refund rent fee back to caller)
        // / 4. `[writable]` Creator's token account receive reward (double bet amount if win - or nothing if lose)
        // / 5. `[writable]` House token account owned by PDA (change based on game result)
        let game_creator = next_account_info(account_info_iter)?;
        let house_account = next_account_info(account_info_iter)?;

        if !game_creator.is_signer || !house_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let game_account = next_account_info(account_info_iter)?;
        // Check if this game account is already initialize and not ended
        let mut game_info = Game::unpack(&game_account.try_borrow_data()?)?;
        if game_info.game_creator_pubkey != *game_creator.key {
            return Err(ProgramError::InvalidAccountData);
        }
        if game_info.result.is_some() {
            return Err(RoshamboError::GameEnded.into());
        }

        // Check the result based on host_seed and public_seed
        let selection = public_seed % 5;
        let host_result = host_seed % 5;

        // just need 1 PDA that can own N temporary token accounts
        let (pda, nonce) = Pubkey::find_program_address(&[b"roshambo"], program_id);

        let receiver_account = next_account_info(account_info_iter)?;
        let house_token_account = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?;
        let pda_program = next_account_info(account_info_iter)?;

        // Draw
        if selection == host_result {
            game_info.result = COption::Some(2);
            // refund bet amount
            let owner_change_ix = spl_token::instruction::transfer(
                token_program.key,
                house_token_account.key,
                receiver_account.key,
                &pda,
                &[&pda],
                game_info.bet_amount,
            )?;

            msg!("Refund bet amount when draw...");
            invoke_signed(
                &owner_change_ix,
                &[
                    house_token_account.clone(),
                    receiver_account.clone(),
                    pda_program.clone(),
                    token_program.clone(),
                ],
                &[&[&b"roshambo"[..], &[nonce]]],
            )?;
        } else {
            let tmp_calc = selection + 5 - host_result;
            if tmp_calc == 1 || tmp_calc == 3 || tmp_calc == 6 || tmp_calc == 8 {
                // Lose
                game_info.result = COption::Some(1);
            } else {
                // Win
                game_info.result = COption::Some(0);

                let owner_change_ix = spl_token::instruction::transfer(
                    token_program.key,
                    house_token_account.key,
                    receiver_account.key,
                    &pda,
                    &[&pda],
                    game_info.bet_amount * 2,
                )?;

                msg!("Refund bet amount when draw...");
                invoke_signed(
                    &owner_change_ix,
                    &[
                        house_token_account.clone(),
                        receiver_account.clone(),
                        pda_program.clone(),
                        token_program.clone(),
                    ],
                    &[&[&b"roshambo"[..], &[nonce]]],
                )?;
            }
        }

        msg!("Closing the game account and refund fee back to creator...");
        **game_creator.try_borrow_mut_lamports()? = game_creator
            .lamports()
            .checked_add(game_account.lamports())
            .ok_or(RoshamboError::AmountOverflow)?;
        **game_account.try_borrow_mut_lamports()? = 0;
        *game_account.try_borrow_mut_data()? = &mut [];

        Ok(())
    }
}
