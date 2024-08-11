#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ProcessState {
    InExec,
    Complete,
    Ready,
}
