use autobattler::dev_tools::profiling::{profile, profile_threaded};

fn main() {
    profile_threaded(60, 4); // run random tests for this many seconds.   
}

#[cfg(test)]
pub mod tests {
    use autobattler::dev_tools::profiling::{profile, profile_threaded};

    #[test]
    pub fn profile_test() {
        profile(10);
    }

    #[test]
    pub fn multithreaded_test() {
        profile_threaded(10u64, 4);
    }
}