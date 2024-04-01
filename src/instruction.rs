// program API, (de)serializing instruction data

use solana_program::program_error::ProgramError;

use crate::error::RoshamboError::InvalidInstruction;

pub enum RoshamboInstruction {
    /// Create a new game by deposit amount of $TOKEN (e.g: wrapped SOL)
    ///
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person create the game
    /// 1. `[writable]` Creator token account
    /// 2. `[writable]` The game account, it will hold all necessary info about the game.
    /// 3. `[writable]` House token account owned by PDA
    /// 4. `[]` The token program
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
    /// 5. `[]` The token program
    /// 6. `[]` The PDA account - get by PublicKey.findProgramAddress
    ClaimReward { host_seed: u64, public_seed: u64 },
}

impl RoshamboInstruction {
    /// Unpacks a byte buffer into a [RoshamboInstruction](enum.RoshamboInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        Ok(match tag {
            0 => Self::NewGame {
                amount: Self::unpack_bet_amount(rest)?,
            },
            1 => {
                let (host_seed, public_seed) = Self::unpack_claim_reward(rest)?;
                Self::ClaimReward {
                    host_seed,
                    public_seed,
                }
            }
            _ => return Err(InvalidInstruction.into()),
        })
    }

    fn unpack_bet_amount(input: &[u8]) -> Result<u64, ProgramError> {
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
