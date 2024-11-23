mod cpi;
mod loaders;
pub mod macros;
mod traits;
mod utils;

use borsh::{BorshDeserialize, BorshSerialize};
pub use cpi::*;
pub use traits::*;
pub use utils::*;

pub use bytemuck::{Pod, Zeroable};
pub use num_enum::{IntoPrimitive, TryFromPrimitive};
pub use thiserror::Error;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
pub enum MyAccount {
    Counter = 0,
    Profile = 1,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, BorshDeserialize, BorshSerialize)]
pub struct Counter {
    pub value: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, BorshDeserialize, BorshSerialize)]
pub struct Profile {
    pub id: u64,
}

account!(MyAccount, Counter);
account!(MyAccount, Profile);

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum MyInstruction {
    Add = 0,
    Initialize = 1,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, BorshDeserialize, BorshSerialize)]
pub struct Add {
    pub value: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, BorshDeserialize, BorshSerialize)]
pub struct Initialize {}

borsh_instruction!(MyInstruction, Add);
borsh_instruction!(MyInstruction, Initialize);

#[repr(u32)]
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq, IntoPrimitive)]
pub enum MyError {
    #[error("You did something wrong")]
    Dummy = 0,
}

error!(MyError);

#[repr(C)]
#[derive(Clone, Copy, Debug, BorshDeserialize, BorshSerialize)]
pub struct MyEvent {
    pub value: u64,
}

event!(MyEvent);
