use thiserror::Error;
use solana_program::{decode_error::DecodeError, program_error::ProgramError};

#[derive(Error, Debug, Copy, Clone)]
pub enum BridgeError {
    /// Invalid instruction
    #[error("Invalid Instruction")]
    InvalidInstruction, // 0
    /// Not Rent Exempt
    #[error("Not Rent Exempt")]
    NotRentExempt,
    /// Expected Amount Mismatch
    #[error("Expected Amount Mismatch")]
    ExpectedAmountMismatch,
    /// Amount Overflow
    #[error("Amount Overflow")]
    AmountOverflow,
    /// The owner of the input isn't set to the program address generated by the program.
    #[error("Input account owner is not the program address")]
    InvalidAccountOwner,
    /// The list of beacon already initalized
    #[error("Account already initialized")]
    AccInitialized, // 5
    /// Invalid instruction data passed in.
    #[error("Failed to unpack instruction data")]
    InstructionUnpackError,
    /// Transfer token failed.
    #[error("Transfer token failed")]
    TokenTransferFailed,
    /// Invalid authority to transfer token
    #[error("Transfer token failed")]
    InvalidTokenAuthority,
    /// Invalid instruction input
    #[error("Invalid beacon instruction input")]
    InvalidBeaconInstruction,
    /// Invalid keys in instruction input
    #[error("Invalid keys in instruction input")]
    InvalidKeysInInstruction, // 10
    /// Not enough signatures in inst provided
    #[error("Invalid number of signatures")]
    InvalidNumberOfSignature,
    /// Invalid beacon signature
    #[error("Invalid beacon signature")]
    InvalidBeaconSignature,
    /// Invalid bool value
    #[error("Invalid bool value")]
    InvalidBoolValue,
    /// Beacon uninit
    #[error("Invalid incognito proxy account beacon uninit")]
    BeaconsUnInitialized,
    /// Block inst merkle tree
    #[error("Invalid lock inst merkle tree")]
    InvalidBeaconMerkleTree, // 15
    /// Invalid sender, receiver
    #[error("Invalid sender, receiver")]
    InvalidTransferTokenData,
    /// Close token account failed.
    #[error("Close token account failed")]
    CloseTokenAccountFailed,
    /// Invalid map account.
    #[error("Invalid map account")]
    InvalidMapAccount,
    /// Unshield request used.
    #[error("Unshield request used")]
    InvalidUnshieldRequestUsed,
    /// Invalid signer.
    #[error("Invalid signer")]
    InvalidSigner, // 20
    /// Invalid signer token authority.
    #[error("Invalid signer token authority")]
    InvalidSignerTokenAuth,
    /// Invalid meta type.
    #[error("Invalid meta type")]
    InvalidMetaType,
}

impl From<BridgeError> for ProgramError {
    fn from(e: BridgeError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for BridgeError {
    fn type_of() -> &'static str {
        "Bridge Error"
    }
}