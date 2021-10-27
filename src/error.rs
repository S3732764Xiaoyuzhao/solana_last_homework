use thiserror::Error;

use solana_program::program_error::ProgramError;

#[derive(Error, Debug, Copy, Clone)]
pub enum CustomError {
    #[error("Invalid Instruction")]
    InvalidInstruction,
    #[error("Deposit Zero")]
    DepositZero,
    #[error("Withdraw Zero")]
    WithdrawZero,
    #[error("Signature Error")]
    SignatureError,
    #[error("User derived address Error")]
    UserDerivedAddressError,
    #[error("Program derived address Error")]
    ProgramDerivedAddressError,
    #[error("Calculation Overflow")]
    CalculationOverflow,
}

impl From<CustomError> for ProgramError {
    fn from(e: CustomError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
