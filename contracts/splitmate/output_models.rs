use ink::prelude::vec::Vec;
use ink::primitives::AccountId;
use ink::storage::traits::StorageLayout;

#[derive(PartialEq, Debug, Eq, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
pub struct GroupMemberDistribution {
    pub member_account: AccountId,
    pub total_debt: i128,
    pub transfers: Vec<GroupMemberDistributionTransfer>,
}

#[derive(PartialEq, Debug, Eq, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
pub struct GroupMemberDistributionTransfer {
    pub member_account: AccountId,
    pub value: u128,
}

#[derive(PartialEq, Debug, Eq, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
pub struct GroupSettledDebts {
    pub group_id: u128,
    pub takers: Vec<AccountId>,
}

#[derive(PartialEq, Debug, Eq, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
pub struct SettleUpResult {
    pub result: bool,
    pub total_settled_debts: Option<Vec<GroupSettledDebts>>,
}

#[derive(PartialEq, Debug, Eq, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
pub struct GroupDistributionByMember {
    pub group_id: u128,
    pub member_distribution: GroupMemberDistribution,
}
