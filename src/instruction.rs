use solana_program::program_error::ProgramError;
use std::convert::TryInto;

use crate::error::BridgeError::InvalidInstruction;
use crate::state::UnshieldRequest;
pub enum BridgeInstruction {

    ///// shield
    Shield {
        /// shield info
        amount: u64,
        inc_address: String,
    },

    ///// shield
    UnShield {
        /// unshield info
        amount: u64,
        unshield_info: UnshieldRequest,
    },

    // Unshield
}

impl BridgeInstruction {
    /// Unpacks a byte buffer into a [BridgeInstruction](enum.BridgeInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        Ok(match tag {
            0 => Self::Shield {
                amount: Self::unpack_amount(rest)?,
                inc_address: Self::unpack_str(rest)?,
            },
            1 => Self::UnShield {
                amount: Self::unpack_amount(rest)?,
            },
            _ => return Err(InvalidInstruction.into()),
        })
    }

    fn unpack_amount(input: &[u8]) -> Result<u64, ProgramError> {
        let amount = input
            .get(..8)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(InvalidInstruction)?;
        Ok(amount)
    }

    fn unpack_str(input: &[u8]) -> Result<String, ProgramError> {
        let s = str::from_utf8(input).ok_or(InvalidInstruction)?;
        Ok(s)
    }
}