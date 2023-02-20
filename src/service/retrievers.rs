use crate::service::types::FlagConfig;
use std::collections::HashMap;

pub trait FlagRetriever {
    fn retrieve(&self) -> Option<HashMap<String, FlagConfig>>;
}

fn parse_json_config(json: &str) -> Option<HashMap<String, FlagConfig>> {
    let json_parsed = json::parse(&json);
    if json_parsed.is_err() {
        print!("Error parsing JSON: {}", json);
        return None;
    }
    let json = json_parsed.unwrap();

    let mut config: HashMap<String, FlagConfig> = HashMap::new();
    for (key, value) in json.entries() {
        let rollout = value["rollout"].as_u8().unwrap();

        let mut variants: Option<HashMap<String, String>> = None;
        if value.has_key("variants") {
            variants = Some(
                value["variants"]
                    .entries()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
            );
        }

        config.insert(
            key.to_string(),
            FlagConfig {
                rollout,
                variants: variants,
            },
        );
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
                return None;
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
