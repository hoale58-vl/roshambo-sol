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
    /// 1. `[writable]` Temporary token account that should be created prior to this instruction and owned by the game creator
    /// 2. `[writable]` The game account, it will hold all necessary info about the game.
    /// 3. `[]` The token program
    NewGame {},

    /// End a game - Receive reward amount if this game win (x2) - or nothing if lose
    ///
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person owned the game
    /// 1. `[signer]` The account of the house verify the result of this game
    /// ...
    /// 2. `[]` The token program
    EndGame { result: u8 },
}

impl RoshamboInstruction {
    /// Unpacks a byte buffer into a [RoshamboInstruction](enum.RoshamboInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        Ok(match tag {
            0 => Self::NewGame {},
            _ => return Err(InvalidInstruction.into()),
        })
    }
}
