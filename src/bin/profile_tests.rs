use autobattler::dev_tools::profiling::*;

fn main() {
    profile_threaded(60, 4); // run random tests for x seconds with y threads.   
}

#[cfg(test)]
pub mod tests {
    use autobattler::dev_tools::profiling::*;

    #[test]
    pub fn profile_test() {
        profile(10);
    }

    #[test]
    pub fn profile_log_test() {
        profile_with_event_log(10)
    }

    #[test]
    pub fn multithreaded_test() {
        profile_threaded(10u64, 4);
    }
}