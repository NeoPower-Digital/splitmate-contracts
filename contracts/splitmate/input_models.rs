use crate::expense::DistributionType;
use ink::prelude::vec::Vec;
use ink::primitives::AccountId;
use ink::storage::traits::StorageLayout;

#[derive(PartialEq, Debug, Eq, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
pub struct ExpenseInput {
    pub group_id: u128,
    pub amount: u128,
    pub payer_address: AccountId,
    pub distribution: DistributionInput,
}

#[derive(PartialEq, Debug, Eq, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
pub struct DistributionInput {
    pub distribution_type: DistributionType,
    pub distribution_by_members: Vec<DistributionByMemberInput>,
}

#[derive(PartialEq, Debug, Eq, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
pub struct DistributionByMemberInput {
    pub member_address: AccountId,
    pub value: u128,
}

#[derive(PartialEq, Debug, Eq, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
pub struct GroupDebtsToPay {
    pub group_id: u128,
    pub receivers: Vec<DistributionByMemberInput>,
}
