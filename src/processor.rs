use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
    instruction::Instruction,
    secp256k1_recover::secp256k1_recover,
    keccak::hash
};

use spl_token::state::Account as TokenAccount;
use arrayref::{array_refs, array_ref};
use crate::{error::BridgeError, instruction::BridgeInstruction, state::{Vault, UnshieldRequest, IncognitoProxy}};

const LEN: usize = 1 + 1 + 32 + 32 + 32 + 32; // ignore last 32 bytes in instruction

pub fn process_instruction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
) -> ProgramResult {
    let instruction = BridgeInstruction::unpack(instruction_data)?;

    match instruction {
        BridgeInstruction::Shield { amount, inc_address } => {
            msg!("Instruction: Shield");
            process_shield(accounts, amount, inc_address, program_id)
        }
        BridgeInstruction::UnShield { unshield_info } => {
            msg!("Instruction: Unshield");
            process_unshield(accounts, unshield_info, program_id)
        }
        BridgeInstruction::InitBeacon { init_beacon_info } => {
            msg!("Instruction: init beacon list");
            process_init_beacon(accounts, init_beacon_info, program_id)
        }
    }
}

fn process_shield(
    accounts: &[AccountInfo],
    amount: u64,
    inc_address: [u8; 148],
    program_id: &Pubkey,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let shield_maker_token_account = next_account_info(account_info_iter)?;
    let vault_token_account = next_account_info(account_info_iter)?;
    let incognito_proxy = next_account_info(account_info_iter)?;
    let shied_maker = next_account_info(account_info_iter)?;
    if !shied_maker.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    let token_program = next_account_info(account_info_iter)?;

    if incognito_proxy.owner != program_id {
        msg!("Invalid incognito proxy");
        return Err(ProgramError::IncorrectProgramId);
    }

    if *vault_token_account.owner != spl_token::id() {
        msg!("Vault token account must be owned by spl token");
        return Err(ProgramError::IncorrectProgramId);
    }
    let vault_token_account_info = TokenAccount::unpack(&vault_token_account.try_borrow_data()?)?;
    let incognito_proxy_info = IncognitoProxy::unpack(&incognito_proxy.try_borrow_data()?)?;
    msg!("bump seed {}", incognito_proxy_info.bump_seed);
    let authority_signer_seeds = &[
        incognito_proxy.key.as_ref(),
        &[incognito_proxy_info.bump_seed],
    ];

    let vault_authority_pubkey =
    Pubkey::create_program_address(authority_signer_seeds, program_id)?;

    if vault_token_account_info.owner != vault_authority_pubkey {
        msg!("Send to wrong vault token account");
        return Err(ProgramError::IncorrectProgramId);
    }

    spl_token_transfer(TokenTransferParams {
        source: shield_maker_token_account.clone(),
        destination: vault_token_account.clone(),
        amount,
        authority: shied_maker.clone(),
        authority_signer_seeds: &[],
        token_program: token_program.clone(),
    })?;

    msg!("Issue pToken to address {:?}, token {}", inc_address , token_program.key);

    Ok(())

}

/// [x] declare vars
/// [x] extract info from input params
/// [x] verify beacon signatures
/// [ ] verify instruction merkle tree
/// [ ] store unshield tx id
/// [x] transfer token back to user

// add logic to proccess unshield for users
fn process_unshield(
    accounts: &[AccountInfo],
    unshield_info: UnshieldRequest,
    program_id: &Pubkey,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let vault_token_account = next_account_info(account_info_iter)?;
    let unshield_token_account = next_account_info(account_info_iter)?;
    let vault_authority_account = next_account_info(account_info_iter)?;
    // let vault_account = next_account_info(account_info_iter)?;
    let incognito_proxy = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;
    let incognito_proxy_info = IncognitoProxy::unpack(&incognito_proxy.try_borrow_data()?)?;

    // if incognito_proxy_info.vault != *vault_account.key {
    //     msg!("Send to wrong vault account");
    //     msg!("{}", incognito_proxy_info.vault);
    //     msg!("{}", vault_account.key);
    //     return Err(ProgramError::IncorrectProgramId);
    // }

    if incognito_proxy.owner != program_id {
        msg!("Invalid incognito proxy");
        return Err(ProgramError::IncorrectProgramId);
    }

    // extract data from input
    let inst = unshield_info.inst;
    if inst.len() < LEN {
        msg!("Invalid instruction input");
        return Err(BridgeError::InvalidBeaconInstruction.into());
    }
    let inst_ = array_ref![inst, 0, LEN];
    #[allow(clippy::ptr_offset_with_cast)]
    let (
        meta_type,
        shard_id,
        token,
        receiver_key,
        _,
        unshield_amount,
        tx_id, // store this data
    ) = array_refs![
        inst_,
        1,
        1,
        32,
        32,
        24,
        8,
        32
    ];
    let meta_type = u8::from_le_bytes(*meta_type);
    let shard_id = u8::from_le_bytes(*shard_id);
    let token_key = Pubkey::new(token);
    let receiver_key = Pubkey::new(receiver_key);
    let unshield_amount = u64::from_le_bytes(*unshield_amount);

    // validate metatype and key provided
    if meta_type != 155 || shard_id != 1 {
        msg!("Invalid beacon instruction metatype {}, {}", meta_type, shard_id);
        return Err(BridgeError::InvalidKeysInInstruction.into());
    }

    let unshield_account_info = TokenAccount::unpack(&unshield_token_account.try_borrow_data()?)?;
    if token_key != unshield_account_info.mint {
        msg!("Token key and key provided not match {}, {}", token_key, unshield_account_info.mint);
        return Err(BridgeError::InvalidKeysInInstruction.into());
    }

    if receiver_key != *unshield_token_account.key {
        msg!("Receive key and key provided not match {}, {}", receiver_key, *unshield_token_account.key);
        return Err(BridgeError::InvalidKeysInInstruction.into());
    }

    // verify beacon signature
    if unshield_info.indexes.len() != unshield_info.signatures.len() {
        msg!("Invalid instruction provided, length of indexes and signatures not match");
        return Err(BridgeError::InvalidBeaconInstruction.into());
    }

    if unshield_info.signatures.len() <= incognito_proxy_info.beacons.len() * 2 / 3 {
        msg!("Invalid instruction input");
        return Err(BridgeError::InvalidNumberOfSignature.into());
    }

    let mut blk_data_bytes = unshield_info.blk_data.to_vec();
    blk_data_bytes.extend_from_slice(&unshield_info.inst_root);
    // Get double block hash from instRoot and other data
    let blk = hash(&hash(&blk_data_bytes[..]).to_bytes());

    for i in 0..unshield_info.indexes.len() {
        let s_r_v = unshield_info.signatures[i];
        let (s_r, v) = s_r_v.split_at(64);
        if v.len() != 1 {
            msg!("Invalid signature v input");
            return Err(BridgeError::InvalidBeaconSignature.into());
        }
        let beacon_key_from_signature_result = secp256k1_recover(
            &blk.to_bytes()[..],
            v[0],
            s_r,
        ).unwrap();
        let beacon_key = incognito_proxy_info.beacons[unshield_info.indexes[i] as usize];
        if beacon_key_from_signature_result != beacon_key {
            msg!("Invalid beacon signature expected {:?} got {:?}", beacon_key.to_bytes(), beacon_key_from_signature_result.to_bytes());
            return Err(BridgeError::InvalidBeaconSignature.into());
        }
    }

    // todo: verify merkle tree
    // append block height to instruction
    // let height_bytes = unshield_info.height.to_le_bytes();
    // let mut inst_vec = inst.to_vec();
    // inst_vec.extend_from_slice(&height_bytes);
    // todo: store txid

    // prepare to transfer token to user
    let authority_signer_seeds = &[
        incognito_proxy.key.as_ref(),
        &[incognito_proxy_info.bump_seed],
    ];

    let vault_authority_pubkey =
    Pubkey::create_program_address(authority_signer_seeds, program_id)?;

    if &vault_authority_pubkey != vault_authority_account.key {
        msg!(
            "Derived vault authority {} does not match the vault authority account provided {}",
            &vault_authority_pubkey.to_string(),
            &vault_authority_account.key.to_string(),
        );
        return Err(BridgeError::InvalidTokenAuthority.into());
    }

    spl_token_transfer(TokenTransferParams {
        source: vault_token_account.clone(),
        destination: unshield_token_account.clone(),
        amount: unshield_amount,
        authority: vault_authority_account.clone(),
        authority_signer_seeds,
        token_program: token_program.clone(),
    })?;

    Ok(())
}

// add logic to proccess init beacon list
fn process_init_beacon(
    accounts: &[AccountInfo],
    init_beacon_info: IncognitoProxy,
    program_id: &Pubkey,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let initalizer = next_account_info(account_info_iter)?;

    if !initalizer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    let incognito_proxy = next_account_info(account_info_iter)?;
    if incognito_proxy.owner != program_id {
        msg!("Invalid incognito proxy");
        return Err(ProgramError::IncorrectProgramId);
    }
    let mut incognito_proxy_info = IncognitoProxy::unpack_from_slice(&incognito_proxy.try_borrow_data()?)?;
    // todo: uncomment in production
    // if incognito_proxy_info.is_initialized {
    //     msg!("Beacon initialized");
    //     return Err(BridgeError::BeaconsInitialized.into());
    // }
    incognito_proxy_info.is_initialized = init_beacon_info.is_initialized;
    incognito_proxy_info.bump_seed = init_beacon_info.bump_seed;
    incognito_proxy_info.vault = init_beacon_info.vault;
    incognito_proxy_info.beacons = init_beacon_info.beacons;
    IncognitoProxy::pack(incognito_proxy_info, &mut incognito_proxy.data.borrow_mut())?;

    Ok(())
}

// check rent exempt
fn assert_rent_exempt(rent: &Rent, account_info: &AccountInfo) -> ProgramResult {
    if !rent.is_exempt(account_info.lamports(), account_info.data_len()) {
        msg!(
            "Rent exempt balance insufficient got {} expected {}",
            &account_info.lamports().to_string(),
            &rent.minimum_balance(account_info.data_len()).to_string(),
        );
        Err(BridgeError::NotRentExempt.into())
    } else {
        Ok(())
    }
}

/// Issue a spl_token `Transfer` instruction.
#[inline(always)]
fn spl_token_transfer(params: TokenTransferParams<'_, '_>) -> ProgramResult {
    let TokenTransferParams {
        source,
        destination,
        authority,
        token_program,
        amount,
        authority_signer_seeds,
    } = params;
    let result = invoke_optionally_signed(
        &spl_token::instruction::transfer(
            token_program.key,
            source.key,
            destination.key,
            authority.key,
            &[],
            amount,
        )?,
        &[source, destination, authority, token_program],
        authority_signer_seeds,
    );
    result.map_err(|_| BridgeError::TokenTransferFailed.into())
}

/// Invoke signed unless signers seeds are empty
#[inline(always)]
fn invoke_optionally_signed(
    instruction: &Instruction,
    account_infos: &[AccountInfo],
    authority_signer_seeds: &[&[u8]],
) -> ProgramResult {
    if authority_signer_seeds.is_empty() {
        invoke(instruction, account_infos)
    } else {
        invoke_signed(instruction, account_infos, &[authority_signer_seeds])
    }
}

struct TokenTransferParams<'a: 'b, 'b> {
    source: AccountInfo<'a>,
    destination: AccountInfo<'a>,
    amount: u64,
    authority: AccountInfo<'a>,
    authority_signer_seeds: &'b [&'b [u8]],
    token_program: AccountInfo<'a>,
}