use std::time::SystemTime;

pub trait Clock {
    fn now(&self) -> SystemTime;
}

/// Adapters define UUID version, uniqueness, and generation failures.
pub trait UuidGenerator {
    type Error;

    fn generate(&mut self) -> std::result::Result<[u8; 16], Self::Error>;
}
