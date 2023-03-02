#![cfg_attr(not(feature = "std"), no_std)]

pub mod errors;
pub mod expense;
pub mod group;
pub mod input_models;

#[ink::contract]
mod splitmate {
    use crate::errors::ContractError;
    use crate::expense::{DistributionMember, DistributionType, Expense};
    use crate::group::{Group, GroupMember};
    use crate::input_models::ExpenseInput;
    use ink::prelude::vec::Vec;
    use ink::storage::Mapping;

    type BaseResult = Result<(), ContractError>;

    #[ink(storage)]
    pub struct Splitmate {
        groups: Mapping<u128, Group>,                 // Group ID -> Group
        group_expenses: Mapping<u128, Vec<Expense>>,  // Group ID -> Expenses
        member_groups: Mapping<AccountId, Vec<u128>>, // Account -> Group IDs
        next_group_id: u128,
    }

    impl Splitmate {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                groups: Mapping::default(),
                group_expenses: Mapping::default(),
                member_groups: Mapping::default(),
                next_group_id: 1,
            }
        }

        #[ink(message)]
        pub fn add_group(&mut self, members_to_add: Vec<AccountId>) -> BaseResult {
            let next_group_id = self.next_group_id.clone();
            let mut new_group_members = Vec::<GroupMember>::new();

            for member_to_add in members_to_add {
                let mut member_to_add_groups = self
                    .member_groups
                    .get(member_to_add)
                    .unwrap_or(Vec::<u128>::new());
                member_to_add_groups.push(next_group_id);
                self.member_groups
                    .insert(member_to_add, &member_to_add_groups);

                new_group_members.push(GroupMember {
                    member_address: member_to_add,
                    debt_value: 0,
                });
            }

            let new_group = Group {
                id: next_group_id,
                members: new_group_members,
                next_expense_id: 1,
            };

            self.groups.insert(next_group_id, &new_group);
            self.next_group_id = self.next_group_id.checked_add(1).unwrap();

            Ok(())
        }

        #[ink(message)]
        pub fn add_expense(&mut self, expense_to_add: ExpenseInput) -> BaseResult {
            let mut group = self.check_group_membership(expense_to_add.group)?;
            let expense = Expense::new(group.next_expense_id.clone(), expense_to_add);

            self.validate_expense(&expense)?;
            self.process_expense_debts(&mut group, &expense)?;

            let mut group_expenses = self
                .group_expenses
                .get(expense.group_id)
                .unwrap_or(Vec::<Expense>::new());
            group_expenses.push(expense.clone());
            self.group_expenses
                .insert(expense.group_id, &group_expenses);

            group.next_expense_id = group.next_expense_id.checked_add(1).unwrap();
            self.groups.insert(expense.group_id, &group);

            Ok(())
        }

        #[ink(message)]
        pub fn get_group(&self, group_id: u128) -> Group {
            // ToDo: Add group existence check
            self.groups.get(group_id).unwrap()
        }

        #[ink(message)]
        pub fn get_expenses_by_group(&self, group_id: u128) -> Vec<Expense> {
            // ToDo: Add group existence check
            self.group_expenses.get(group_id).unwrap()
        }

        pub fn check_group_membership(&self, group_id: u128) -> Result<Group, ContractError> {
            // ToDo: Encapsulate this
            let group = self.groups.get(group_id);
            if group.is_none() {
                return Err(ContractError::GroupDoesNotExist);
            }

            let group = group.unwrap();

            return match self.member_groups.get(self.env().caller()) {
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

        pub fn validate_expense(&self, expense: &Expense) -> BaseResult {
            if expense.amount == 0 {
                return Err(ContractError::ExpenseAmountIsZero);
            }

            if expense.members.len() == 0 {
                return Err(ContractError::ExpenseWithoutDistributionMembers);
            }

            // ToDo: Add the following validations ->
            // - Sum of payers must be equal to total_amount
            // - Sum of split.members.amount must be equal to total_amount

            Ok(())
        }

        pub fn process_expense_debts(
            &mut self,
            group: &mut Group,
            expense: &Expense,
        ) -> BaseResult {
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
                group.members.swap_remove(group_member_index);

                // Calculate how much the member has to pay
                let amount_to_pay = self.calculate_amount_to_pay_by_member(
                    expense,
                    expense_distribution_member.clone(),
                );

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

        pub fn calculate_amount_to_pay_by_member(
            &self,
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
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::test::{
            callee, default_accounts, set_account_balance, set_caller, DefaultAccounts,
        };
        use ink::env::DefaultEnvironment;

        fn get_contract_account_id() -> AccountId {
            callee::<DefaultEnvironment>()
        }

        fn get_default_accounts() -> DefaultAccounts<DefaultEnvironment> {
            default_accounts::<DefaultEnvironment>()
        }

        fn set_sender(sender: AccountId) {
            set_caller::<DefaultEnvironment>(sender);
        }

        fn set_balance(balance: u128) {
            set_account_balance::<DefaultEnvironment>(get_contract_account_id(), balance)
        }

        fn init() -> (Splitmate, DefaultAccounts<DefaultEnvironment>) {
            (Splitmate::new(), get_default_accounts())
        }
    }
}
