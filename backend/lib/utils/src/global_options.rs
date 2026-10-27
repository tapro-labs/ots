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
        Self::init_from_env().unwrap().with_build_version_fallback()
    }
}

impl GlobalOptions {
    fn with_build_version_fallback(mut self) -> Self {
        if self.build_version.is_empty() {
            // envconfig does not support compile time envs
            // so we default to that here if we don't have a build version
            self.build_version = option_env!("OTS_BUILD_VERSION")
                .unwrap_or("0.0.0")
                .to_owned();
        }

        self
    }

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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn options_from(vars: &[(&str, &str)]) -> GlobalOptions {
        let map: HashMap<String, String> = vars
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        GlobalOptions::init_from_hashmap(&map).unwrap()
    }

    fn compile_time_version() -> &'static str {
        option_env!("OTS_BUILD_VERSION").unwrap_or("0.0.0")
    }

    #[test]
    fn test_all_options_default_to_empty_when_unset() {
        let options = options_from(&[]);

        assert_eq!(options.frontend_server_url, "");
        assert_eq!(options.backend_server_url, "");
        assert_eq!(options.build_version, "");
    }

    #[test]
    fn test_options_are_read_from_their_variables() {
        let options = options_from(&[
            ("FRONTEND_SERVER_URL", "https://ots.example.com"),
            ("BACKEND_SERVER_URL", "https://api.ots.example.com"),
            ("OTS_BUILD_VERSION", "1.2.3"),
        ]);

        assert_eq!(options.frontend_server_url, "https://ots.example.com");
        assert_eq!(options.backend_server_url, "https://api.ots.example.com");
        assert_eq!(options.build_version, "1.2.3");
    }

    #[test]
    fn test_runtime_build_version_is_kept() {
        let options = options_from(&[("OTS_BUILD_VERSION", "9.9.9")]).with_build_version_fallback();

        assert_eq!(options.build_version, "9.9.9");
    }

    #[test]
    fn test_empty_build_version_falls_back_to_compile_time_version() {
        let options = options_from(&[("OTS_BUILD_VERSION", "")]).with_build_version_fallback();

        assert_eq!(options.build_version, compile_time_version());
        assert!(!options.build_version.is_empty());
    }

    #[test]
    fn test_default_always_has_a_build_version() {
        assert!(!GlobalOptions::default().build_version.is_empty());
    }

    #[test]
    fn test_exactly_one_of_dev_and_prod_is_true() {
        let options = options_from(&[]);

        assert_ne!(options.is_dev(), options.is_prod());
        assert_eq!(options.is_dev(), cfg!(debug_assertions));
    }
}
