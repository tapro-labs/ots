use envconfig::Envconfig;

#[derive(Envconfig)]
pub struct GlobalOptions {
    #[envconfig(from = "FRONTEND_SERVER_URL", default = "")]
    pub frontend_server_url: String,

    #[envconfig(from = "BACKEND_SERVER_URL", default = "")]
    pub backend_server_url: String,
}

impl Default for GlobalOptions {
    fn default() -> Self {
        Self::init_from_env().unwrap()
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
