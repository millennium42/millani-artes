use super::ports::{Clock, UuidGenerator};
use std::collections::VecDeque;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const ID_A: [u8; 16] = [
    0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x41, 0x11, 0x81, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
];
const ID_B: [u8; 16] = [
    0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x42, 0x22, 0x82, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22,
];

struct FixedClock(SystemTime);

impl Clock for FixedClock {
    fn now(&self) -> SystemTime {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GenerationFailure {
    Unavailable,
}

struct SequenceUuid {
    remaining: VecDeque<Result<[u8; 16], GenerationFailure>>,
    calls: usize,
}

impl UuidGenerator for SequenceUuid {
    type Error = GenerationFailure;

    fn generate(&mut self) -> Result<[u8; 16], Self::Error> {
        self.calls += 1;
        self.remaining
            .pop_front()
            .expect("synthetic provider exhausted")
    }
}

#[test]
fn clock_can_be_replaced_and_preserves_instants() {
    let past = FixedClock(UNIX_EPOCH - Duration::from_secs(1));
    let future = FixedClock(UNIX_EPOCH + Duration::from_secs(42));
    let injected: &dyn Clock = &past;
    assert_eq!(injected.now(), past.0);
    assert_eq!(injected.now(), past.0);
    let injected: &dyn Clock = &future;
    assert_eq!(injected.now(), future.0);
}

#[test]
fn uuid_provider_preserves_sequence_through_dynamic_injection() {
    let mut provider = SequenceUuid {
        remaining: VecDeque::from([Ok(ID_A), Ok(ID_B)]),
        calls: 0,
    };
    let injected: &mut dyn UuidGenerator<Error = GenerationFailure> = &mut provider;
    assert_eq!(injected.generate(), Ok(ID_A));
    assert_eq!(injected.generate(), Ok(ID_B));
    assert_eq!(provider.calls, 2);
    assert!(provider.remaining.is_empty());
}

#[test]
fn uuid_failure_remains_opaque_without_consuming_a_replacement() {
    let mut provider = SequenceUuid {
        remaining: VecDeque::from([Err(GenerationFailure::Unavailable), Ok(ID_A)]),
        calls: 0,
    };
    let injected: &mut dyn UuidGenerator<Error = GenerationFailure> = &mut provider;
    assert_eq!(injected.generate(), Err(GenerationFailure::Unavailable));
    assert_eq!(provider.calls, 1);
    assert_eq!(provider.remaining.len(), 1);
    assert_eq!(provider.generate(), Ok(ID_A));
    assert_eq!(provider.calls, 2);
}
