use ink::prelude::{string::String, vec::Vec};
use ink::primitives::AccountId;
use ink::storage::traits::StorageLayout;

#[derive(PartialEq, Debug, Eq, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
pub struct GroupMember {
    pub address: AccountId,
    pub name: String,
    pub debt_value: i128,
}

#[derive(PartialEq, Debug, Eq, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
pub struct Group {
    pub id: u128,
    pub name: String,
    pub members: Vec<GroupMember>,
    pub next_expense_id: u32,
}

impl Group {
    pub fn new(id: u128, name: String, members: Vec<GroupMember>) -> Group {
        Group {
            id,
            name,
            members,
            next_expense_id: 1,
        }
    }
}
