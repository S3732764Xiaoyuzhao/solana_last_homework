
use crate::{
    instruction::Instruction,
    error::CustomError,
    error::CustomError::{DepositZero, WithdrawZero,SignatureError,UserDerivedAddressError,
        ProgramDerivedAddressError},
    state::UserBalance,
};
use borsh::BorshDeserialize;
use borsh::BorshSerialize;
use solana_program::{
    instruction::{AccountMeta, Instruction as SysInstruction},
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed},
    msg,
    program_error::ProgramError,
    pubkey::Pubkey, 
    program_pack::Pack,
};
use spl_token::state::Account as TokenAccount;
use spl_associated_token_account::get_associated_token_address;

pub struct Processor;
impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = Instruction::unpack(instruction_data)?;

        match instruction {
            Instruction::Deposit{amount}  => {
                msg!("Instruction: Deposit");
                Self::process_deposit(program_id, accounts, amount)
            }
            Instruction::Withdraw{nonce}  => {
                msg!("Instruction: Withdraw");
                Self::process_withdraw(program_id, accounts, nonce)
            }
            Instruction::CreateProgramAssociatedAddresse => {
                msg!("Instruction: CreateProgramAssociatedAddresse");
                Self::process_devided(program_id, accounts)
            }
        }
    }
    fn process_deposit(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
    ) -> ProgramResult {

        msg!("process_deposit");
        let account_info_iter = &mut accounts.iter();
        if amount == 0 {
            return Err(DepositZero.into());
        }
        let user_account = next_account_info(account_info_iter)?; 
        if !user_account.is_signer {
            return Err(SignatureError.into());
        }
        let ada_account = next_account_info(account_info_iter)?; 
        let user_derived_account = next_account_info(account_info_iter)?; 
        let program_associated_account = next_account_info(account_info_iter)?; 
        let token_program = next_account_info(account_info_iter)?;
        let spl_token_account = next_account_info(account_info_iter)?;
        let seed = "last_homework";
        let check_user_derived_pubkey = Pubkey::create_with_seed(user_account.key,
             &seed, program_id).unwrap();
        if &check_user_derived_pubkey != user_derived_account.key{
            return Err(UserDerivedAddressError.into());
        }
        let mut user_derived_account_data: UserBalance =
        BorshDeserialize::deserialize(&mut &user_derived_account.data.borrow_mut()[..])
            .unwrap();
        msg!("user_derived_account_data.balance {:?}",user_derived_account_data.balance);
        msg!("amount {:?}",amount);
        user_derived_account_data.balance = user_derived_account_data.balance.checked_add(amount).ok_or(CustomError::CalculationOverflow)?;
        msg!("user_derived_account_data.balance {:?}",user_derived_account_data.balance);
        let ada_account_unpack = TokenAccount::unpack_from_slice(&ada_account.data.borrow())?;
        msg!("ada_account_unpack {:?}",ada_account_unpack);
        let program_associated_account_unpack = TokenAccount::unpack_from_slice(&program_associated_account.data.borrow())?;
        msg!("ada_account_unpack {:?}",program_associated_account_unpack);
        invoke(

            &spl_token::instruction::transfer_checked(
                &token_program.key,
                &ada_account.key,
                &spl_token_account.key,
                &program_associated_account.key,
                &user_account.key,
                &[],
                amount as u64,
                9,
            )?,
            &[
                spl_token_account.clone(),
                user_account.clone(),
                token_program.clone(),
                ada_account.clone(),
                program_associated_account.clone(),
            ],
        )?;
        user_derived_account_data.serialize(&mut &mut user_derived_account.data.borrow_mut()[..])?;
        let ada_account_unpack = TokenAccount::unpack_from_slice(&ada_account.data.borrow())?;
        msg!("ada_account_unpack {:?}",ada_account_unpack);
        let program_associated_account_unpack = TokenAccount::unpack_from_slice(&program_associated_account.data.borrow())?;
        msg!("ada_account_unpack {:?}",program_associated_account_unpack);
        Ok(())
    }
    fn process_withdraw(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        nonce: u8,
    ) -> ProgramResult {

        msg!("process_withdraw");
        let account_info_iter = &mut accounts.iter();
        let user_account = next_account_info(account_info_iter)?; 
        let ada_account = next_account_info(account_info_iter)?; 
        let user_derived_account = next_account_info(account_info_iter)?; 
        let program_derived_account = next_account_info(account_info_iter)?; 
        let program_associated_account = next_account_info(account_info_iter)?; 
        let token_program = next_account_info(account_info_iter)?;
        let spl_token_account = next_account_info(account_info_iter)?;
        let seed = "last_homework";
        let check_user_derived_pubkey = Pubkey::create_with_seed(user_account.key,
             &seed, program_id).unwrap();
        let ada_account_unpack = TokenAccount::unpack_from_slice(&ada_account.data.borrow())?;
        msg!("ada_account_unpack {:?}",ada_account_unpack);   
        let program_associated_account_unpack = TokenAccount::unpack_from_slice(&program_associated_account.data.borrow())?;
        msg!("program_associated_account_unpack {:?}",program_associated_account_unpack);
        if &check_user_derived_pubkey != user_derived_account.key{
            return Err(UserDerivedAddressError.into());
        }
        let  (check_program_derived_pubkey,_) = Pubkey::find_program_address(&[b"last_homework"], &program_id);
        if &check_program_derived_pubkey != program_derived_account.key{
            return Err(ProgramDerivedAddressError.into());
        }
        let mut user_derived_account_data: UserBalance =
        BorshDeserialize::deserialize(&mut &user_derived_account.data.borrow_mut()[..])
            .unwrap();
        if user_derived_account_data.balance == 0{
            return Err(WithdrawZero.into());
        }
        invoke_signed(
            &spl_token::instruction::transfer_checked(
                &token_program.key,
                &program_associated_account.key,
                &spl_token_account.key,
                &ada_account.key,
                &program_derived_account.key,
                &[],
                user_derived_account_data.balance as u64,
                9,
            )?,
            &[
                spl_token_account.clone(),
                program_derived_account.clone(),
                token_program.clone(),
                ada_account.clone(),
                program_associated_account.clone(),
                program_derived_account.clone(),
            ],
            &[&[b"last_homework",&[nonce]]],
        )?;
        let ada_account_unpack = TokenAccount::unpack_from_slice(&ada_account.data.borrow())?;
        msg!("ada_account_unpack {:?}",ada_account_unpack);
        let program_associated_account_unpack = TokenAccount::unpack_from_slice(&program_associated_account.data.borrow())?;
        msg!("program_associated_account_unpack {:?}",program_associated_account_unpack);
        user_derived_account_data.balance = 0;
        user_derived_account_data.serialize(&mut &mut user_derived_account.data.borrow_mut()[..])?;

        Ok(())
    }

    fn process_devided(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let funder_info = next_account_info(account_info_iter)?;
        let program_devided_account_info = next_account_info(account_info_iter)?;
        let program_associated_account_info = next_account_info(account_info_iter)?;
        let spl_token_mint_info = next_account_info(account_info_iter)?;
        let spl_token_program_info = next_account_info(account_info_iter)?;
        let spl_associated_program_info = next_account_info(account_info_iter)?;
        let system_program_info = next_account_info(account_info_iter)?;
        let rent_sysvar_info = next_account_info(account_info_iter)?;
        let (program_devided, devided_bump_seed) =
            Pubkey::find_program_address(&[b"last_homework"], &program_id);
        let program_associated = get_associated_token_address(&program_devided, spl_token_mint_info.key);
        if !funder_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        if program_devided != *program_devided_account_info.key {
            return Err(ProgramError::InvalidSeeds);
        }
        if program_associated != *program_associated_account_info.key {
            return Err(ProgramError::InvalidSeeds);
        }
        let seed :&[&[_]] = &[b"last_homework", &[devided_bump_seed]];
        invoke_signed(
            &SysInstruction {
                program_id: *spl_associated_program_info.key,
                accounts: vec![
                    AccountMeta::new(*funder_info.key, true), 
                    AccountMeta::new(*program_associated_account_info.key, false), 
                    AccountMeta::new_readonly(*program_devided_account_info.key, false), 
                    AccountMeta::new_readonly(*spl_token_mint_info.key, false),          
                    AccountMeta::new_readonly(*system_program_info.key, false),          
                    AccountMeta::new_readonly(*spl_token_program_info.key, false),       
                    AccountMeta::new_readonly(*rent_sysvar_info.key, false),             
                ],
                data: vec![],
            },
            &[
                spl_associated_program_info.clone(),
                funder_info.clone(),
                program_devided_account_info.clone(),
                program_associated_account_info.clone(),
                spl_token_mint_info.clone(),
                system_program_info.clone(),
                spl_token_program_info.clone(),
                rent_sysvar_info.clone(),
            ],
            &[seed],
        )
    }

}