use crate::entities::BoxEntity;
use std::sync::Mutex;

pub static SELECTED_BOX: Mutex<Option<BoxEntity>> = Mutex::new(None);
