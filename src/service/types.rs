use std::collections::HashMap;

pub struct FlagConfig {
    pub rollout: u8, // what percentage of the traffic should see the feature
    pub variants: Option<HashMap<String, String>>, // flag enabled only for specific variants
}

#[derive(Clone)]
pub enum FlagRetrieverType {
    URL,
    JSON,
    NULL,
}

#[derive(Clone)]
pub struct FlagServiceOptions {
    pub retriever_type: Option<FlagRetrieverType>,
    pub retriever_url: Option<String>,
    pub retriever_data: Option<String>,
    pub refresh_interval: u64,
}
