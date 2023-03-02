use crate::input_models::ExpenseInput;
use ink::prelude::vec::Vec;
use ink::primitives::AccountId;
use ink::storage::traits::StorageLayout;

#[derive(PartialEq, Debug, Eq, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
pub enum DistributionType {
    EQUALLY,
    UNEQUALLY,
}

#[derive(PartialEq, Debug, Eq, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
pub struct DistributionMember {
    pub member_address: AccountId,
    pub paid: u128,
    pub must_pay: u128,
}

#[derive(PartialEq, Debug, Eq, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
pub struct Expense {
    pub id: u32,
    pub group_id: u128,
    pub amount: u128,
    pub distribution_type: DistributionType,
    pub members: Vec<DistributionMember>,
}

impl Expense {
    pub fn new(id: u32, expense_to_add: ExpenseInput) -> Expense {
        let members = expense_to_add
            .distribution
            .distribution_by_members
            .iter()
            .map(|distribution_by_member| {
                let paid_value =
                    if distribution_by_member.member_address == expense_to_add.payer_address {
                        expense_to_add.amount
                    } else {
                        0
                    };

                DistributionMember {
                    member_address: distribution_by_member.member_address,
                    paid: paid_value,
                    must_pay: distribution_by_member.value,
                }
            })
            .collect();

        Expense {
            id,
            group_id: expense_to_add.group,
            amount: expense_to_add.amount,
            distribution_type: expense_to_add.distribution.distribution_type,
            members,
        }
    }
}
