pub mod redis_store;

use utils::time::Time;

pub trait SecretStore: Send {
    fn store_for_a_period(&mut self, key: String, value: &str, time: Time) -> Result<(), String>;
    fn get_and_forget(&mut self, key: String) -> Result<String, String>;
}
