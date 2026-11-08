use crate::model::Closing;
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Choice {
    Cancel,
    Background,
    Stop,
}
pub fn preference(previous: Closing, choice: Choice, remember: bool) -> Closing {
    if !remember {
        return previous;
    }
    match choice {
        Choice::Cancel => previous,
        Choice::Background => Closing::Background,
        Choice::Stop => Closing::Stop,
    }
}
#[derive(Default)]
pub struct SaveGate {
    pending: Option<u64>,
}
impl SaveGate {
    pub fn wait(&mut self, generation: u64) {
        self.pending = Some(generation);
    }
    pub fn acknowledge(&mut self, generation: u64) -> bool {
        if self.pending == Some(generation) {
            self.pending = None;
            true
        } else {
            false
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_checked_committing_actions_change_the_close_preference() {
        for old in [Closing::Ask, Closing::Stop, Closing::Background] {
            for remember in [false, true] {
                assert_eq!(preference(old, Choice::Cancel, remember), old);
                assert_eq!(
                    preference(old, Choice::Stop, remember),
                    if remember { Closing::Stop } else { old }
                );
                assert_eq!(
                    preference(old, Choice::Background, remember),
                    if remember { Closing::Background } else { old }
                );
            }
        }
    }
    #[test]
    fn an_older_completed_save_cannot_authorize_shutdown() {
        let mut gate = SaveGate::default();
        gate.wait(7);
        assert!(!gate.acknowledge(6));
        assert!(gate.acknowledge(7));
        assert!(!gate.acknowledge(7));
    }
}
