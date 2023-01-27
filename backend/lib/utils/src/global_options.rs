use envconfig::Envconfig;

#[derive(Envconfig)]
pub struct GlobalOptions {
    #[envconfig(from = "FRONTEND_SERVER_URL", default = "")]
    pub frontend_server_url: String,

    #[envconfig(from = "BACKEND_SERVER_URL", default = "")]
    pub backend_server_url: String,

    #[envconfig(from = "OTS_BUILD_VERSION", default = "")]
    pub build_version: String,
}

impl Default for GlobalOptions {
    fn default() -> Self {
        let mut instance = Self::init_from_env().unwrap();

        if instance.build_version.is_empty() {
            // envconfig does not support compile time envs
            // so we default to that here if we don't have a build version
            instance.build_version = option_env!("OTS_BUILD_VERSION").unwrap_or("0.0.0").to_owned();
        }

        instance
    }
}

impl GlobalOptions {
    pub fn is_dev(&self) -> bool {
        #[cfg(debug_assertions)]
        {
            true
        }
        #[cfg(not(debug_assertions))]
        {
            false
        }
    }

    pub fn is_prod(&self) -> bool {
        #[cfg(debug_assertions)]
        {
            false
        }
        #[cfg(not(debug_assertions))]
        {
            true
        }
    }
}
