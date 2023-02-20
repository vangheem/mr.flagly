# Introduction

Feature flagging should not be so complicated.

Mr Flagly is a decentralized feature flagging system written in Rust with bindings for other languages.

It does not depend on third party services or require you to deploy services and databases to manage.

Supported feature flag definition sources:

- URL
- JSON Value
- Environment Variable


## Rust usage

Setup your flag service:

```rust
use mrflagly::service::{FlagService, FlagServiceOptions};
let flag_service = FlagService::new(FlagServiceOptions {
    finder_type: mrflagly::service::types::FlagFinderType::URL,
    url: "https://path/to/hosted/json/file",
    refresh_interval: 600,
    data: None,
    env_var: None,
})
```

Then, to check for feature flag:

```rust
if flag_service.enabled("feature_x", false /* default value */, Some(HashMap::from([(String::from("user_id"), String::from("123")),])) /* optional context */) {
    // do something
}
```


## Python support

```python
import mrflagly
import json

flag_service = mrflagly.FlagService(data=json.dumps({"feature_x": {"rollout": 100}}))
if flag_service.enabled("feature_x", False, None):
    # do something
```


## JSON format

JSON format for feature flag data:

```
{
    "my_feature": {
        "rollout": 100
    },
    "my_feature_with_variants": {
        "rollout": 0,
        "variants" {
            "user_id": ["123"],
            "company_id": ["123"]
        }
    }
}
```