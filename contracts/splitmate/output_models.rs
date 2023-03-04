use ink::prelude::vec::Vec;
use ink::primitives::AccountId;
use ink::storage::traits::StorageLayout;

#[derive(PartialEq, Debug, Eq, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
pub struct DistributionMemberSummary {
    pub member_account: AccountId,
    pub total_debt: i128,
    pub debts: Vec<DistributionMemberTransfer>,
}

#[derive(PartialEq, Debug, Eq, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
pub struct DistributionMemberTransfer {
    pub member_account: AccountId,
    pub debt_value: u128,
}

#[derive(PartialEq, Debug, Eq, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
pub struct SettledDebts {
    pub group_id: u128,
    pub receivers: Vec<AccountId>,
}

#[derive(PartialEq, Debug, Eq, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
pub struct SettleUpResult {
    pub result: bool,
    pub settled_debts: Option<Vec<SettledDebts>>,
}
