use crate::service::types::FlagConfig;
use std::collections::HashMap;

pub trait FlagRetriever {
    fn retrieve(&self) -> Option<HashMap<String, FlagConfig>>;
}

fn parse_json_config(json: &str) -> Option<HashMap<String, FlagConfig>> {
    let json_parsed = serde_json::from_str(json);
    if json_parsed.is_err() {
        print!("Error parsing JSON: {}", json);
        return None;
    }
    let json: serde_json::Value = json_parsed.unwrap();
    let json = json
        .as_object()
        .expect("Expected a JSON object at the root");

    let mut config: HashMap<String, FlagConfig> = HashMap::new();
    for (key, value) in json {
        let rollout = value["rollout"].as_u64().unwrap() as u8;

        // let mut variants: Option<HashMap<String, Vec<String>> = None;
        let mut variants: Option<HashMap<String, Vec<String>>> = None;
        if let Some(json_variants) = value.get("variants") {
            let json_variants = json_variants
                .as_object()
                .expect("Expected `variants` to be an onject");
            variants = Some(
                json_variants
                    .iter()
                    .map(|(k, v)| {
                        (
                            k.to_string(),
                            v.as_array()
                                .expect("Variant should be a list of values")
                                .iter()
                                .map(|x| {
                                    x.as_str()
                                        .expect("Variant values should be strings")
                                        .to_string()
                                })
                                .collect(),
                        )
                    })
                    .collect(),
            );
        }

        config.insert(key.to_string(), FlagConfig { rollout, variants });
    }

    Some(config)
}

pub struct URLRetriever {
    url: String,
}

impl URLRetriever {
    pub fn new(url: &str) -> URLRetriever {
        URLRetriever {
            url: url.to_string(),
        }
    }
}

impl FlagRetriever for URLRetriever {
    fn retrieve(&self) -> Option<HashMap<String, FlagConfig>> {
        match ureq::get(&self.url).call() {
            Ok(resp) => {
                if resp.status() != 200 {
                    return None;
                }

                let body = resp.into_string().unwrap();
                parse_json_config(&body)
            }
            Err(e) => {
                println!("Error: {}", e);
                None
            }
        }
    }
}

pub struct JSONStringRetriever {
    data: String,
}

impl JSONStringRetriever {
    pub fn new(data: String) -> JSONStringRetriever {
        JSONStringRetriever {
            data: data.to_string(),
        }
    }
}

impl FlagRetriever for JSONStringRetriever {
    fn retrieve(&self) -> Option<HashMap<String, FlagConfig>> {
        parse_json_config(&self.data)
    }
}

pub struct JSONEnvVarRetriever {
    env_var_name: String,
}

impl JSONEnvVarRetriever {
    pub fn new(env_var_name: String) -> JSONEnvVarRetriever {
        JSONEnvVarRetriever {
            env_var_name: env_var_name.to_string(),
        }
    }
}

impl FlagRetriever for JSONEnvVarRetriever {
    fn retrieve(&self) -> Option<HashMap<String, FlagConfig>> {
        let data = std::env::var(&self.env_var_name);
        if data.is_err() {
            print!("Error retrieving env var: {}", self.env_var_name);
            return None;
        }
        parse_json_config(&data.unwrap())
    }
}
