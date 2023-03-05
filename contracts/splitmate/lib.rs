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
    use crate::output_models::{
        GroupMemberDistribution, GroupMemberDistributionTransfer, GroupSettledDebts, MemberAccount,
        SettleUpResult,
    };
    use crate::utils::{
        add_to_member_groups, check_group_membership, get_group_by_id,
        get_member_group_distributions, get_member_groups, process_expense_debts,
        process_giver_debt, update_group_debt, BaseResult,
    };
    use ink::prelude::{string::String, vec::Vec};
    use ink::storage::Mapping;
    use openbrush::contracts::traits::psp22::PSP22Ref;

    #[ink(storage)]
    pub struct Splitmate {
        pub token_address: AccountId,
        pub groups: Mapping<u128, Group>, // Group ID -> Group
        pub group_expenses: Mapping<u128, Vec<Expense>>, // Group ID -> Expenses
        pub member_groups: Mapping<AccountId, Vec<u128>>, // Account -> Group IDs
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

        #[ink(message)]
        pub fn join_group(&mut self, group_id: u128, member_name: String) -> BaseResult {
            let caller_address = self.env().caller();
            let mut group = get_group_by_id(&self, group_id)?;

            group.members.push(GroupMember {
                address: caller_address,
                name: member_name,
                debt_value: 0,
            });

            self.groups.insert(group_id, &group);

            add_to_member_groups(self, caller_address, group_id);

            Ok(())
        }

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

        #[ink(message)]
        pub fn get_group_distribution(
            &self,
            group_id: u128,
        ) -> Result<Vec<GroupMemberDistribution>, ContractError> {
            let group = check_group_membership(&self, group_id)?;
            let mut group_distribution = Vec::<GroupMemberDistribution>::new();

            let givers: Vec<GroupMember> = group
                .members
                .clone()
                .into_iter()
                .filter(|m| m.debt_value > 0)
                .collect();

            if givers.len() == 0 {
                return Ok(group_distribution);
            }

            let mut takers: Vec<GroupMember> = group
                .members
                .clone()
                .into_iter()
                .filter(|m| m.debt_value < 0)
                .collect();
            takers.sort_by(|a, b| a.debt_value.cmp(&b.debt_value));

            for giver in givers {
                let mut distribution_member = GroupMemberDistribution {
                    member_account: giver.address,
                    total_debt: giver.debt_value,
                    transfers: Vec::<GroupMemberDistributionTransfer>::new(),
                };

                let mut pending_debt = giver.debt_value as u128;
                while pending_debt > 0 {
                    let debt_transfer = process_giver_debt(pending_debt, &mut takers);

                    distribution_member.transfers.push(debt_transfer.clone());

                    pending_debt = pending_debt.checked_sub(debt_transfer.value).unwrap();
                }

                group_distribution.push(distribution_member);
            }

            Ok(group_distribution)
        }

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

        #[ink(message)]
        pub fn get_group(&self, group_id: u128) -> Result<Group, ContractError> {
            check_group_membership(&self, group_id)
        }

        #[ink(message)]
        pub fn get_expenses_by_group(&self, group_id: u128) -> Result<Vec<Expense>, ContractError> {
            check_group_membership(&self, group_id)?;
            Ok(self.group_expenses.get(group_id).unwrap())
        }

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
                    if PSP22Ref::transfer(
                        &mut self.token_address,
                        taker.member_address,
                        taker.value,
                        Vec::new(),
                    )
                    .is_err()
                    {
                        update_group_debt(
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

                    update_group_debt(&mut group, taker.member_address, true, taker.value);

                    group_settled_debt_amount =
                        group_settled_debt_amount.checked_add(taker.value).unwrap();

                    group_settled_debts.takers.push(taker.member_address);
                }

                update_group_debt(
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
            (Splitmate::new("asd"), get_default_accounts())
        }

        #[ink::test]
        fn get_distribution_works() {
            // Arrange
            let (mut contract, accounts) = init();
            set_sender(accounts.alice);

            let group = Group {
                id: 1,
                name: "test".to_str(),
                members: vec![
                    GroupMember {
                        address: accounts.alice,
                        name: "Alice".to_str(),
                        debt_value: 100,
                    },
                    GroupMember {
                        address: accounts.bob,
                        name: "Bob".to_str(),
                        debt_value: -50,
                    },
                    GroupMember {
                        address: accounts.charlie,
                        name: "Charlie".to_str(),
                        debt_value: -20,
                    },
                    GroupMember {
                        address: accounts.django,
                        name: "Django".to_str(),
                        debt_value: -30,
                    },
                    GroupMember {
                        address: accounts.eve,
                        name: "Eve".to_str(),
                        debt_value: -45,
                    },
                    GroupMember {
                        address: accounts.frank,
                        name: "Frank".to_str(),
                        debt_value: -5,
                    },
                ],
                next_expense_id: 1,
            };

            contract.groups.insert(1, &group);
            contract.member_groups.insert(accounts.alice, &vec![1]);

            // Act
            let distribution_summary = contract.get_group_distribution(1).unwrap();

            // Assert
            assert_eq!(distribution_summary.len(), 1);
            assert_eq!(distribution_summary[0].total_debt, 100);
            assert_eq!(distribution_summary[0].transfers.len(), 3);
        }
    }
}
