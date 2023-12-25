use envconfig::Envconfig;
use redis::ConnectionAddr::Tcp;
use redis::{
    Client, Commands, Connection, ConnectionInfo, FromRedisValue, RedisConnectionInfo,
    RedisResult as RResult, ToRedisArgs,
};
use utils::time::Time;

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

impl RedisStore {
    pub fn connect(config: RedisConfig) -> Self {
        let client = Client::open(ConnectionInfo {
            addr: Tcp(config.host, config.port),
            redis: RedisConnectionInfo {
                password: config.password,
                ..Default::default()
            },
        })
        .unwrap();
        let con = client.get_connection().unwrap();

        Self { connection: con }
    }

    pub fn connect_default() -> Self {
        Self::connect(RedisConfig::init_from_env().unwrap())
    }
}

impl RedisStore {
    pub fn get<T: FromRedisValue, K: ToRedisArgs>(&mut self, key: K) -> RedisResult<T> {
        self.connection.get(key)
    }

    pub fn store<K: ToRedisArgs, V: ToRedisArgs>(&mut self, key: K, value: V) -> RedisResult<()> {
        self.connection.set(key, value)
    }

    pub fn store_for_a_period<K: ToRedisArgs, V: ToRedisArgs>(
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

    pub fn get_and_forget<T: FromRedisValue, K: ToRedisArgs>(&mut self, key: K) -> RedisResult<T> {
        let value = self.get(&key);

        self.forget(&key).unwrap_or(());

        value
    }
}
