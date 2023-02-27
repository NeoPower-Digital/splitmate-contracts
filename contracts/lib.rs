#![cfg_attr(not(feature = "std"), no_std)]

pub mod errors;
pub mod expense;
pub mod group;

#[ink::contract]
mod splitmate {
    use crate::errors::ContractError;
    use crate::expense::Expense;
    use crate::group::{Group, GroupMember};
    use ink::prelude::vec::Vec;
    use ink::storage::Mapping;

    type BaseResult = Result<(), ContractError>;

    #[ink(storage)]
    pub struct Splitmate {
        groups: Mapping<u128, Group>,                 // Group ID -> Group
        expenses: Mapping<u128, Vec<Expense>>,        // Group ID -> Expenses
        member_groups: Mapping<AccountId, Vec<u128>>, // Account -> Group IDs
        next_group_id: u128,
    }

    impl Splitmate {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                groups: Mapping::default(),
                expenses: Mapping::default(),
                member_groups: Mapping::default(),
                next_group_id: 0,
            }
        }

        #[ink(message)]
        pub fn add_group(&mut self, members_to_add: Vec<AccountId>) -> BaseResult {
            let next_group_id = self.next_group_id.clone();
            let mut new_group_members = Vec::<GroupMember>::new();

            for member in members_to_add {
                let mut member_groups =
                    self.member_groups.get(member).unwrap_or(Vec::<u128>::new());
                member_groups.push(next_group_id);
                self.member_groups.insert(member, &member_groups);

                new_group_members.push(GroupMember {
                    member,
                    debt_value: 0,
                });
            }

            let new_group = Group {
                id: self.next_group_id.clone(),
                members: new_group_members,
            };

            self.groups.insert(self.next_group_id.clone(), &new_group);
            self.next_group_id = self.next_group_id.checked_add(1).unwrap();

            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::expense::{Split, SplitType};
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
