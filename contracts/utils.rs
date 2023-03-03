use ink::codegen::Env;
use ink::prelude::vec::Vec;
use ink::primitives::AccountId;

use crate::{
    errors::ContractError,
    expense::{DistributionMember, DistributionType, Expense},
    group::{Group, GroupMember},
    output_models::DistributionMemberTransfer,
    splitmate::Splitmate,
};

pub type BaseResult = Result<(), ContractError>;

pub fn process_expense_debts(group: &mut Group, expense: &Expense) -> BaseResult {
    for expense_distribution_member in expense.members.clone() {
        // Check/Get the group member reference and remove it
        let group_member_index = group.members.iter().position(|group_member| {
            group_member.member_address == expense_distribution_member.member_address
        });
        let group_member_index = match group_member_index {
            Some(index) => index,
            None => return Err(ContractError::ExpenseDistributionMemberIsNotInTheGroup),
        };

        let mut group_member = group.members[group_member_index].clone();
        group.members.remove(group_member_index);

        // Calculate how much the member has to pay
        let amount_to_pay =
            calculate_amount_to_pay_by_member(expense, expense_distribution_member.clone());

        // Calculate the difference between the amount the member has to pay and the amount the member paid
        let debt = (amount_to_pay as i128)
            .checked_sub(expense_distribution_member.paid as i128)
            .unwrap();

        // Update the member debt
        group_member.debt_value = group_member.debt_value.checked_add(debt).unwrap();

        group.members.push(group_member);
    }

    Ok(())
}

pub fn process_giver_debt(
    debt_value: u128,
    receivers: &mut Vec<GroupMember>,
) -> DistributionMemberTransfer {
    // With same balance
    let receiver_with_same_balance_amount =
        find_receiver_by(receivers, |r| r.debt_value.abs() as u128 == debt_value);

    if receiver_with_same_balance_amount.is_some() {
        return DistributionMemberTransfer {
            member_account: receiver_with_same_balance_amount.unwrap(),
            debt_value,
        };
    }

    // With more balance
    let receiver_with_more_balance_amount =
        find_receiver_by(receivers, |r| r.debt_value.abs() as u128 > debt_value);

    if receiver_with_more_balance_amount.is_some() {
        return DistributionMemberTransfer {
            member_account: receiver_with_more_balance_amount.unwrap(),
            debt_value,
        };
    }

    let receiver = receivers[0].clone();
    receivers.remove(0);

    DistributionMemberTransfer {
        member_account: receiver.member_address,
        debt_value: receiver.debt_value.abs() as u128,
    }
}

pub fn calculate_amount_to_pay_by_member(
    expense: &Expense,
    split_member: DistributionMember,
) -> u128 {
    return match expense.distribution_type {
        DistributionType::EQUALLY => expense
            .amount
            .checked_div(expense.members.len() as u128)
            .unwrap(),
        DistributionType::UNEQUALLY => split_member.must_pay,
    };
}

pub fn find_receiver_by<P>(receivers: &mut Vec<GroupMember>, predicate: P) -> Option<AccountId>
where
    P: FnMut(&GroupMember) -> bool,
{
    let receiver_index = receivers.iter().position(predicate);

    if receiver_index.is_some() {
        let receiver = receivers[receiver_index.unwrap()].clone();
        receivers.remove(receiver_index.unwrap());

        return Some(receiver.member_address);
    }

    None
}

pub fn check_group_membership(
    instance: &Splitmate,
    group_id: u128,
) -> Result<Group, ContractError> {
    // ToDo: Encapsulate this
    let group = instance.groups.get(group_id);
    if group.is_none() {
        return Err(ContractError::GroupDoesNotExist);
    }

    let group = group.unwrap();

    return match instance.member_groups.get(instance.env().caller()) {
        Some(member_groups) => {
            if !member_groups
                .iter()
                .any(|member_group_id| *member_group_id == group_id)
            {
                return Err(ContractError::MemberIsNotInTheGroup);
            }

            Ok(group)
        }
        None => Err(ContractError::MemberIsNotInTheGroup),
    };
}
