use autobattler::dev_tools::profiling::profile;

fn main() {
    profile(60); // run random tests for this many seconds.   
}

#[cfg(test)]
pub mod tests {
    use autobattler::dev_tools::profiling::profile;

    #[test]
    pub fn profile_test() {
        profile(60);
    }
}