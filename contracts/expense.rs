use ink::prelude::vec::Vec;
use ink::primitives::AccountId;
use ink::storage::traits::StorageLayout; //ink_storage::collections::Vec

#[derive(PartialEq, Debug, Eq, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
pub enum SplitType {
    EQUALLY,
    UNEQUALLY,
}

#[derive(PartialEq, Debug, Eq, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
pub struct Expense {
    pub total_amount: u128,
    pub payers: Vec<(AccountId, u128)>,
    pub members: Vec<AccountId>,
    pub split: Split,
}

#[derive(PartialEq, Debug, Eq, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
pub struct Split {
    pub split_type: SplitType,
    pub members_split: Vec<(AccountId, u128)>,
}
