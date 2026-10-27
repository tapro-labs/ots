use envconfig::Envconfig;
use redis::ConnectionAddr::Tcp;
use redis::{
    Client, Commands, Connection, ConnectionInfo, FromRedisValue, IntoConnectionInfo,
    RedisConnectionInfo, RedisResult as RResult, ToRedisArgs, ToSingleRedisArg,
};
use utils::time::Time;

use crate::SecretStore;

pub type RedisResult<T> = RResult<T>;

#[derive(Envconfig)]
pub struct RedisConfig {
    #[envconfig(from = "REDIS_HOST", default = "127.0.0.1")]
    host: String,

    #[envconfig(from = "REDIS_PASSWORD")]
    password: Option<String>,

    #[envconfig(from = "REDIS_PORT", default = "6379")]
    port: u16,
}

pub struct RedisStore {
    connection: Connection,
}

fn connection_info(config: RedisConfig) -> ConnectionInfo {
    let mut redis_settings = RedisConnectionInfo::default();

    // An empty REDIS_PASSWORD (e.g. `REDIS_PASSWORD=` in an env file) means
    // "no password". Sending AUTH with an empty password fails.
    if let Some(pw) = config.password.filter(|value| !value.is_empty()) {
        redis_settings = redis_settings.set_password(pw);
    }

    Tcp(config.host, config.port)
        .into_connection_info()
        .expect("a TCP address is always a valid connection info")
        .set_redis_settings(redis_settings)
}

impl RedisStore {
    pub fn try_connect(config: RedisConfig) -> RedisResult<Self> {
        let client = Client::open(connection_info(config))?;
        let connection = client.get_connection()?;

        Ok(Self { connection })
    }

    pub fn connect(config: RedisConfig) -> Self {
        Self::try_connect(config).unwrap()
    }

    pub fn connect_default() -> Self {
        Self::connect(RedisConfig::init_from_env().unwrap())
    }
}

impl RedisStore {
    pub fn get<T: FromRedisValue, K: ToSingleRedisArg>(&mut self, key: K) -> RedisResult<T> {
        self.connection.get(key)
    }

    pub fn store<K: ToSingleRedisArg, V: ToSingleRedisArg>(
        &mut self,
        key: K,
        value: V,
    ) -> RedisResult<()> {
        self.connection.set(key, value)
    }

    pub fn store_for_a_period<K: ToSingleRedisArg, V: ToSingleRedisArg>(
        &mut self,
        key: K,
        value: V,
        time: Time,
    ) -> RedisResult<()> {
        self.store(&key, value)?;

        // We try to cast to i64
        // we check here for this edge case, as new redis versions require i64
        // instead of u64 or usize for some reason
        if let Ok(expiration) = time.as_ms().try_into() {
            return self.connection.pexpire(&key, expiration);
        }

        panic!("It seems we reached the end of time...");
    }

    pub fn forget<K: ToRedisArgs>(&mut self, key: K) -> RedisResult<()> {
        self.connection.del(key)
    }

    pub fn get_and_forget<T: FromRedisValue, K: ToSingleRedisArg>(
        &mut self,
        key: K,
    ) -> RedisResult<T> {
        // GETDEL (Redis 6.2+) reads and deletes in one atomic step, so two
        // concurrent readers cannot both receive the same one-time secret.
        self.connection.get_del(key)
    }
}

impl SecretStore for RedisStore {
    fn store_for_a_period(&mut self, key: String, value: &str, time: Time) -> Result<(), String> {
        RedisStore::store_for_a_period(self, key, value, time).map_err(|e| e.to_string())
    }

    fn get_and_forget(&mut self, key: String) -> Result<String, String> {
        RedisStore::get_and_forget(self, key).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::net::TcpListener;

    use utils::uuid::Uuid;

    use super::*;

    fn config_from(vars: &[(&str, &str)]) -> Result<RedisConfig, envconfig::Error> {
        let map: HashMap<String, String> = vars
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        RedisConfig::init_from_hashmap(&map)
    }

    fn password_of(info: &ConnectionInfo) -> Option<&str> {
        info.redis_settings().password()
    }

    // --- RedisConfig ---

    #[test]
    fn test_config_defaults_when_nothing_is_set() {
        let config = config_from(&[]).unwrap();

        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 6379);
        assert!(config.password.is_none());
    }

    #[test]
    fn test_config_reads_all_variables() {
        let config = config_from(&[
            ("REDIS_HOST", "redis"),
            ("REDIS_PORT", "6380"),
            ("REDIS_PASSWORD", "hunter2"),
        ])
        .unwrap();

        assert_eq!(config.host, "redis");
        assert_eq!(config.port, 6380);
        assert_eq!(config.password.as_deref(), Some("hunter2"));
    }

    #[test]
    fn test_config_rejects_non_numeric_port() {
        assert!(config_from(&[("REDIS_PORT", "redis")]).is_err());
    }

    #[test]
    fn test_config_rejects_out_of_range_port() {
        assert!(config_from(&[("REDIS_PORT", "65536")]).is_err());
        assert!(config_from(&[("REDIS_PORT", "-1")]).is_err());
    }

    // --- connection_info ---

    #[test]
    fn test_connection_info_uses_tcp_host_and_port() {
        let config = config_from(&[("REDIS_HOST", "redis"), ("REDIS_PORT", "6380")]).unwrap();
        let info = connection_info(config);

        assert_eq!(*info.addr(), Tcp("redis".to_string(), 6380));
        assert_eq!(info.redis_settings().db(), 0);
        assert!(info.redis_settings().username().is_none());
    }

    #[test]
    fn test_connection_info_passes_password() {
        let config = config_from(&[("REDIS_PASSWORD", "hunter2")]).unwrap();

        assert_eq!(password_of(&connection_info(config)), Some("hunter2"));
    }

    #[test]
    fn test_connection_info_without_password() {
        let config = config_from(&[]).unwrap();

        assert_eq!(password_of(&connection_info(config)), None);
    }

    #[test]
    fn test_connection_info_treats_empty_password_as_no_password() {
        // Regression: `REDIS_PASSWORD=` gave Some(""), so the client sent
        // `AUTH ""` and the connection failed against a Redis without a password.
        let config = config_from(&[("REDIS_PASSWORD", "")]).unwrap();

        assert_eq!(password_of(&connection_info(config)), None);
    }

    // --- try_connect ---

    fn unused_local_port() -> u16 {
        // Bind to port 0 to get a free port, then release it so nothing listens on it.
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.local_addr().unwrap().port()
    }

    #[test]
    fn test_try_connect_returns_error_when_server_is_unreachable() {
        let port = unused_local_port().to_string();
        let config = config_from(&[("REDIS_PORT", port.as_str())]).unwrap();

        let error = RedisStore::try_connect(config)
            .err()
            .expect("connection must fail");

        assert!(error.is_connection_refusal() || error.is_io_error());
    }

    #[test]
    #[should_panic]
    fn test_connect_panics_when_server_is_unreachable() {
        let port = unused_local_port().to_string();
        let config = config_from(&[("REDIS_PORT", port.as_str())]).unwrap();

        RedisStore::connect(config);
    }

    // --- Live Redis tests ---
    //
    // These need a running Redis server. They read REDIS_HOST, REDIS_PORT and
    // REDIS_PASSWORD like the app does. Run them with:
    //
    //   docker run --rm -p 6379:6379 redis
    //   cargo test -p store -- --ignored

    fn live_store() -> RedisStore {
        RedisStore::connect_default()
    }

    fn unique_key() -> String {
        format!("ots-test:{}", Uuid::new_v4())
    }

    fn pttl(store: &mut RedisStore, key: &str) -> i64 {
        redis::cmd("PTTL")
            .arg(key)
            .query(&mut store.connection)
            .unwrap()
    }

    #[test]
    #[ignore = "needs a running Redis server"]
    fn test_live_store_for_a_period_sets_value_and_expiry() {
        let mut store = live_store();
        let key = unique_key();

        store
            .store_for_a_period(&key, "value", Time::from_minutes(5))
            .unwrap();

        let value: String = store.get(&key).unwrap();
        assert_eq!(value, "value");

        let ttl = pttl(&mut store, &key);
        assert!(ttl > 0 && ttl <= 300_000, "unexpected ttl {}", ttl);

        store.forget(&key).unwrap();
    }

    #[test]
    #[ignore = "needs a running Redis server"]
    fn test_live_store_for_a_period_with_zero_time_removes_the_key() {
        let mut store = live_store();
        let key = unique_key();

        store
            .store_for_a_period(&key, "value", Time::from_seconds(0))
            .unwrap();

        let value: Option<String> = store.get(&key).unwrap();
        assert!(value.is_none());
    }

    #[test]
    #[ignore = "needs a running Redis server"]
    fn test_live_get_and_forget_returns_value_only_once() {
        let mut store = live_store();
        let key = unique_key();
        store.store(&key, "value").unwrap();

        let first: String = store.get_and_forget(&key).unwrap();
        let second: Option<String> = store.get_and_forget(&key).unwrap();

        assert_eq!(first, "value");
        assert!(second.is_none());
    }

    #[test]
    #[ignore = "needs a running Redis server"]
    fn test_live_secret_store_round_trip() {
        let mut store: Box<dyn SecretStore> = Box::new(live_store());
        let key = unique_key();

        store
            .store_for_a_period(key.clone(), "secret", Time::from_minutes(5))
            .unwrap();

        assert_eq!(store.get_and_forget(key.clone()).unwrap(), "secret");
        // A missing key cannot be read as a String, so a second read is an error.
        assert!(store.get_and_forget(key).is_err());
    }
}
