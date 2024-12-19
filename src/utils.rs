use crate::*;

impl Contract {
    pub(crate) fn assert_paused(&self) {
        assert!(!self.paused, "Contact paused");
    }

    pub(crate) fn assert_operator(&self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.operator_id,
            "ERR_NOT_AN_OPERATOR"
        );
    }
}

pub(crate) fn remaining_gas() -> Gas {
    Gas::from_gas(env::prepaid_gas().as_gas() - env::used_gas().as_gas())
}

pub(crate) fn is_valid_string(input: &str) -> bool {
    input.chars().all(|c| c.is_ascii_lowercase())
}
