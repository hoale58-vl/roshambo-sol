// program specific errors

use thiserror::Error;

use solana_program::program_error::ProgramError;

#[derive(Error, Debug, Copy, Clone)]
pub enum RoshamboError {
    /// Invalid instruction
    #[error("Invalid Instruction")]
    InvalidInstruction,
    /// Not Rent Exempt
    #[error("Not Rent Exempt")]
    NotRentExempt,
    /// Not Rent Exempt
    #[error("GameEnded")]
    GameEnded,
    /// Amount Overflow
    #[error("Amount Overflow")]
    AmountOverflow,
}

impl From<RoshamboError> for ProgramError {
    fn from(e: RoshamboError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
