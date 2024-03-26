pub mod retrievers;
use std::collections::hash_map::DefaultHasher;
pub mod types;
pub use retrievers::*;
use std::hash::Hasher;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub use types::*;

type FlagConfigType = Arc<Mutex<HashMap<String, FlagConfig>>>;

pub struct FlagService {
    flag_config: FlagConfigType,
}

fn wait_for_lock(
    flag_config: &FlagConfigType,
) -> Option<std::sync::MutexGuard<'_, HashMap<String, FlagConfig>>> {
    let mut i = 0;
    loop {
        match flag_config.try_lock() {
            Ok(lock) => {
                return Some(lock);
            }
            Err(_) => {
                std::thread::sleep(std::time::Duration::from_millis(1));
                i += 1;
                if i > 100 {
                    return None;
                }
            }
        }
    }
}

fn update_config(flag_config: FlagConfigType, config: HashMap<String, FlagConfig>) {
    match wait_for_lock(&flag_config) {
        Some(mut fc) => {
            fc.clear();
            for (key, value) in config {
                fc.insert(key, value);
            }
        }
        None => {
            println!("Could not get lock");
        }
    }
}

fn reload_config(flag_config: FlagConfigType, opts: &FlagServiceOptions) {
    match opts.finder_type {
        types::FlagFinderType::URL => {
            let retriever = URLRetriever::new(opts.url.as_ref().unwrap());
            let config = retriever.retrieve();
            if config.is_some() {
                update_config(flag_config, config.unwrap())
            }
        }
        types::FlagFinderType::JSON => {
            let retriever = JSONStringRetriever::new(opts.data.as_ref().unwrap().clone());
            let config = retriever.retrieve();
            if config.is_some() {
                update_config(flag_config, config.unwrap())
            }
        }
        types::FlagFinderType::ENVVAR => {
            let retriever = JSONEnvVarRetriever::new(opts.env_var.as_ref().unwrap().clone());
            let config = retriever.retrieve();
            if config.is_some() {
                update_config(flag_config, config.unwrap())
            }
        }
        _ => {}
    }
}

fn reload_config_forever(flag_config: FlagConfigType, opts: &FlagServiceOptions) {
    loop {
        std::thread::sleep(std::time::Duration::from_secs(opts.refresh_interval));
        reload_config(flag_config.clone(), opts);
    }
}

impl FlagService {
    pub fn new(options: FlagServiceOptions) -> FlagService {
        let real_config = Arc::new(Mutex::new(HashMap::new()));
        let svc = FlagService {
            flag_config: Arc::clone(&real_config),
        };

        reload_config(Arc::clone(&svc.flag_config), &options.clone());
        if options.refresh_interval > 0 {
            let fc = Arc::clone(&svc.flag_config);
            let opts = options.clone();
            std::thread::spawn(move || {
                reload_config_forever(fc, &opts);
            });
        }
        svc
    }

    pub fn get_flag_config(&self) -> HashMap<String, FlagConfig> {
        match wait_for_lock(&self.flag_config) {
            Some(fc) => fc.clone(),
            None => {
                println!("Could not get lock");
                return HashMap::new();
            }
        }
    }

    pub fn enabled(
        &self,
        name: &str,
        default: bool,
        context: Option<HashMap<String, String>>,
    ) -> bool {
        let fc = self.get_flag_config();
        let config = fc.get(name);
        if let Some(config) = config {
            if config.rollout >= 100 {
                return true;
            } else if config.rollout > 0 {
                let mut hasher = DefaultHasher::new();
                hasher.write(name.as_bytes());
                if context.is_some() {
                    for (key, value) in context.as_ref().unwrap() {
                        hasher.write(key.as_bytes());
                        hasher.write(value.as_bytes());
                    }
                }
                let hash = hasher.finish();
                if let Ok(bucket) = u8::try_from(hash % 100) {
                    return bucket < config.rollout;
                } else {
                    println!("Error converting hash to u8: {}", hash)
                }
            } else if context.is_some()
                && config.variants.is_some()
                && config.variants.as_ref().unwrap().len() > 0
            {
                let ucontext = context.unwrap();
                for (key, value) in config.variants.as_ref().unwrap() {
                    if ucontext.contains_key(key)
                        && value.iter().any(|v| v == ucontext.get(key).unwrap())
                    {
                        return true;
                    } else {
                        return false;
                    }
                }
            }
        }
        return default;
    }
}

#[cfg(test)]
mod tests {
    use crate::service::FlagService;
    use httptest::matchers::any;
    use httptest::responders::status_code;
    use httptest::{cycle, Expectation, ServerPool};
    static SERVER_POOL: ServerPool = ServerPool::new(2);
    use std::collections::HashMap;

    #[test]
    fn it_url_config_service_works() {
        let server = SERVER_POOL.get_server();
        server.expect(
            Expectation::matching(any()).respond_with(status_code(200).body(
                r#"
{
    "feature_rolled_out": {
        "rollout": 100
    },
    "feature_variant": {
        "rollout": 0,
        "variants": {
            "user_id": ["123"]
        }
    }
}"#,
            )),
        );

        let flag_service = FlagService::new(crate::service::FlagServiceOptions {
            refresh_interval: 0,
            finder_type: crate::types::FlagFinderType::URL,
            url: Some(server.url("/").to_string()),
            env_var: None,
            data: None,
        });

        assert_eq!(
            flag_service.enabled("feature_rolled_out", false, None),
            true
        );
        assert_eq!(
            flag_service.enabled(
                "feature_variant",
                false,
                Some(HashMap::from([("user_id".to_string(), "123".to_string()),]))
            ),
            true
        );
        assert_eq!(
            flag_service.enabled(
                "feature_variant",
                false,
                Some(HashMap::from(
                    [("user_id".to_string(), "1234".to_string()),]
                ))
            ),
            false
        );
    }

    #[test]
    fn it_hashes_context_rollout() {
        let server = SERVER_POOL.get_server();
        server.expect(
            Expectation::matching(any()).respond_with(status_code(200).body(
                r#"
{"feature": {"rollout": 50}}"#,
            )),
        );

        let flag_service = FlagService::new(crate::service::FlagServiceOptions {
            refresh_interval: 0,
            finder_type: crate::types::FlagFinderType::URL,
            url: Some(server.url("/").to_string()),
            env_var: None,
            data: None,
        });

        assert_eq!(
            flag_service.enabled(
                "feature",
                false,
                Some(HashMap::from(
                    [("user_id".to_string(), "1234".to_string()),]
                ))
            ),
            true
        );
        assert_eq!(
            flag_service.enabled(
                "feature",
                true,
                Some(HashMap::from([("user_id".to_string(), "123".to_string())]))
            ),
            false
        );
    }

    #[test]
    fn reloads_config() {
        let server = SERVER_POOL.get_server();
        server.expect(Expectation::matching(any()).times(2..).respond_with(cycle![
            status_code(200).body(r#"{"feature": {"rollout": 100}}"#),
            status_code(200).body(r#"{"feature": {"rollout": 0}}"#)
        ]));

        let flag_service = FlagService::new(crate::service::FlagServiceOptions {
            refresh_interval: 1,
            finder_type: crate::types::FlagFinderType::URL,
            url: Some(server.url("/").to_string()),
            env_var: None,
            data: None,
        });

        assert_eq!(flag_service.enabled("feature", false, None), true);

        std::thread::sleep(
            std::time::Duration::from_secs(1) + std::time::Duration::from_millis(100),
        );
        assert_eq!(flag_service.enabled("feature", false, None), false);
    }
}
