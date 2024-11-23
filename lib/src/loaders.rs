use bytemuck::Pod;
use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::{find_program_address, Pubkey},
};
use pinocchio_system::instructions::Transfer;
#[cfg(feature = "spl")]
use solana_program::program_pack::Pack;

use crate::{
    AccountDeserialize, AccountInfoValidation, AsAccount, CloseAccount, Discriminator,
    LamportTransfer,
};
#[cfg(feature = "spl")]
use crate::{AccountValidation, AsSplToken};

impl AccountInfoValidation for AccountInfo {
    fn assert_signer(&self) -> Result<&Self, ProgramError> {
        if !self.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }
        Ok(self)
    }

    fn assert_writable(&self) -> Result<&Self, ProgramError> {
        if !self.is_writable() {
            return Err(ProgramError::MissingRequiredSignature);
        }
        Ok(self)
    }

    fn assert_executable(&self) -> Result<&Self, ProgramError> {
        if !self.executable() {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(self)
    }

    fn assert_empty(&self) -> Result<&Self, ProgramError> {
        if !self.data_is_empty() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }
        Ok(self)
    }

    fn assert_program(&self, program_id: &Pubkey) -> Result<&Self, ProgramError> {
        self.assert_key(program_id)?.assert_executable()
    }

    fn assert_type<T: Discriminator>(&self, program_id: &Pubkey) -> Result<&Self, ProgramError> {
        self.assert_owner(program_id)?;
        if self.try_borrow_data()?[0].ne(&T::discriminator()) {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(self)
    }

    fn assert_owner(&self, owner: &Pubkey) -> Result<&Self, ProgramError> {
        if self.owner().ne(owner) {
            return Err(ProgramError::InvalidAccountOwner);
        }
        Ok(self)
    }

    fn assert_key(&self, address: &Pubkey) -> Result<&Self, ProgramError> {
        if self.key().ne(address) {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(self)
    }

    fn assert_seeds(&self, seeds: &[&[u8]], program_id: &Pubkey) -> Result<&Self, ProgramError> {
        let pda = find_program_address(seeds, program_id);
        if self.key().ne(&pda.0) {
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
    fn as_account<T>(&self, program_id: &Pubkey) -> Result<&T, ProgramError>
    where
        T: AccountDeserialize + Discriminator + Pod,
    {
        unsafe {
            self.assert_owner(program_id)?;
            T::try_from_bytes(std::slice::from_raw_parts(
                self.try_borrow_data()?.as_ptr(),
                8 + std::mem::size_of::<T>(),
            ))
        }
    }

    fn as_account_mut<T>(&self, program_id: &Pubkey) -> Result<&mut T, ProgramError>
    where
        T: AccountDeserialize + Discriminator + Pod,
    {
        unsafe {
            self.assert_owner(program_id)?;
            T::try_from_bytes_mut(std::slice::from_raw_parts_mut(
                self.try_borrow_mut_data()?.as_mut_ptr(),
                8 + std::mem::size_of::<T>(),
            ))
        }
    }
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
