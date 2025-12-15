#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug)]
pub(crate) enum TargetParadigm {
    // common targeting paradigms.
    Me,            // caster
    CurrentTarget, // self explanatory.
    Nearest(u8),   // returns the n nearest targets
    Furthest(u8)   // returns the n furthest targets
}
