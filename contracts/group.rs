use ink::prelude::vec::Vec;
use ink::primitives::AccountId;
use ink::storage::traits::StorageLayout;

#[derive(PartialEq, Debug, Eq, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
pub struct Group {
    pub id: u128,
    pub members: Vec<GroupMember>,
    pub next_expense_id: u32,
}

#[derive(PartialEq, Debug, Eq, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
pub struct GroupMember {
    pub member_address: AccountId,
    pub debt_value: i128,
}
