use std::collections::HashMap;

pub struct FlagConfig {
    pub rollout: u8, // what percentage of the traffic should see the feature
    pub variants: Option<HashMap<String, String>>, // flag enabled only for specific variants
}

#[derive(Clone)]
pub enum FlagFinderType {
    URL,
    JSON,
    ENVVAR,
    NULL,
}

#[derive(Clone)]
pub struct FlagServiceOptions {
    pub finder_type: FlagFinderType,
    pub url: Option<String>,
    pub data: Option<String>,
    pub env_var: Option<String>,
    pub refresh_interval: u64,
}
