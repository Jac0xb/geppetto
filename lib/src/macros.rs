#[macro_export]
macro_rules! account {
    ($discriminator_name:ident, $struct_name:ident) => {
        impl $struct_name
        where
            Self: borsh::BorshSerialize,
        {
            pub fn to_bytes(&self) -> Vec<u8> {
                borsh::to_vec(self).unwrap()
            }
        }

        impl $crate::Discriminator for $struct_name {
            fn discriminator() -> u8 {
                $discriminator_name::$struct_name.into()
            }
        }

        impl $crate::AccountValidation for $struct_name {
            fn assert<F>(
                &self,
                condition: F,
            ) -> Result<&Self, pinocchio::program_error::ProgramError>
            where
                F: Fn(&Self) -> bool,
            {
                if !condition(self) {
                    return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
                }
                Ok(self)
            }

            fn assert_err<F>(
                &self,
                condition: F,
                err: pinocchio::program_error::ProgramError,
            ) -> Result<&Self, pinocchio::program_error::ProgramError>
            where
                F: Fn(&Self) -> bool,
            {
                if !condition(self) {
                    return Err(err);
                }
                Ok(self)
            }

            fn assert_msg<F>(
                &self,
                condition: F,
                msg: &str,
            ) -> Result<&Self, pinocchio::program_error::ProgramError>
            where
                F: Fn(&Self) -> bool,
            {
                match $crate::assert(
                    condition(self),
                    pinocchio::program_error::ProgramError::InvalidAccountData,
                    msg,
                ) {
                    Err(err) => Err(err.into()),
                    Ok(()) => Ok(self),
                }
            }

            fn assert_mut<F>(
                &mut self,
                condition: F,
            ) -> Result<&mut Self, pinocchio::program_error::ProgramError>
            where
                F: Fn(&Self) -> bool,
            {
                if !condition(self) {
                    return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
                }
                Ok(self)
            }

            fn assert_mut_err<F>(
                &mut self,
                condition: F,
                err: pinocchio::program_error::ProgramError,
            ) -> Result<&mut Self, pinocchio::program_error::ProgramError>
            where
                F: Fn(&Self) -> bool,
            {
                if !condition(self) {
                    return Err(err);
                }
                Ok(self)
            }

            fn assert_mut_msg<F>(
                &mut self,
                condition: F,
                msg: &str,
            ) -> Result<&mut Self, pinocchio::program_error::ProgramError>
            where
                F: Fn(&Self) -> bool,
            {
                match $crate::assert(
                    condition(self),
                    pinocchio::program_error::ProgramError::InvalidAccountData,
                    msg,
                ) {
                    Err(err) => Err(err.into()),
                    Ok(()) => Ok(self),
                }
            }
        }
    };
}

#[macro_export]
macro_rules! error {
    ($struct_name:ident) => {
        impl From<$struct_name> for pinocchio::program_error::ProgramError {
            fn from(e: $struct_name) -> Self {
                pinocchio::program_error::ProgramError::Custom(e as u32)
            }
        }
    };
}

#[macro_export]
macro_rules! event {
    ($struct_name:ident) => {
        impl $struct_name
        where
            Self: borsh::BorshSerialize,
        {
            pub fn to_bytes(&self) -> Vec<u8> {
                borsh::to_vec(self).unwrap()
            }
        }

        impl $crate::Loggable for $struct_name {
            fn log(&self) {
                pinocchio::log::sol_log_data(&[self.to_bytes().as_slice()]);
            }

            fn log_return(&self) {
                pinocchio::program::set_return_data(self.to_bytes().as_slice());
            }
        }
    };
}

#[macro_export]
macro_rules! bytemuck_instruction {
    ($discriminator_name:ident, $struct_name:ident) => {
        $crate::impl_instruction_from_bytes!($struct_name);

        impl $crate::Discriminator for $struct_name {
            fn discriminator() -> u8 {
                $discriminator_name::$struct_name as u8
            }
        }

        impl $struct_name {
            pub fn to_bytes(&self) -> Vec<u8> {
                [
                    [$discriminator_name::$struct_name as u8].to_vec(),
                    bytemuck::bytes_of(self).to_vec(),
                ]
                .concat()
            }
        }
    };
}

#[macro_export]
macro_rules! borsh_instruction {
    ($discriminator_name:ident, $struct_name:ident) => {
        impl $crate::Discriminator for $struct_name {
            fn discriminator() -> u8 {
                $discriminator_name::$struct_name as u8
            }
        }

        // TODO: Vectors are horrible in SVM land :(
        impl $struct_name
        where
            Self: borsh::BorshSerialize,
            Self: borsh::BorshDeserialize,
        {
            pub fn try_from_bytes(
                data: &[u8],
            ) -> Result<Self, pinocchio::program_error::ProgramError> {
                <Self>::try_from_slice(data).or(Err(
                    pinocchio::program_error::ProgramError::InvalidInstructionData,
                ))
            }

            pub fn to_bytes(&self) -> Vec<u8> {
                [
                    [$discriminator_name::$struct_name as u8].to_vec(),
                    borsh::to_vec(self).unwrap(),
                ]
                .concat()
            }
        }
    };
}
