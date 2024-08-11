use crate::process_state::ProcessState;
use rand::{thread_rng, Rng};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Process {
    pub pid: Option<u8>,
    pub p_burst_time: u8,
    pub p_state: ProcessState,
    pub p_arrival_time: u8,
    pub p_remaining_time: u8,
}
impl Process {
    pub fn new() -> Process {
        let burst_time = thread_rng().gen_range(1..8);
        Self {
            pid: Self::pid_assigner(),
            p_burst_time: burst_time,
            p_state: ProcessState::Ready,
            p_arrival_time: thread_rng().gen_range(1..7),
            p_remaining_time: burst_time,
        }
    }

    fn pid_assigner() -> Option<u8> {
        Some(thread_rng().gen::<u8>())
    }
}

impl Default for Process {
    fn default() -> Self {
        Self::new()
    }
}
