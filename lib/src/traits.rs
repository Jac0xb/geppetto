use bytemuck::Pod;
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

pub trait AccountDeserialize {
    fn try_from_bytes(data: &[u8]) -> Result<&Self, ProgramError>;
    fn try_from_bytes_mut(data: &mut [u8]) -> Result<&mut Self, ProgramError>;
}

impl<T> AccountDeserialize for T
where
    T: Discriminator + Pod,
{
    fn try_from_bytes(data: &[u8]) -> Result<&Self, ProgramError> {
        if Self::discriminator().ne(&data[0]) {
            return Err(ProgramError::InvalidAccountData);
        }
        bytemuck::try_from_bytes::<Self>(&data[8..]).or(Err(ProgramError::InvalidAccountData))
    }

    fn try_from_bytes_mut(data: &mut [u8]) -> Result<&mut Self, ProgramError> {
        if Self::discriminator().ne(&data[0]) {
            return Err(ProgramError::InvalidAccountData);
        }
        bytemuck::try_from_bytes_mut::<Self>(&mut data[8..])
            .or(Err(ProgramError::InvalidAccountData))
    }
}

/// Account data is sometimes stored via a header and body type,
/// where the former resolves the type of the latter (e.g. merkle trees with a generic size const).
/// This trait parses a header type from the first N bytes of some data, and returns the remaining
/// bytes, which are then available for further processing.
///
/// See module-level tests for example usage.
pub trait AccountHeaderDeserialize {
    fn try_header_from_bytes(data: &[u8]) -> Result<(&Self, &[u8]), ProgramError>;
    fn try_header_from_bytes_mut(data: &mut [u8]) -> Result<(&mut Self, &mut [u8]), ProgramError>;
}

impl<T> AccountHeaderDeserialize for T
where
    T: Discriminator + Pod,
{
    fn try_header_from_bytes(data: &[u8]) -> Result<(&Self, &[u8]), ProgramError> {
        if Self::discriminator().ne(&data[0]) {
            return Err(ProgramError::InvalidAccountData);
        }
        let (prefix, remainder) = data[8..].split_at(std::mem::size_of::<T>());
        Ok((
            bytemuck::try_from_bytes::<Self>(prefix).or(Err(ProgramError::InvalidAccountData))?,
            remainder,
        ))
    }

    fn try_header_from_bytes_mut(data: &mut [u8]) -> Result<(&mut Self, &mut [u8]), ProgramError> {
        let (prefix, remainder) = data[8..].split_at_mut(std::mem::size_of::<T>());
        Ok((
            bytemuck::try_from_bytes_mut::<Self>(prefix)
                .or(Err(ProgramError::InvalidAccountData))?,
            remainder,
        ))
    }
}

pub trait AccountValidation {
    fn assert<F>(&self, condition: F) -> Result<&Self, ProgramError>
    where
        F: Fn(&Self) -> bool;

    fn assert_err<F>(&self, condition: F, err: ProgramError) -> Result<&Self, ProgramError>
    where
        F: Fn(&Self) -> bool;

    fn assert_msg<F>(&self, condition: F, msg: &str) -> Result<&Self, ProgramError>
    where
        F: Fn(&Self) -> bool;

    fn assert_mut<F>(&mut self, condition: F) -> Result<&mut Self, ProgramError>
    where
        F: Fn(&Self) -> bool;

    fn assert_mut_err<F>(
        &mut self,
        condition: F,
        err: ProgramError,
    ) -> Result<&mut Self, ProgramError>
    where
        F: Fn(&Self) -> bool;

    fn assert_mut_msg<F>(&mut self, condition: F, msg: &str) -> Result<&mut Self, ProgramError>
    where
        F: Fn(&Self) -> bool;
}

pub trait AccountInfoValidation {
    fn assert_signer(&self) -> Result<&Self, ProgramError>;
    fn assert_writable(&self) -> Result<&Self, ProgramError>;
    fn assert_executable(&self) -> Result<&Self, ProgramError>;
    fn assert_empty(&self) -> Result<&Self, ProgramError>;
    fn assert_type<T: Discriminator>(&self, program_id: &Pubkey) -> Result<&Self, ProgramError>;
    fn assert_program(&self, program_id: &Pubkey) -> Result<&Self, ProgramError>;
    // fn is_sysvar(&self, sysvar_id: &Pubkey) -> Result<&Self, ProgramError>;
    fn assert_key(&self, address: &Pubkey) -> Result<&Self, ProgramError>;
    fn assert_owner(&self, program_id: &Pubkey) -> Result<&Self, ProgramError>;
    fn assert_seeds(&self, seeds: &[&[u8]], program_id: &Pubkey) -> Result<&Self, ProgramError>;
}

pub trait Discriminator {
    fn discriminator() -> u8;
}

/// Performs:
/// 1. Program owner check
/// 2. Discriminator byte check
/// 3. Checked bytemuck conversion of account data to &T or &mut T.
pub trait AsAccount {
    fn as_account<T>(&self, program_id: &Pubkey) -> Result<&T, ProgramError>
    where
        T: AccountDeserialize + Discriminator + Pod;

    fn as_account_mut<T>(&self, program_id: &Pubkey) -> Result<&mut T, ProgramError>
    where
        T: AccountDeserialize + Discriminator + Pod;
}

#[cfg(feature = "spl")]
pub trait AsSplToken {
    fn as_mint(&self) -> Result<spl_token::state::Mint, ProgramError>;
    fn as_token_account(&self) -> Result<spl_token::state::Account, ProgramError>;
    fn as_associated_token_account(
        &self,
        owner: &Pubkey,
        mint: &Pubkey,
    ) -> Result<spl_token::state::Account, ProgramError>;
}

// TODO Work in progress
pub trait LamportTransfer<'a> {
    fn send(&'a self, lamports: u64, to: &'a AccountInfo) -> Result<(), ProgramError>;
    fn collect(&'a self, lamports: u64, from: &'a AccountInfo) -> Result<(), ProgramError>;
}

pub trait CloseAccount<'a> {
    fn close(&'a self, to: &'a AccountInfo) -> Result<(), ProgramError>;
}

pub trait Loggable {
    fn log(&self);
    fn log_return(&self);
}

pub trait ProgramOwner {
    fn owner() -> Pubkey;
}
