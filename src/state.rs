// program objects, (de)serializing state

use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use solana_program::{
    program_error::ProgramError,
    program_option::COption,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

pub struct Game {
    pub is_initialized: bool,
    pub bet_amount: u64,
    pub game_creator_pubkey: Pubkey,
    pub result: COption<u8>,
}

impl Sealed for Game {}
impl IsInitialized for Game {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

/// Initialization flag size for account state
pub const INITIALIZED_BYTES: usize = 1;
pub const U64_LENGTH: usize = 8;
pub const PUBKEY_BYTES: usize = 32;
pub const OPTIONAL_U8: usize = 5;
pub const ACCOUNT_STATE_SPACE: usize = INITIALIZED_BYTES + U64_LENGTH + PUBKEY_BYTES + OPTIONAL_U8;

fn pack_coption_u8(src: &COption<u8>, dst: &mut [u8; OPTIONAL_U8]) {
    let (tag, body) = mut_array_refs![dst, 4, 1];
    match src {
        COption::Some(result) => {
            *tag = [1, 0, 0, 0];
            *body = result.to_le_bytes();
        }
        COption::None => {
            *tag = [0; 4];
        }
    }
}

fn unpack_coption_u8(src: &[u8; OPTIONAL_U8]) -> Result<COption<u8>, ProgramError> {
    let (tag, body) = array_refs![src, 4, 1];
    match *tag {
        [0, 0, 0, 0] => Ok(COption::None),
        [1, 0, 0, 0] => Ok(COption::Some(u8::from_le_bytes(*body))),
        _ => Err(ProgramError::InvalidAccountData),
    }
}

impl Pack for Game {
    const LEN: usize = ACCOUNT_STATE_SPACE;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, ACCOUNT_STATE_SPACE];
        let (is_initialized, bet_amount, game_creator_pubkey, result) = array_refs![
            src,
            INITIALIZED_BYTES,
            U64_LENGTH,
            PUBKEY_BYTES,
            OPTIONAL_U8
        ];
        let is_initialized = match is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };

        Ok(Game {
            is_initialized,
            bet_amount: u64::from_le_bytes(*bet_amount),
            game_creator_pubkey: Pubkey::new_from_array(*game_creator_pubkey),
            result: unpack_coption_u8(result)?,
        })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, ACCOUNT_STATE_SPACE];
        let (is_initialized_dst, bet_amount_dst, game_creator_pubkey_dst, result_dst) = mut_array_refs![
            dst,
            INITIALIZED_BYTES,
            U64_LENGTH,
            PUBKEY_BYTES,
            OPTIONAL_U8
        ];

        let Game {
            is_initialized,
            bet_amount,
            game_creator_pubkey,
            ref result,
        } = self;

        is_initialized_dst[0] = *is_initialized as u8;
        *bet_amount_dst = bet_amount.to_le_bytes();
        game_creator_pubkey_dst.copy_from_slice(game_creator_pubkey.as_ref());
        pack_coption_u8(result, result_dst);
    }
}
