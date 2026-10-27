pub mod redis_store;

use utils::time::Time;

pub trait SecretStore: Send {
    fn store_for_a_period(&mut self, key: String, value: &str, time: Time) -> Result<(), String>;
    fn get_and_forget(&mut self, key: String) -> Result<String, String>;
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    fn assert_rocket_state<T: Send + Sync + 'static>() {}

    #[test]
    fn test_boxed_store_in_mutex_can_be_rocket_managed_state() {
        // The server keeps `Mutex<Box<dyn SecretStore>>` in Rocket state, which
        // needs `Send + Sync + 'static`. This breaks at compile time if the
        // `Send` bound is removed from the trait.
        assert_rocket_state::<Mutex<Box<dyn SecretStore>>>();
        assert_rocket_state::<Mutex<Box<redis_store::RedisStore>>>();
    }
}
