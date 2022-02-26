use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
    system_instruction,
};

use spl_token::state::Account as TokenAccount;

use crate::{error::BridgeError, instruction::BridgeInstruction, state::{Vault, UnshieldRequest, IncognitoProxy}};

pub struct Processor;
impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = BridgeInstruction::unpack(instruction_data)?;

        match instruction {
            BridgeInstruction::Shield { amount, inc_address } => {
                msg!("Instruction: Shield");
                Self::process_shield(accounts, amount, inc_address, program_id)
            }
            BridgeInstruction::UnShield { unshield_info } => {
                msg!("Instruction: Unshield");
                Self::process_unshield(accounts, unshield_info, program_id)
            }
            BridgeInstruction::InitBeacon { init_beacon_info } => {
                msg!("Instruction: init beacon list");
                Self::process_init_beacon(accounts, init_beacon_info, program_id)      
            }
        }
    }

    fn process_shield(
        accounts: &[AccountInfo],
        amount: u64,
        inc_address: String,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let shied_maker = next_account_info(account_info_iter)?;

        if !shied_maker.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        let shield_maker_token_account = next_account_info(account_info_iter)?;
        let vault_token_account = next_account_info(account_info_iter)?;
        let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;
        let incognito_proxy = next_account_info(account_info_iter)?;
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

        if vault_token_account_info.owner != vault_authority_pubkey {
            msg!("Send to wrong vault token account");
            return Err(ProgramError::IncorrectProgramId); 
        }
        assert_rent_exempt(rent, vault_token_account)?;
  
        let transfer_to_vault_tx = spl_token::instruction::transfer(
            token_program.key,
            shield_maker_token_account.key,
            vault_token_account.key,
            shied_maker.key,
            &[&shied_maker.key],
            amount,
        )?;
        msg!("Calling the token program to transfer token from user account to vault");
        invoke(
            &transfer_to_vault_tx,
            &[
                shield_maker_token_account.clone(),
                vault_token_account.clone(),
                shied_maker.clone(),
                token_program.clone(),
            ],
        )?;
        msg!("Issue pToken to address {}, token {}", inc_address, token_program.key);

        Ok(())
    }

    //todo: transfer sol to vault bridge

    // add logic to proccess unshield for users
    fn process_unshield(
        accounts: &[AccountInfo],
        unshield_info: UnshieldRequest,
        program_id: &Pubkey,
    ) -> ProgramResult {

        Ok(())
    }

    // add logic to proccess init beacon list
    fn process_init_beacon(
        accounts: &[AccountInfo],
        init_beacon_info: IncognitoProxy,
        program_id: &Pubkey,
    ) -> ProgramResult {
        
        Ok(())
    }
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