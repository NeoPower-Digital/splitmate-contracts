#![cfg_attr(not(feature = "std"), no_std)]

pub mod errors;
pub mod expense;
pub mod group;
pub mod input_models;
pub mod output_models;
pub mod utils;

#[ink::contract]
mod splitmate {
    use crate::errors::ContractError;
    use crate::expense::Expense;
    use crate::group::{Group, GroupMember};
    use crate::input_models::{ExpenseInput, GroupDebtsToPay};
    use crate::output_models::{GroupSettledDebts, MemberAccount, SettleUpResult};
    use crate::utils::{
        add_to_member_groups, check_group_membership, get_group_by_id,
        get_member_group_distributions, get_member_groups, process_expense_debts,
        update_member_group_debt, BaseResult,
    };
    use ink::prelude::{string::String, vec::Vec};
    use ink::storage::Mapping;
    use openbrush::contracts::traits::psp22::PSP22Ref;

    #[ink(storage)]
    pub struct Splitmate {
        /// ERC20 token address
        pub token_address: AccountId,
        /// Mapping Group ID -> Group object
        pub groups: Mapping<u128, Group>,
        /// Mapping Group ID -> Group expenses
        pub group_expenses: Mapping<u128, Vec<Expense>>,
        /// Mapping Member -> Group IDs
        pub member_groups: Mapping<AccountId, Vec<u128>>,
        /// Group ID incremental
        pub next_group_id: u128,
    }

    impl Splitmate {
        #[ink(constructor)]
        pub fn new(token_address: AccountId) -> Self {
            Self {
                token_address,
                groups: Mapping::default(),
                group_expenses: Mapping::default(),
                member_groups: Mapping::default(),
                next_group_id: 1,
            }
        }

        /// Adds a new group with the caller as the first member.
        /// Includes a group name and a caller representative name.
        /// Initializes the caller with zero debts.
        /// Adds the group ID to the Mapping Member -> Group IDs.
        #[ink(message)]
        pub fn add_group(&mut self, group_name: String, caller_name: String) -> BaseResult {
            let caller_address = self.env().caller();
            let next_group_id = self.next_group_id.clone();

            let new_group_members = [GroupMember {
                address: caller_address,
                name: caller_name,
                debt_value: 0,
            }]
            .to_vec();

            let new_group = Group::new(next_group_id, group_name, new_group_members);
            self.groups.insert(next_group_id, &new_group);

            add_to_member_groups(self, caller_address, next_group_id);

            self.next_group_id = self.next_group_id.checked_add(1).unwrap();

            Ok(())
        }

        /// Adds the caller to the group corresponding to the group_id.
        /// Includes a caller representative name.
        /// Initializes the caller with zero debts.
        /// Adds the group ID to the Mapping Member -> Group IDs.
        #[ink(message)]
        pub fn join_group(&mut self, group_id: u128, caller_name: String) -> BaseResult {
            let caller_address = self.env().caller();
            let mut group = get_group_by_id(&self, group_id)?;

            group.members.push(GroupMember {
                address: caller_address,
                name: caller_name,
                debt_value: 0,
            });

            self.groups.insert(group_id, &group);

            add_to_member_groups(self, caller_address, group_id);

            Ok(())
        }

        /// Adds an expense to a specific group.
        /// Checks if the caller is in the specified group.
        /// Validates the expense values.
        /// Updates the member balances/debts.
        /// Adds the expense to the Mapping Group ID -> Group Expenses.
        /// Updates the Group Expense ID incremental.
        #[ink(message)]
        pub fn add_expense(&mut self, expense_to_add: ExpenseInput) -> BaseResult {
            let mut group = check_group_membership(&self, expense_to_add.group_id)?;
            let expense = Expense::new(group.next_expense_id.clone(), expense_to_add);

            expense.validate()?;
            process_expense_debts(&mut group, &expense)?;

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

        /// Settles up selected debts for specific groups.
        /// Transfers ERC20 tokens for each debt.  
        /// Updates the group debts.
        /// Informs if all the debts were paid.     
        #[ink(message)]
        pub fn settle_up(
            &mut self,
            debts_to_pay: Vec<GroupDebtsToPay>,
        ) -> Result<SettleUpResult, ContractError> {
            // ToDo: Add debts validation
            let mut total_settled_debts = Vec::<GroupSettledDebts>::new();

            for group_debts_to_pay in debts_to_pay {
                let mut group = check_group_membership(&self, group_debts_to_pay.group_id)?;

                let mut group_settled_debt_amount: u128 = 0;
                let mut group_settled_debts = GroupSettledDebts {
                    group_id: group.id,
                    takers: Vec::<AccountId>::new(),
                };

                for taker in group_debts_to_pay.takers {
                    // ToDo: Check account balances before transfers
                    if PSP22Ref::transfer(
                        &mut self.token_address,
                        taker.member_address,
                        taker.value,
                        Vec::new(),
                    )
                    .is_err()
                    {
                        update_member_group_debt(
                            &mut group,
                            self.env().caller(),
                            false,
                            group_settled_debt_amount,
                        );

                        total_settled_debts.push(group_settled_debts);

                        return Ok(SettleUpResult {
                            result: false,
                            total_settled_debts: Some(total_settled_debts),
                        });
                    };

                    update_member_group_debt(&mut group, taker.member_address, true, taker.value);

                    group_settled_debt_amount =
                        group_settled_debt_amount.checked_add(taker.value).unwrap();

                    group_settled_debts.takers.push(taker.member_address);
                }

                update_member_group_debt(
                    &mut group,
                    self.env().caller(),
                    false,
                    group_settled_debt_amount,
                );
                total_settled_debts.push(group_settled_debts);
            }

            Ok(SettleUpResult {
                result: true,
                total_settled_debts: None,
            })
        }

        /// Gets the specified member groups and debts.
        #[ink(message)]
        pub fn get_member_account(&self) -> Result<MemberAccount, ContractError> {
            let caller = self.env().caller();
            let groups = get_member_groups(&self, caller)?;
            let debts_by_group = get_member_group_distributions(&self, caller)?;

            Ok(MemberAccount {
                groups,
                debts_by_group,
            })
        }

        /// Gets the specified group data.
        #[ink(message)]
        pub fn get_group(&self, group_id: u128) -> Result<Group, ContractError> {
            check_group_membership(&self, group_id)
        }

        /// Gets all the expenses of the specified group.
        #[ink(message)]
        pub fn get_expenses_by_group(&self, group_id: u128) -> Result<Vec<Expense>, ContractError> {
            check_group_membership(&self, group_id)?;
            Ok(self.group_expenses.get(group_id).unwrap())
        }
    }
}
