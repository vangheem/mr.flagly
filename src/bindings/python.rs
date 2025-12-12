use crate::service;
use crate::service::types;
use pyo3::prelude::*;
use std::collections::HashMap;

#[pyclass]
struct FlagService {
    flag_service: service::FlagService,
}

#[pymethods]
impl FlagService {
    #[new]
    #[pyo3(signature = (url=None, data=None, env_var=None, refresh_interval=300))]
    fn new(
        url: Option<String>,
        data: Option<String>,
        env_var: Option<String>,
        refresh_interval: u64,
    ) -> PyResult<FlagService> {
        let mut finder_type = types::FlagFinderType::NULL;
        if url.is_some() {
            finder_type = types::FlagFinderType::URL;
        } else if data.is_some() {
            finder_type = types::FlagFinderType::JSON;
        } else if env_var.is_some() {
            finder_type = types::FlagFinderType::ENVVAR;
        }

        Ok(FlagService {
            flag_service: service::FlagService::new(service::FlagServiceOptions {
                finder_type,
                url,
                refresh_interval,
                data,
                env_var,
            }),
        })
    }

    #[pyo3(signature = (name, default=false, context=None))]
    pub fn enabled(
        &self,
        name: &str,
        default: bool,
        context: Option<HashMap<String, String>>,
    ) -> PyResult<bool> {
        Ok(self.flag_service.enabled(name, default, context))
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn mrflagly(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<FlagService>()?;
    Ok(())
}
