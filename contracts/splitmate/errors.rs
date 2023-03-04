#[derive(PartialEq, Debug, Eq, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum ContractError {
    MemberIsNotInTheGroup,
    ExpenseDistributionMemberIsNotInTheGroup,
    ExpenseAmountIsZero,
    ExpenseWithoutPayers,
    ExpenseWithoutMembers,
    ExpenseWithoutDistributionMembers,
    GroupDoesNotExist,
    TransferError,
}
