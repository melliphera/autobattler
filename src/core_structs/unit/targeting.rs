#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug)]
pub(crate) enum TargetParadigm {
    // common targeting paradigms.
    Me,            // caster
    CurrentTarget, // self explanatory.
    Nearest(u8),   // returns the n nearest targets
    Furthest(u8)   // returns the n furthest targets
}

impl TargetParadigm {
    pub(crate) fn get_target_count(&self) -> u8 {
        match self {
         TargetParadigm::Nearest(n) | TargetParadigm::Furthest(n) => {*n}
         _ => {1}
        }
    }
}