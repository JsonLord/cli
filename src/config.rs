// Copyright 2026 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Configuration and Dynamic Service Registry
//!
//! Loads user and project configuration files to support self-hosted
//! workspace services alongside standard Google Workspace discovery services.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Auth scheme for a configured service.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum AuthType {
    #[default]
    None,
    Bearer,
    ApiKey,
    GoogleOauth,
}

/// Schema format used by a service.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum SchemaType {
    #[default]
    Discovery,
    Openapi,
}

/// Configuration for an individual service entry.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ServiceConfig {
    pub base_url: Option<String>,
    pub schema_url: Option<String>,
    pub schema_file: Option<String>,
    #[serde(default)]
    pub schema_type: SchemaType,
    #[serde(default)]
    pub auth: AuthType,
    pub bearer_token: Option<String>,
    pub api_key: Option<String>,
    pub header_name: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,
    #[serde(default)]
    pub is_google: bool,
}

impl ServiceConfig {
    /// Resolves an authentication secret (e.g. `$COWORK_TOKEN` or `env:COWORK_TOKEN` or raw string).
    pub fn resolve_secret(value: Option<&str>) -> Option<String> {
        let val = value?.trim();
        if val.is_empty() {
            return None;
        }

        if let Some(var_name) = val.strip_prefix('$') {
            std::env::var(var_name).ok()
        } else if let Some(var_name) = val.strip_prefix("env:") {
            std::env::var(var_name).ok()
        } else {
            Some(val.to_string())
        }
    }

    /// Returns resolved bearer token if present.
    pub fn resolved_bearer_token(&self) -> Option<String> {
        Self::resolve_secret(self.bearer_token.as_deref())
    }

    /// Returns resolved API key if present.
    pub fn resolved_api_key(&self) -> Option<String> {
        Self::resolve_secret(self.api_key.as_deref())
    }
}

/// Root CLI configuration file structure (`config.toml` or `.cws.toml`).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RegistryConfig {
    #[serde(default)]
    pub services: HashMap<String, ServiceConfig>,
}

impl RegistryConfig {
    /// Loads configuration merged from user config and project local config files.
    pub fn load() -> Self {
        let mut combined = Self::default();

        // 1. Load default built-in services (e.g. cowork)
        combined.insert_default_services();

        // 2. Load user configuration (~/.config/gws/config.toml or CWS equivalents)
        if let Some(user_config) = Self::load_user_config() {
            combined.merge(user_config);
        }

        // 3. Load project local configuration (.cws.toml or cws.toml in current/parent dirs)
        if let Some(project_config) = Self::load_project_config() {
            combined.merge(project_config);
        }

        combined
    }

    /// Merges another config into this config (other takes precedence).
    pub fn merge(&mut self, other: RegistryConfig) {
        for (name, service) in other.services {
            self.services.insert(name, service);
        }
    }

    fn insert_default_services(&mut self) {
        // Register default 'cowork' service pointing to OpenUI Cowork
        self.services.insert(
            "cowork".to_string(),
            ServiceConfig {
                base_url: Some("https://leon4gr45-openui-cowork.hf.space".to_string()),
                schema_url: Some("https://leon4gr45-openui-cowork.hf.space/openapi.json".to_string()),
                schema_file: None,
                schema_type: SchemaType::Openapi,
                auth: AuthType::Bearer,
                bearer_token: Some("$COWORK_TOKEN".to_string()),
                api_key: None,
                header_name: None,
                description: Some("OpenUI Cowork self-hosted workspace service".to_string()),
                version: Some("v1".to_string()),
                is_google: false,
            },
        );
    }

    fn load_user_config() -> Option<Self> {
        let config_dir = crate::auth_commands::config_dir();
        let candidate_paths = vec![
            config_dir.join("config.toml"),
            config_dir.join("cws.toml"),
        ];

        for path in candidate_paths {
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(config) = toml::from_str::<RegistryConfig>(&content) {
                        return Some(config);
                    }
                }
            }
        }
        None
    }

    fn load_project_config() -> Option<Self> {
        let current_dir = std::env::current_dir().ok()?;
        let mut dir = current_dir.as_path();

        loop {
            let candidates = [dir.join(".cws.toml"), dir.join("cws.toml")];
            for path in candidates {
                if path.exists() {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(config) = toml::from_str::<RegistryConfig>(&content) {
                            return Some(config);
                        }
                    }
                }
            }
            match dir.parent() {
                Some(parent) => dir = parent,
                None => break,
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_resolution() {
        std::env::set_var("TEST_SECRET_VAR", "secret_val_123");

        assert_eq!(
            ServiceConfig::resolve_secret(Some("$TEST_SECRET_VAR")),
            Some("secret_val_123".to_string())
        );
        assert_eq!(
            ServiceConfig::resolve_secret(Some("env:TEST_SECRET_VAR")),
            Some("secret_val_123".to_string())
        );
        assert_eq!(
            ServiceConfig::resolve_secret(Some("raw_token_xyz")),
            Some("raw_token_xyz".to_string())
        );
        assert_eq!(
            ServiceConfig::resolve_secret(Some("$NON_EXISTENT_VAR")),
            None
        );
    }

    #[test]
    fn test_toml_parsing() {
        let toml_str = r#"
        [services.cowork]
        base_url = "https://leon4gr45-openui-cowork.hf.space"
        schema_url = "https://leon4gr45-openui-cowork.hf.space/openapi.json"
        schema_type = "openapi"
        auth = "bearer"
        bearer_token = "$COWORK_TOKEN"
        "#;

        let config: RegistryConfig = toml::from_str(toml_str).unwrap();
        let cowork = config.services.get("cowork").unwrap();
        assert_eq!(
            cowork.base_url.as_deref(),
            Some("https://leon4gr45-openui-cowork.hf.space")
        );
        assert_eq!(cowork.schema_type, SchemaType::Openapi);
        assert_eq!(cowork.auth, AuthType::Bearer);
        assert_eq!(cowork.bearer_token.as_deref(), Some("$COWORK_TOKEN"));
    }

    #[test]
    fn test_invalid_toml_fails() {
        let invalid_toml = r#"
        [services.cowork
        base_url = invalid_url_without_quotes
        "#;
        assert!(toml::from_str::<RegistryConfig>(invalid_toml).is_err());
    }

    #[test]
    fn test_config_merging() {
        let mut base = RegistryConfig::default();
        base.insert_default_services();

        let override_toml = r#"
        [services.cowork]
        base_url = "https://custom-cowork.org"
        "#;
        let other: RegistryConfig = toml::from_str(override_toml).unwrap();
        base.merge(other);

        let cowork = base.services.get("cowork").unwrap();
        assert_eq!(cowork.base_url.as_deref(), Some("https://custom-cowork.org"));
    }
}
