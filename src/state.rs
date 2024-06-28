extern crate lazy_static;

use crate::model::box_entity::BoxEntityWrapper;
use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};

lazy_static! {
    pub static ref SELECTED_BOX: Arc<Mutex<Option<BoxEntityWrapper>>> = Arc::new(Mutex::new(None));
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_initialization() {
        // Add test for state initialization
    }
}
