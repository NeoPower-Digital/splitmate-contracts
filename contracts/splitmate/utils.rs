use ink::codegen::Env;
use ink::prelude::vec::Vec;
use ink::primitives::AccountId;

use crate::{
    errors::ContractError,
    expense::{DistributionType, Expense, ExpenseMember},
    group::{Group, GroupMember},
    output_models::{GroupDistributionByMember, GroupMemberDistributionTransfer},
    splitmate::Splitmate,
};

pub type BaseResult = Result<(), ContractError>;

pub fn process_expense_debts(group: &mut Group, expense: &Expense) -> BaseResult {
    for expense_distribution_member in expense.members.clone() {
        // Check/Get the group member reference and remove it
        let group_member_index = group
            .members
            .iter()
            .position(|group_member| group_member.address == expense_distribution_member.address);
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
    takers: &mut Vec<GroupMember>,
) -> GroupMemberDistributionTransfer {
    // With same balance
    let taker_with_same_balance_amount =
        find_taker_by(takers, |r| r.debt_value.abs() as u128 == debt_value);

    if taker_with_same_balance_amount.is_some() {
        return GroupMemberDistributionTransfer {
            member_account: taker_with_same_balance_amount.unwrap(),
            value: debt_value,
        };
    }

    // With more balance
    let taker_with_more_balance_amount =
        find_taker_by(takers, |r| r.debt_value.abs() as u128 > debt_value);

    if taker_with_more_balance_amount.is_some() {
        return GroupMemberDistributionTransfer {
            member_account: taker_with_more_balance_amount.unwrap(),
            value: debt_value,
        };
    }

    let taker = takers[0].clone();
    takers.remove(0);

    GroupMemberDistributionTransfer {
        member_account: taker.address,
        value: taker.debt_value.abs() as u128,
    }
}

pub fn update_group_debt(
    group: &mut Group,
    member_address: AccountId,
    is_taker: bool,
    amount: u128,
) {
    let member_position = group
        .members
        .iter()
        .position(|m| m.address == member_address)
        .unwrap();
    let mut member = group.members[member_position].clone();
    group.members.remove(member_position);
    member.debt_value = if is_taker {
        member.debt_value.checked_add(amount as i128).unwrap()
    } else {
        member.debt_value.checked_sub(amount as i128).unwrap()
    };
    group.members.push(member)
}

pub fn calculate_amount_to_pay_by_member(expense: &Expense, split_member: ExpenseMember) -> u128 {
    return match expense.distribution_type {
        DistributionType::EQUALLY => expense
            .amount
            .checked_div(expense.members.len() as u128)
            .unwrap(),
        DistributionType::UNEQUALLY => split_member.must_pay,
    };
}

pub fn find_taker_by<P>(takers: &mut Vec<GroupMember>, predicate: P) -> Option<AccountId>
where
    P: FnMut(&GroupMember) -> bool,
{
    let taker_index = takers.iter().position(predicate);

    if taker_index.is_some() {
        let taker = takers[taker_index.unwrap()].clone();
        takers.remove(taker_index.unwrap());

        return Some(taker.address);
    }

    None
}

pub fn check_group_membership(
    instance: &Splitmate,
    group_id: u128,
) -> Result<Group, ContractError> {
    let group = get_group_by_id(instance, group_id)?;

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

pub fn get_group_by_id(instance: &Splitmate, group_id: u128) -> Result<Group, ContractError> {
    match instance.groups.get(group_id) {
        Some(group) => Ok(group),
        None => Err(ContractError::GroupDoesNotExist),
    }
}

pub fn get_member_group_distributions(
    instance: &Splitmate,
    member_address: AccountId,
) -> Result<Vec<GroupDistributionByMember>, ContractError> {
    let member_groups = instance
        .member_groups
        .get(member_address)
        .unwrap_or(Vec::<u128>::new());

    let mut caller_distributions = Vec::<GroupDistributionByMember>::new();

    for group_id in member_groups {
        let group_distribution = instance.get_group_distribution(group_id)?;
        let group_distribution_by_caller = group_distribution
            .iter()
            .find(|gmd| gmd.member_account == member_address)
            .unwrap();
        caller_distributions.push(GroupDistributionByMember {
            group_id,
            member_distribution: group_distribution_by_caller.clone(),
        });
    }

    Ok(caller_distributions)
}

pub fn get_member_groups(
    instance: &Splitmate,
    member_address: AccountId,
) -> Result<Vec<Group>, ContractError> {
    let group_ids = instance.member_groups.get(member_address);
    let mut member_groups = Vec::<Group>::new();

    if group_ids.is_none() {
        return Ok(member_groups);
    }

    for group_id in group_ids.unwrap() {
        member_groups.push(instance.groups.get(group_id).unwrap());
    }

    Ok(member_groups)
}

pub fn add_to_member_groups(instance: &mut Splitmate, member_address: AccountId, group_id: u128) {
    let mut member_groups = instance
        .member_groups
        .get(member_address)
        .unwrap_or(Vec::<u128>::new());

    member_groups.push(group_id);

    instance
        .member_groups
        .insert(member_address, &member_groups);
}
