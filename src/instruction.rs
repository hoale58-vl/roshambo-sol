// program API, (de)serializing instruction data

use solana_program::program_error::ProgramError;

use crate::error::RoshamboError::InvalidInstruction;

pub enum RoshamboInstruction {
    /// Initialize Config - All games using this config will use the Mint Token same as this config
    ///
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person create the config
    /// 1. `[writable]` Config token account which will be initialized
    /// 2. `[]` The mint token account
    Initialize {
        min_bet_amount: u64,
        max_bet_amount: u64,
    },

    /// Create a new game by deposit amount of $TOKEN (e.g: wrapped SOL)
    ///
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person create the game
    /// 1. `[writable]` Creator token account
    /// 2. `[writable]` The game account, it will hold all necessary info about the game.
    /// 3. `[writable]` House token account owned by PDA
    /// 4. `[writable]` Roshambo config
    /// 5. `[]` The token program
    NewGame { amount: u64 },

    /// End a game - Receive reward amount if this game win (x2) - or nothing if lose
    ///
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person owned the game - game creator
    /// 1. `[signer]` The account of the house verify the result of this game
    /// 2. `[writable]` The game account, it will hold all necessary info about the game (close after this and refund rent fee back to caller)
    /// 3. `[writable]` Temporary token account owned by PDA that the game creator bet before (close if lose - double if win)
    /// 4. `[writable]` House token account owned by PDA (change based on game result)
    /// 5. `[writable]` Roshambo config
    /// 6. `[]` The token program
    /// 7. `[]` The PDA account - get by PublicKey.findProgramAddress
    ClaimReward { host_seed: u64, public_seed: u64 },

    /// Update min - max bet amount for specific config
    ///
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person who create the config
    /// 1. `[writable]` Initialized Config account
    UpdateConfig {
        min_bet_amount: u64,
        max_bet_amount: u64,
    },

    /// Withdraw token from house token account
    ///
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person who create the config
    /// 1. `[]` Initialized Config account
    /// 2. `[writable]` House token account owned by PDA
    /// 3. `[]` The token program
    /// 4. `[]` The PDA account - get by PublicKey.findProgramAddress
    Withdraw { amount: u64 },
}

impl RoshamboInstruction {
    /// Unpacks a byte buffer into a [RoshamboInstruction](enum.RoshamboInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        Ok(match tag {
            0 => {
                let (min_bet_amount, max_bet_amount) = Self::unpack_config(rest)?;
                Self::Initialize {
                    min_bet_amount,
                    max_bet_amount,
                }
            }
            1 => Self::NewGame {
                amount: Self::unpack_amount(rest)?,
            },
            2 => {
                let (host_seed, public_seed) = Self::unpack_claim_reward(rest)?;
                Self::ClaimReward {
                    host_seed,
                    public_seed,
                }
            }
            3 => {
                let (min_bet_amount, max_bet_amount) = Self::unpack_config(rest)?;
                Self::UpdateConfig {
                    min_bet_amount,
                    max_bet_amount,
                }
            }
            4 => Self::Withdraw {
                amount: Self::unpack_amount(rest)?,
            },
            _ => return Err(InvalidInstruction.into()),
        })
    }

    fn unpack_config(input: &[u8]) -> Result<(u64, u64), ProgramError> {
        let min_bet_amount = input
            .get(..8)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(InvalidInstruction)?;

        let max_bet_amount = input
            .get(9..16)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(InvalidInstruction)?;

        Ok((min_bet_amount, max_bet_amount))
    }

    fn unpack_amount(input: &[u8]) -> Result<u64, ProgramError> {
        let bet_amount = input
            .get(..8)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(InvalidInstruction)?;
        Ok(bet_amount)
    }

    fn unpack_claim_reward(input: &[u8]) -> Result<(u64, u64), ProgramError> {
        let host_seed = input
            .get(..8)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(InvalidInstruction)?;

        let public_seed = input
            .get(9..16)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(InvalidInstruction)?;

        Ok((host_seed, public_seed))
    }
}
