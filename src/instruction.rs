use solana_program::{
    program_error::ProgramError,
    msg,
};
use std::{convert::TryInto, str};
use crate::error::BridgeError::{
    InvalidInstruction,
    InstructionUnpackError
};
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
        unshield_info: UnshieldRequest,
    },

    // Unshield
}

impl BridgeInstruction {
    /// Unpacks a byte buffer into a [BridgeInstruction](enum.BridgeInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
        Ok(match tag {
            0 => {
                let (amount, rest) = Self::unpack_u64(rest)?;
                let (inc_address, rest) = Self::unpack_str(rest, 148)?;
                Self::Shield {
                    amount,
                    inc_address
                }
            },
            1 => {
                let (inst_len, rest) = Self::unpack_u8(rest)?;
                let (inst, rest) =  Self::unpack_str(rest, inst_len as usize)?;
                let (height, rest) = Self::unpack_u64(rest)?;
                let (inst_paths_len, rest) = Self::unpack_u8(rest)?;
                let mut inst_paths = Vec::with_capacity(inst_paths_len as usize + 1);
                for _ in 0..inst_paths_len {
                    let (inst_node, rest) = Self::unpack_bytes32(rest)?;
                    inst_paths.push(*inst_node);
                }
                let (inst_paths_is_left_len, rest) = Self::unpack_u8(rest)?;
                let mut inst_path_is_lefts = Vec::with_capacity(inst_paths_is_left_len as usize + 1);
                for _ in 0..inst_paths_is_left_len {
                    let (inst_paths_is_left, rest) = Self::unpack_bool(rest)?;
                    inst_path_is_lefts.push(inst_paths_is_left);
                }
                let (inst_root, rest) = Self::unpack_bytes32(rest)?;
                let (blk_data, rest) = Self::unpack_bytes32(rest)?;
                let (indexes_len, rest) = Self::unpack_u8(rest)?;
                let mut indexes = Vec::with_capacity(indexes_len as usize + 1);
                for _ in 0..indexes_len {
                    let (index, rest) = Self::unpack_u8(rest)?;
                    indexes.push(index);
                }
                let (signature_len, rest) = Self::unpack_u8(rest)?;
                let mut signatures = Vec::with_capacity(signature_len as usize + 1);
                for _ in 0..signature_len {
                    let (signature, rest) = Self::unpack_bytes65(rest)?;
                    signatures.push(*signature);
                }
                Self::UnShield {
                    unshield_info: UnshieldRequest {
                        inst,
                        height,
                        inst_paths,
                        inst_path_is_lefts,
                        inst_root: *inst_root,
                        blk_data: *blk_data,
                        indexes,
                        signatures,
                    }
                }
            },
            _ => return Err(InvalidInstruction.into()),
        })
    }

    fn unpack_u64(input: &[u8]) -> Result<(u64, &[u8]), ProgramError> {
        if input.len() < 8 {
            msg!("u64 cannot be unpacked");
            return Err(InstructionUnpackError.into());
        }
        let (bytes, rest) = input.split_at(8);
        let value = bytes
            .get(..8)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(InstructionUnpackError)?;
        Ok((value, rest))
    }

    fn unpack_str(input: &[u8], len: usize) -> Result<(String, &[u8]), ProgramError> {
        if input.len() < len {
            msg!("inc address cannot be unpacked");
            return Err(InstructionUnpackError.into());
        }
        let (bytes, rest) = input.split_at(len);
        let output = String::from_utf8_lossy(bytes);
        Ok((output.to_string(), rest))
    }

    fn unpack_u8(input: &[u8]) -> Result<(u8, &[u8]), ProgramError> {
        if input.is_empty() {
            msg!("u8 cannot be unpacked");
            return Err(InstructionUnpackError.into());
        }
        let (bytes, rest) = input.split_at(1);
        let value = bytes
            .get(..1)
            .and_then(|slice| slice.try_into().ok())
            .map(u8::from_le_bytes)
            .ok_or(InstructionUnpackError)?;
        Ok((value, rest))
    }

    fn unpack_bytes32(input: &[u8]) -> Result<(&[u8; 32], &[u8]), ProgramError> {
        if input.len() < 32 {
            msg!("32 bytes cannot be unpacked");
            return Err(InstructionUnpackError.into());
        }
        let (bytes, rest) = input.split_at(32);
        Ok((
            bytes
                .try_into()
                .map_err(|_| InstructionUnpackError)?,
            rest,
        ))
    }

    fn unpack_bytes65(input: &[u8]) -> Result<(&[u8; 65], &[u8]), ProgramError> {
        if input.len() < 65 {
            msg!("65 bytes cannot be unpacked");
            return Err(InstructionUnpackError.into());
        }
        let (bytes, rest) = input.split_at(65);
        Ok((
            bytes
                .try_into()
                .map_err(|_| InstructionUnpackError)?,
            rest,
        ))
    }

    fn unpack_bool(input: &[u8]) -> Result<(bool, &[u8]), ProgramError> {
        let (value, rest) = Self::unpack_u8(input)?;

        match value {
            0 => Ok((false, rest)),
            1 => Ok((false, rest)),
            _ => {
                msg!("Boolean cannot be unpacked");
                Err(ProgramError::InvalidAccountData)
            }
        }
    }
}