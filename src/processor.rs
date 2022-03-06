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
    keccak::hash,
    system_instruction,
};
use std::str;
use std::{collections::BTreeMap};
use borsh::{BorshSerialize};
use spl_token::state::Account as TokenAccount;
use arrayref::{array_refs, array_ref};
use crate::{error::BridgeError, instruction::BridgeInstruction, state::{UnshieldRequest, IncognitoProxy, Vault}};

const LEN: usize = 1 + 1 + 32 + 32 + 32 + 32; // ignore last 32 bytes in instruction
const ASC: [u8; 32] = [0x8c,0x97,0x25,0x8f,0x4e,0x24,0x89,0xf1,0xbb,0x3d,0x10,0x29,0x14,0x8e,0x0d,0x83,0x0b,0x5a,0x13,0x99,0xda,0xff,0x10,0x84,0x04,0x8e,0x7b,0xd8,0xdb,0xe9,0xf8,0x59];

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
            msg!("Instruction: init beacon list and vault state");
            process_init_beacon(accounts, init_beacon_info, program_id)
        }

        BridgeInstruction::InitVault { } => {
            msg!("Instruction: init beacon list and vault state");
            process_init_vault(accounts, program_id)
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
    let authority_signer_seeds = &[
        incognito_proxy.key.as_ref(),
        &[incognito_proxy_info.bump_seed],
    ];

    let vault_authority_pubkey =
    Pubkey::create_program_address(authority_signer_seeds, program_id)?;

    let asc_key = Pubkey::new(&ASC);
    let incognio_proxy_associated_acc = Pubkey::find_program_address(
        &[
            &vault_authority_pubkey.to_bytes(),
            &spl_token::id().to_bytes(),
            &vault_token_account_info.mint.to_bytes(),
        ],
        &asc_key,
    ).0;

    if incognio_proxy_associated_acc != *vault_token_account.key {
        msg!("Only send to incognito proxy account will be accepted");
        return Err(ProgramError::IncorrectProgramId);
    }

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

    msg!("Issue pToken to incognitoproxy,address,token,amount:{},{},{},{}", incognito_proxy.key,str::from_utf8(&inc_address[..]).unwrap(), vault_token_account_info.mint, amount);

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
    let vault_account = next_account_info(account_info_iter)?;
    let incognito_proxy = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;
    let incognito_proxy_info = IncognitoProxy::unpack_unchecked(&incognito_proxy.data.borrow())?;
    if !incognito_proxy_info.is_initialized() {
       return Err(BridgeError::BeaconsUnInitialized.into())
    }

    if incognito_proxy_info.vault != *vault_account.key {
        msg!("Send to wrong vault account");
        return Err(ProgramError::IncorrectProgramId);
    }

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
        tx_id,
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
    let unshield_amount_u64 = u64::from_be_bytes(*unshield_amount);

    // validate metatype and key provided
    if meta_type != 157 || shard_id != 1 {
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
        let index_beacon = unshield_info.indexes[i];
        let beacon_key = incognito_proxy_info.beacons[index_beacon as usize];
        if beacon_key_from_signature_result != beacon_key {
            return Err(BridgeError::InvalidBeaconSignature.into());
        }
    }

    // todo: verify merkle tree
    // append block height to instruction
    // let height_bytes = unshield_info.height.to_le_bytes();
    // let mut inst_vec = inst.to_vec();
    // inst_vec.extend_from_slice(&height_bytes);
    // todo: store txid
    process_insert_map_vault(vault_account, program_id, tx_id)?;

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

    let vault_token_account_info = TokenAccount::unpack(&vault_token_account.try_borrow_data()?)?;

    if vault_token_account_info.owner != vault_authority_pubkey {
        msg!("Invalid vault token account owner");
        return Err(BridgeError::InvalidAccountOwner.into());
    }

    spl_token_transfer(TokenTransferParams {
        source: vault_token_account.clone(),
        destination: unshield_token_account.clone(),
        amount: unshield_amount_u64,
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
    let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;

    if !initalizer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    let incognito_proxy = next_account_info(account_info_iter)?;
    assert_rent_exempt(rent, incognito_proxy)?;
    let mut incognito_proxy_info = assert_uninitialized::<IncognitoProxy>(incognito_proxy)?;
    if incognito_proxy.owner != program_id {
        msg!("Invalid incognito proxy");
        return Err(ProgramError::IncorrectProgramId);
    }

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

// process init vault state - to store unshielding txIDs
fn process_init_vault(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let initalizer = next_account_info(account_info_iter)?;
    let vault_account = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

    if !initalizer.is_signer {
        return Err(ProgramError::MissingRequiredSignature)
    }

    let (map_pda, map_bump) = Pubkey::find_program_address(
        &[b"map".as_ref()],
        program_id
    );

    if map_pda != *vault_account.key || !vault_account.is_writable || !vault_account.data_is_empty() {
        return Err(BridgeError::InvalidMapAccount.into())
    }

    let rent = Rent::get()?;
    let rent_lamports = rent.minimum_balance(Vault::LEN);

    let create_map_ix = &system_instruction::create_account(
        initalizer.key, 
        vault_account.key, 
        rent_lamports, 
        Vault::LEN.try_into().unwrap(), 
        program_id
    );

    msg!("Creating MapAccount account");
    invoke_signed(
        create_map_ix, 
        &[
            initalizer.clone(),
            vault_account.clone(),
            system_program.clone()
        ],
        &[&[
            b"map".as_ref(),
            &[map_bump]
        ]]
    )?;

    msg!("Deserializing MapAccount account");
    // let mut vault_state = try_from_slice_unchecked::<Vault>(&vault_account.data.borrow()).unwrap();
    let mut vault_state = Vault::deserialize(&vault_account.data.borrow());
    if vault_state.is_initialized() {
        msg!("Vault initialized");
        return Err(BridgeError::VaultInitialized.into());
    }

    vault_state.is_initialized = true;
    let empty_map: BTreeMap<[u8; 32], bool> = BTreeMap::new();
    vault_state.map = empty_map;

    msg!("Serializing MapAccount account");
    vault_state.serialize(&mut &mut vault_account.data.borrow_mut()[..])?;

    Ok(())
}

// process init vault state - to store unshielding txIDs
fn process_insert_map_vault(vault_account: &AccountInfo, program_id: &Pubkey, key: &[u8; 32]) -> ProgramResult {
    if vault_account.data.borrow()[0] == 0 || *vault_account.owner != *program_id {
        return Err(BridgeError::InvalidMapAccount.into())
    }

    msg!("Deserializing MapAccount account");
    let mut vault_state = Vault::deserialize(&vault_account.data.borrow());

    if vault_state.map.contains_key(key) {
        return Err(BridgeError::TxIDExisted.into())
    }

    vault_state.map.insert(*key, true);
    
    msg!("Serializing MapAccount account");
    vault_state.serialize(&mut &mut vault_account.data.borrow_mut()[..])?;

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

fn assert_uninitialized<T: Pack + IsInitialized>(
    account_info: &AccountInfo,
) -> Result<T, ProgramError> {
    let account: T = T::unpack_unchecked(&account_info.data.borrow())?;
    if account.is_initialized() {
        Err(BridgeError::BeaconsInitialized.into())
    } else {
        Ok(account)
    }
}

