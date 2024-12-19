use solana_program::{msg, program_error::ProgramError};
use thiserror::Error;

#[derive(Error, Debug, Copy, Clone)]
pub enum HypeError {
    #[error("Invalid Accounts Number")]
    InvalidAccountsNumber = 6000,
    #[error("Invalid Instruction")]
    InvalidInstruction = 6001,
    #[error("Not Rent Exempt")]
    NotRentExempt = 6002,
    #[error("Expected Amount Mismatch")]
    ExpectedAmountMismatch = 6003,
    #[error("Initial Supply Must Be Non Zero")]
    InitialSupplyMustBeNonZero = 6004,
    #[error("Invalid Account Owner")]
    InvalidAccountOwner = 6005,
    #[error("Invalid Account Key")]
    InvalidAccountKey = 6006,
    #[error("Insufficient Funds")]
    InsufficientFunds = 6007,
    #[error("Invalid Token Mint")]
    InvalidTokenMint = 6008,
    #[error("Arithmetic Overflow")]
    ArithmeticOverflow = 6009,
    #[error("Invalid Hype Authority")]
    InvalidHypeAuthority = 6010,
    #[error("Invalid Token Program ID")]
    InvalidTokenProgramId = 6011,
    #[error("Invalid System Program ID")]
    InvalidSystemProgramId = 6012,
    #[error("Admin Signature Required")]
    AdminSignatureRequired = 6013,
    #[error("Invalid Root Account")]
    InvalidRootAccount = 6014,
    #[error("Invalid Account Tag")]
    InvalidAccountTag = 6015,
    #[error("Invalid Admin")]
    InvalidAdmin = 6016,
    #[error("Invalid Token 2022 Program ID")]
    InvalidToken2022ProgramId = 6017,
    #[error("Invalid Associated Token ID")]
    InvalidAssociatedTokenId = 6018,
    #[error("Invalid Client Account")]
    InvalidClientAccount = 6019,
    #[error("Invalid Token Account")]
    InvalidTokenAccount = 6020,
    #[error("Invalid Base Currency Mint")]
    InvalidBaseCrncyMint = 6021,
    #[error("Invalid Network ID")]
    InvalidNetworkId = 6022,
    #[error("Invalid Associated Token Address")]
    InvalidAssociatedTokenAddress = 6023,
    #[error("Invalid Token Supply")]
    InvalidTokenSupply = 6024,
    #[error("Too Small Quantity")]
    TooSmallQuantity = 6025,
    #[error("Max Trade Cost Exceeded")]
    MaxTradeCostExceeded = 6026,
    #[error("Invalid Base Currency Program Address")]
    InvalidBaseCrncyProgramAddress = 6027,
    #[error("Too Big Quantity")]
    TooBigQuantity = 6028,
    #[error("Invalid total supply")]
    InvalidTotalSupply = 6029,
    #[error("Invalid Mint Size")]
    InvalidMintSize = 6030,
    #[error("Invalid TVL")]
    InvalidTVL = 6031,
    #[error("Address has to be lower case")]
    AddressHasToBeLowerCase = 6032,
    #[error("Invalid Holder Account")]
    InvalidHolderAccount = 6033,
    #[error("Invalid Holder Admin")]
    InvalidHolderAdmin = 6034,
    #[error("Invalid Data Length")]
    InvalidDataLength = 6035,
    #[error("Invalid New Account PDA")]
    InvalidNewAccountPDA = 6036,
    #[error("Invalid New Operator Account")]
    InvalidNewOperatorAccount = 6037,
    #[error("Max Networks Count Exceeded")]
    MaxNetworksCountExceeded = 6038,
    #[error("Invalid Address")]
    InvalidAddress = 6039,
    #[error("Invalid Ref Address")]
    InvalidRefAddress = 6040,
    #[error("Invalid Validator")]
    InvalidValidator = 6041,
    #[error("Token already verified")]
    TokenAlreadyVerified = 6042,
    #[error("Invalid Fee Wallet")]
    InvalidFeeWallet = 6043,
    #[error("Invalid Supply")]
    InvalidSupply = 6044,
}

impl From<HypeError> for ProgramError {
    fn from(e: HypeError) -> Self {
        msg!("Error: {}", e);
        ProgramError::Custom(e as u32)
    }
}
