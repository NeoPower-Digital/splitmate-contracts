#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod my_psp22 {
    use openbrush::contracts::psp22::*;
    use openbrush::traits::Storage;

    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct USDNP {
        #[storage_field]
        psp22: psp22::Data,
    }

    impl PSP22 for USDNP {}

    impl USDNP {
        #[ink(constructor)]
        pub fn new(initial_supply: Balance) -> Self {
            let mut _instance = Self::default();
            _instance
                ._mint_to(_instance.env().caller(), initial_supply)
                .expect("Should mint");
            _instance
        }
    }
}
