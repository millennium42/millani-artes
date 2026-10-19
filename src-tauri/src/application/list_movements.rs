use crate::domain::Movement;

#[derive(Debug)]
pub struct ListMovementsError;

impl std::fmt::Display for ListMovementsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "movements could not be read")
    }
}

impl std::error::Error for ListMovementsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

pub trait MovementReadStore {
    type Error;
    fn read_all_movements(&self) -> Result<Vec<Movement>, Self::Error>;
}

pub fn list_movements(store: &impl MovementReadStore) -> Result<Vec<Movement>, ListMovementsError> {
    store.read_all_movements().map_err(|_| ListMovementsError)
}

#[cfg(test)]
#[path = "list_movements_tests.rs"]
mod tests;
