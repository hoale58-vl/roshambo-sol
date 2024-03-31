// program objects, (de)serializing state

use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

pub struct Game {
    pub is_initialized: bool,
    pub game_creator_pubkey: Pubkey,
    pub temp_token_account_pubkey: Pubkey,
}

impl Sealed for Game {}
impl IsInitialized for Game {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for Game {
    const LEN: usize = 65;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, Game::LEN];
        let (is_initialized, game_creator_pubkey, temp_token_account_pubkey) =
            array_refs![src, 1, 32, 32];
        let is_initialized = match is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };

        Ok(Game {
            is_initialized,
            game_creator_pubkey: Pubkey::new_from_array(*game_creator_pubkey),
            temp_token_account_pubkey: Pubkey::new_from_array(*temp_token_account_pubkey),
        })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, Game::LEN];
        let (is_initialized_dst, game_creator_pubkey_dst, temp_token_account_pubkey_dst) =
            mut_array_refs![dst, 1, 32, 32];

        let Game {
            is_initialized,
            game_creator_pubkey,
            temp_token_account_pubkey,
        } = self;

        is_initialized_dst[0] = *is_initialized as u8;
        game_creator_pubkey_dst.copy_from_slice(game_creator_pubkey.as_ref());
        temp_token_account_pubkey_dst.copy_from_slice(temp_token_account_pubkey.as_ref());
    }
}
