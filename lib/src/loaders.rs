use borsh::{BorshDeserialize, BorshSerialize};
use pinocchio::{
    account_info::AccountInfo,
    instruction::Seed,
    msg,
    program_error::ProgramError,
    pubkey::{self, find_program_address, Pubkey},
};
use pinocchio_system::instructions::Transfer;
#[cfg(feature = "spl")]
use solana_program::program_pack::Pack;

use crate::{
    allocate_account, AccountInfoValidation, AsAccount, CloseAccount, Discriminator,
    LamportTransfer,
};

#[cfg(feature = "spl")]
use crate::{AccountValidation, AsSplToken};

impl AccountInfoValidation for AccountInfo {
    fn assert_signer(&self) -> Result<&Self, ProgramError> {
        if !self.is_signer() {
            msg!("Account is not a signer:");
            pubkey::log(self.key());
            return Err(ProgramError::MissingRequiredSignature);
        }
        Ok(self)
    }

    fn assert_writable(&self) -> Result<&Self, ProgramError> {
        if !self.is_writable() {
            msg!("Account is not writable:");
            pubkey::log(self.key());
            return Err(ProgramError::MissingRequiredSignature);
        }
        Ok(self)
    }

    fn assert_executable(&self) -> Result<&Self, ProgramError> {
        if !self.executable() {
            msg!("Account is not executable:");
            pubkey::log(self.key());
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(self)
    }

    fn assert_empty(&self) -> Result<&Self, ProgramError> {
        if !self.data_is_empty() {
            msg!("Account is not empty:");
            pubkey::log(self.key());
            return Err(ProgramError::AccountAlreadyInitialized);
        }
        Ok(self)
    }

    fn assert_not_empty(&self) -> Result<&Self, ProgramError> {
        if self.data_is_empty() {
            msg!("Account is empty:");
            pubkey::log(self.key());
            return Err(ProgramError::AccountAlreadyInitialized);
        }
        Ok(self)
    }

    fn assert_program(&self, program_id: &Pubkey) -> Result<&Self, ProgramError> {
        self.assert_key(program_id)?.assert_executable()
    }

    fn assert_type<T: Discriminator>(&self, program_id: &Pubkey) -> Result<&Self, ProgramError> {
        self.assert_owner(program_id)?;

        let expected_discriminator = T::discriminator();
        let actual_discriminator = self.try_borrow_data()?[0];

        if actual_discriminator.ne(&expected_discriminator) {
            msg!(
                "Account is invalid type (expected, actual): {:?}, {:?}",
                expected_discriminator,
                actual_discriminator
            );
            pubkey::log(self.key());
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(self)
    }

    fn assert_owner(&self, owner: &Pubkey) -> Result<&Self, ProgramError> {
        if self.owner().ne(owner) {
            msg!("Account owner mismatch (expected, actual):");
            pubkey::log(owner);
            pubkey::log(self.owner());
            return Err(ProgramError::InvalidAccountOwner);
        }
        Ok(self)
    }

    fn assert_key(&self, address: &Pubkey) -> Result<&Self, ProgramError> {
        if self.key().ne(address) {
            msg!("Account key mismatch:");
            pubkey::log(self.key());
            pubkey::log(address);
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(self)
    }

    fn assert_seeds(&self, seeds: &[&[u8]], program_id: &Pubkey) -> Result<&Self, ProgramError> {
        let pda = find_program_address(seeds, program_id);
        if self.key().ne(&pda.0) {
            msg!("Account is invalid seeds (expected, actual):");
            pubkey::log(&pda.0);
            pubkey::log(self.key());
            return Err(ProgramError::InvalidSeeds);
        }
        Ok(self)
    }

    // fn is_sysvar(&self, sysvar_id: &Pubkey) -> Result<&Self, ProgramError> {
    // self.has_owner(&pinocchio::sysvars::ID)?
    //     .has_address(sysvar_id)
    // }
}

impl AsAccount for AccountInfo {
    fn as_account<T>(&self, program_id: &Pubkey) -> Result<T, ProgramError>
    where
        T: BorshDeserialize + BorshSerialize + Discriminator,
    {
        self.assert_owner(program_id)?;
        T::try_from_slice(&self.try_borrow_data()?[1..])
            .map_err(|_| ProgramError::InvalidAccountData)
    }

    fn save_account<T>(&self, program_id: &Pubkey, data: &T) -> Result<(), ProgramError>
    where
        T: BorshDeserialize + BorshSerialize + Discriminator,
    {
        self.assert_owner(program_id)?.assert_writable()?;

        let mut account_data_ref = self.try_borrow_mut_data()?;
        account_data_ref[0] = T::discriminator();

        // TODO: Need to resize account data if it's not enough.

        account_data_ref[1..].copy_from_slice(
            &data
                .try_to_vec()
                .map_err(|_| ProgramError::InvalidAccountData)?,
        );
        Ok(())
    }

    // TODO: Program_id is the same as owner DUH
    fn create_account<T>(
        &self,
        data: &T,
        system_program: &AccountInfo,
        payer: &AccountInfo,
        owner: &Pubkey,
        seeds: &[Seed],
    ) -> Result<(), ProgramError>
    where
        T: BorshDeserialize + BorshSerialize + Discriminator,
    {
        self.assert_empty()?
            .assert_owner(system_program.key())?
            .assert_writable()?;

        let serialized_data = data
            .try_to_vec()
            .map_err(|_| ProgramError::InvalidAccountData)?;

        let space = 1 + serialized_data.len();

        allocate_account(self, system_program, payer, space, owner, seeds)?;

        let mut data = self.try_borrow_mut_data()?;
        data[0] = T::discriminator();

        data[1..].copy_from_slice(&serialized_data);

        Ok(())
    }

    // fn as_account_mut<T>(&self, program_id: &Pubkey) -> Result<&mut T, ProgramError>
    // where
    //     T: BorshDeserialize + BorshSerialize + Discriminator,
    // {
    //     unsafe {
    //         self.assert_owner(program_id)?;
    //         T::try_from_bytes_mut(std::slice::from_raw_parts_mut(
    //             self.try_borrow_mut_data()?.as_mut_ptr(),
    //             8 + std::mem::size_of::<T>(),
    //         ))
    //     }
    // }
}

impl<'a> LamportTransfer<'a> for AccountInfo {
    // TODO: This way of transfer is non-standard and doesn't show up in explorers.
    #[inline(always)]
    fn send(&'a self, lamports: u64, to: &'a AccountInfo) -> Result<(), ProgramError> {
        *self.try_borrow_mut_lamports()? -= lamports;
        *to.try_borrow_mut_lamports()? += lamports;
        Ok(())
    }

    #[inline(always)]
    fn collect(&'a self, lamports: u64, from: &'a AccountInfo) -> Result<(), ProgramError> {
        Transfer {
            from,
            to: self,
            lamports,
        }
        .invoke()
    }
}

impl<'a> CloseAccount<'a> for AccountInfo {
    fn close(&'a self, to: &'a AccountInfo) -> Result<(), ProgramError> {
        // Realloc data to zero.
        self.realloc(0, true)?;

        // Return rent lamports.
        self.send(self.lamports(), to);

        Ok(())
    }
}

#[cfg(feature = "spl")]
impl AsSplToken for AccountInfo<'_> {
    fn as_mint(&self) -> Result<spl_token::state::Mint, ProgramError> {
        unsafe {
            self.has_owner(&spl_token::ID)?;
            spl_token::state::Mint::unpack(std::slice::from_raw_parts(
                self.try_borrow_data()?.as_ptr(),
                spl_token::state::Mint::LEN,
            ))
        }
    }

    fn as_token_account(&self) -> Result<spl_token::state::Account, ProgramError> {
        unsafe {
            self.has_owner(&spl_token::ID)?;
            spl_token::state::Account::unpack(std::slice::from_raw_parts(
                self.try_borrow_data()?.as_ptr(),
                spl_token::state::Account::LEN,
            ))
        }
    }

    fn as_associated_token_account(
        &self,
        owner: &Pubkey,
        mint: &Pubkey,
    ) -> Result<spl_token::state::Account, ProgramError> {
        self.has_address(&spl_associated_token_account::get_associated_token_address(
            owner, mint,
        ))?
        .as_token_account()
    }
}

#[cfg(feature = "spl")]
impl AccountValidation for spl_token::state::Mint {
    fn assert<F>(&self, condition: F) -> Result<&Self, ProgramError>
    where
        F: Fn(&Self) -> bool,
    {
        if !condition(self) {
            return Err(solana_program::program_error::ProgramError::InvalidAccountData);
        }
        Ok(self)
    }

    fn assert_err<F>(
        &self,
        condition: F,
        err: solana_program::program_error::ProgramError,
    ) -> Result<&Self, solana_program::program_error::ProgramError>
    where
        F: Fn(&Self) -> bool,
    {
        if !condition(self) {
            return Err(err);
        }
        Ok(self)
    }

    fn assert_msg<F>(&self, condition: F, msg: &str) -> Result<&Self, ProgramError>
    where
        F: Fn(&Self) -> bool,
    {
        match crate::assert(
            condition(self),
            solana_program::program_error::ProgramError::InvalidAccountData,
            msg,
        ) {
            Err(err) => Err(err.into()),
            Ok(()) => Ok(self),
        }
    }

    fn assert_mut<F>(&mut self, _condition: F) -> Result<&mut Self, ProgramError>
    where
        F: Fn(&Self) -> bool,
    {
        panic!("not implemented")
    }

    fn assert_mut_err<F>(
        &mut self,
        _condition: F,
        _err: solana_program::program_error::ProgramError,
    ) -> Result<&mut Self, solana_program::program_error::ProgramError>
    where
        F: Fn(&Self) -> bool,
    {
        panic!("not implemented")
    }

    fn assert_mut_msg<F>(&mut self, _condition: F, _msg: &str) -> Result<&mut Self, ProgramError>
    where
        F: Fn(&Self) -> bool,
    {
        panic!("not implemented")
    }
}

#[cfg(feature = "spl")]
impl AccountValidation for spl_token::state::Account {
    fn assert<F>(&self, condition: F) -> Result<&Self, ProgramError>
    where
        F: Fn(&Self) -> bool,
    {
        if !condition(self) {
            return Err(solana_program::program_error::ProgramError::InvalidAccountData);
        }
        Ok(self)
    }

    fn assert_err<F>(
        &self,
        condition: F,
        err: solana_program::program_error::ProgramError,
    ) -> Result<&Self, solana_program::program_error::ProgramError>
    where
        F: Fn(&Self) -> bool,
    {
        if !condition(self) {
            return Err(err);
        }
        Ok(self)
    }

    fn assert_msg<F>(&self, condition: F, msg: &str) -> Result<&Self, ProgramError>
    where
        F: Fn(&Self) -> bool,
    {
        match crate::assert(
            condition(self),
            solana_program::program_error::ProgramError::InvalidAccountData,
            msg,
        ) {
            Err(err) => Err(err.into()),
            Ok(()) => Ok(self),
        }
    }

    fn assert_mut<F>(&mut self, _condition: F) -> Result<&mut Self, ProgramError>
    where
        F: Fn(&Self) -> bool,
    {
        panic!("not implemented")
    }

    fn assert_mut_err<F>(
        &mut self,
        _condition: F,
        _err: solana_program::program_error::ProgramError,
    ) -> Result<&mut Self, solana_program::program_error::ProgramError>
    where
        F: Fn(&Self) -> bool,
    {
        panic!("not implemented")
    }

    fn assert_mut_msg<F>(&mut self, _condition: F, _msg: &str) -> Result<&mut Self, ProgramError>
    where
        F: Fn(&Self) -> bool,
    {
        panic!("not implemented")
    }
}
