use borsh::{BorshDeserialize, BorshSerialize};

#[derive( BorshSerialize, BorshDeserialize, Debug )]
pub struct UserBalance{
    pub balance:u64,
}