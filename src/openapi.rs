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

//! OpenAPI 3.x to Discovery REST Description Adapter
//!
//! Converts OpenAPI 3.x JSON or YAML specifications into the internal
//! `discovery::RestDescription` model. This allows the CLI to dynamically generate
//! commands and validate requests for self-hosted OpenAPI services using the exact
//! same command builder and execution engine as Google Discovery services.

use std::collections::HashMap;

use serde::Deserialize;

use crate::discovery::{
    JsonSchema, JsonSchemaProperty, MethodParameter, RestDescription, RestMethod,
    SchemaRef,
};

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct OpenApiSpec {
    pub openapi: Option<String>,
    pub swagger: Option<String>,
    pub info: Option<OpenApiInfo>,
    pub servers: Option<Vec<OpenApiServer>>,
    #[serde(default)]
    pub paths: HashMap<String, OpenApiPathItem>,
    pub components: Option<OpenApiComponents>,
}

#[derive(Debug, Deserialize)]
pub struct OpenApiInfo {
    pub title: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OpenApiServer {
    pub url: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct OpenApiPathItem {
    pub get: Option<OpenApiOperation>,
    pub post: Option<OpenApiOperation>,
    pub put: Option<OpenApiOperation>,
    pub patch: Option<OpenApiOperation>,
    pub delete: Option<OpenApiOperation>,
}

#[derive(Debug, Deserialize)]
pub struct OpenApiOperation {
    pub summary: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "operationId")]
    pub operation_id: Option<String>,
    #[serde(default)]
    pub parameters: Vec<OpenApiParameter>,
    #[serde(rename = "requestBody")]
    pub request_body: Option<OpenApiRequestBody>,
    pub responses: Option<HashMap<String, OpenApiResponse>>,
}

#[derive(Debug, Deserialize)]
pub struct OpenApiParameter {
    pub name: String,
    #[serde(rename = "in")]
    pub parameter_in: String, // "path", "query", "header"
    pub description: Option<String>,
    #[serde(default)]
    pub required: bool,
    pub schema: Option<OpenApiSchema>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct OpenApiRequestBody {
    pub description: Option<String>,
    pub content: Option<HashMap<String, OpenApiMediaType>>,
}

#[derive(Debug, Deserialize)]
pub struct OpenApiMediaType {
    pub schema: Option<OpenApiSchema>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct OpenApiResponse {
    pub description: Option<String>,
    pub content: Option<HashMap<String, OpenApiMediaType>>,
}

#[derive(Debug, Deserialize)]
pub struct OpenApiComponents {
    pub schemas: Option<HashMap<String, OpenApiSchema>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenApiSchema {
    #[serde(rename = "type")]
    pub schema_type: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "$ref")]
    pub schema_ref: Option<String>,
    pub format: Option<String>,
    pub properties: Option<HashMap<String, OpenApiSchema>>,
    pub items: Option<Box<OpenApiSchema>>,
    #[serde(default)]
    pub required: Vec<String>,
}

pub const DEFAULT_COWORK_OPENAPI_SPEC: &str = r##"{
  "openapi": "3.0.3",
  "info": {
    "title": "OpenUI Cowork API",
    "description": "API for OpenUI Cowork workspace services (files, health, documents, etc.)",
    "version": "1.0.0"
  },
  "servers": [
    {
      "url": "https://leon4gr45-openui-cowork.hf.space"
    }
  ],
  "components": {
    "securitySchemes": {
      "BearerAuth": {
        "type": "http",
        "scheme": "bearer"
      },
      "ApiKeyAuth": {
        "type": "apiKey",
        "in": "header",
        "name": "X-API-Key"
      }
    },
    "schemas": {
      "HealthResponse": {
        "type": "object",
        "properties": {
          "status": { "type": "string" },
          "timestamp": { "type": "string" }
        }
      },
      "FileListResponse": {
        "type": "object",
        "properties": {
          "backend": { "type": "string" },
          "files": {
            "type": "array",
            "items": {
              "$ref": "#/components/schemas/FileItem"
            }
          }
        }
      },
      "FileItem": {
        "type": "object",
        "properties": {
          "id": { "type": "string" },
          "name": { "type": "string" },
          "size": { "type": "integer" },
          "mimeType": { "type": "string" }
        }
      },
      "FileUploadResponse": {
        "type": "object",
        "properties": {
          "id": { "type": "string" },
          "name": { "type": "string" },
          "status": { "type": "string" }
        }
      },
      "Document": {
        "type": "object",
        "properties": {
          "id": { "type": "string" },
          "title": { "type": "string" },
          "content": { "type": "string" }
        }
      }
    }
  },
  "paths": {
    "/health": {
      "get": {
        "summary": "Check backend health status",
        "operationId": "healthcheck",
        "responses": {
          "200": {
            "description": "Health status",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/HealthResponse"
                }
              }
            }
          }
        }
      }
    },
    "/api/files": {
      "get": {
        "summary": "List files stored in memory/backend",
        "operationId": "files_list",
        "parameters": [
          {
            "name": "limit",
            "in": "query",
            "required": false,
            "schema": {
              "type": "integer"
            },
            "description": "Max files to return"
          }
        ],
        "responses": {
          "200": {
            "description": "List of files",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/FileListResponse"
                }
              }
            }
          }
        }
      },
      "post": {
        "summary": "Upload or create file",
        "operationId": "files_create",
        "requestBody": {
          "content": {
            "application/json": {
              "schema": {
                "$ref": "#/components/schemas/FileItem"
              }
            }
          }
        },
        "responses": {
          "200": {
            "description": "File upload response",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/FileUploadResponse"
                }
              }
            }
          }
        }
      }
    },
    "/api/documents": {
      "get": {
        "summary": "List documents in workspace",
        "operationId": "documents_list",
        "responses": {
          "200": {
            "description": "Document list",
            "content": {
              "application/json": {
                "schema": {
                  "type": "array",
                  "items": {
                    "$ref": "#/components/schemas/Document"
                  }
                }
              }
            }
          }
        }
      },
      "post": {
        "summary": "Create new document",
        "operationId": "documents_create",
        "requestBody": {
          "content": {
            "application/json": {
              "schema": {
                "$ref": "#/components/schemas/Document"
              }
            }
          }
        },
        "responses": {
          "200": {
            "description": "Created document",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/Document"
                }
              }
            }
          }
        }
      }
    },
    "/api/documents/{id}": {
      "get": {
        "summary": "Get document by ID",
        "operationId": "documents_get",
        "parameters": [
          {
            "name": "id",
            "in": "path",
            "required": true,
            "schema": {
              "type": "string"
            },
            "description": "Document ID"
          }
        ],
        "responses": {
          "200": {
            "description": "Document details",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/Document"
                }
              }
            }
          }
        }
      },
      "delete": {
        "summary": "Delete document by ID",
        "operationId": "documents_delete",
        "parameters": [
          {
            "name": "id",
            "in": "path",
            "required": true,
            "schema": {
              "type": "string"
            },
            "description": "Document ID"
          }
        ],
        "responses": {
          "200": {
            "description": "Deletion status",
            "content": {
              "application/json": {
                "schema": {
                  "type": "object"
                }
              }
            }
          }
        }
      }
    }
  }
}"##;

/// Converts an OpenAPI spec string (JSON or YAML) into a `RestDescription`.
pub fn convert_openapi_to_rest_description(
    service_name: &str,
    spec_content: &str,
    base_url_override: Option<&str>,
) -> anyhow::Result<RestDescription> {
    let spec: OpenApiSpec = if let Ok(json_spec) = serde_json::from_str(spec_content) {
        json_spec
    } else {
        serde_yaml::from_str(spec_content)?
    };

    let title = spec.info.as_ref().and_then(|i| i.title.clone());
    let description = spec.info.as_ref().and_then(|i| i.description.clone());
    let version = spec
        .info
        .as_ref()
        .and_then(|i| i.version.clone())
        .unwrap_or_else(|| "v1".to_string());

    let root_url = if let Some(override_url) = base_url_override {
        override_url.trim_end_matches('/').to_string() + "/"
    } else if let Some(servers) = &spec.servers {
        if let Some(first) = servers.first() {
            first.url.trim_end_matches('/').to_string() + "/"
        } else {
            "http://localhost/".to_string()
        }
    } else {
        "http://localhost/".to_string()
    };

    let mut doc = RestDescription {
        name: service_name.to_string(),
        version,
        title,
        description,
        root_url,
        service_path: String::new(),
        base_url: None,
        schemas: HashMap::new(),
        resources: HashMap::new(),
        parameters: HashMap::new(),
        auth: None,
    };

    // 1. Convert components.schemas into doc.schemas
    if let Some(components) = spec.components {
        if let Some(schemas) = components.schemas {
            for (schema_name, oapi_schema) in schemas {
                doc.schemas
                    .insert(schema_name.clone(), convert_schema(&schema_name, &oapi_schema));
            }
        }
    }

    // 2. Parse paths and organize into resources and methods
    for (path, item) in spec.paths {
        let operations = [
            ("GET", &item.get),
            ("POST", &item.post),
            ("PUT", &item.put),
            ("PATCH", &item.patch),
            ("DELETE", &item.delete),
        ];

        for (http_method, op_opt) in operations {
            if let Some(op) = op_opt {
                add_operation_to_doc(&mut doc, service_name, &path, http_method, op);
            }
        }
    }

    Ok(doc)
}

fn convert_schema(name: &str, oapi: &OpenApiSchema) -> JsonSchema {
    let mut properties = HashMap::new();
    if let Some(props) = &oapi.properties {
        for (prop_name, prop_schema) in props {
            properties.insert(prop_name.clone(), convert_schema_property(prop_schema));
        }
    }

    JsonSchema {
        id: Some(name.to_string()),
        schema_type: oapi.schema_type.clone(),
        description: oapi.description.clone(),
        properties,
        schema_ref: oapi
            .schema_ref
            .as_deref()
            .map(clean_schema_ref),
        items: oapi
            .items
            .as_ref()
            .map(|it| Box::new(convert_schema_property(it))),
        required: oapi.required.clone(),
        additional_properties: None,
    }
}

fn convert_schema_property(oapi: &OpenApiSchema) -> JsonSchemaProperty {
    let mut properties = HashMap::new();
    if let Some(props) = &oapi.properties {
        for (prop_name, prop_schema) in props {
            properties.insert(prop_name.clone(), convert_schema_property(prop_schema));
        }
    }

    JsonSchemaProperty {
        prop_type: oapi.schema_type.clone(),
        description: oapi.description.clone(),
        schema_ref: oapi
            .schema_ref
            .as_deref()
            .map(clean_schema_ref),
        format: oapi.format.clone(),
        items: oapi
            .items
            .as_ref()
            .map(|it| Box::new(convert_schema_property(it))),
        properties,
        read_only: false,
        default: None,
        enum_values: None,
        additional_properties: None,
    }
}

fn clean_schema_ref(r: &str) -> String {
    if let Some(stripped) = r.strip_prefix("#/components/schemas/") {
        stripped.to_string()
    } else {
        r.to_string()
    }
}

fn add_operation_to_doc(
    doc: &mut RestDescription,
    service_name: &str,
    path: &str,
    http_method: &str,
    op: &OpenApiOperation,
) {
    // Clean path for discovery format: remove leading slash
    let clean_path = path.trim_start_matches('/');

    // Deduce resource name and method name
    let path_segments: Vec<&str> = clean_path
        .split('/')
        .filter(|s| !s.is_empty() && !s.starts_with('{'))
        .collect();

    let (resource_name, method_name) = if let Some(op_id) = &op.operation_id {
        if op_id.contains('_') {
            let parts: Vec<&str> = op_id.split('_').collect();
            let res = parts[..parts.len() - 1].join("_");
            let m = parts.last().unwrap().to_string();
            (res, m)
        } else {
            let res = path_segments
                .iter()
                .find(|&&s| s != "api" && s != "v1" && s != "v2")
                .copied()
                .unwrap_or_else(|| path_segments.first().copied().unwrap_or("default"))
                .to_string();
            (res, op_id.clone())
        }
    } else {
        let res = path_segments
            .iter()
            .find(|&&s| s != "api" && s != "v1" && s != "v2")
            .copied()
            .unwrap_or_else(|| path_segments.first().copied().unwrap_or("default"))
            .to_string();
        let m = match http_method {
            "GET" => {
                if path.ends_with('}') {
                    "get".to_string()
                } else {
                    "list".to_string()
                }
            }
            "POST" => "create".to_string(),
            "PUT" | "PATCH" => "update".to_string(),
            "DELETE" => "delete".to_string(),
            _ => http_method.to_lowercase(),
        };
        (res, m)
    };

    let mut parameters = HashMap::new();
    let mut parameter_order = Vec::new();

    for param in &op.parameters {
        let location = match param.parameter_in.as_str() {
            "path" => "path",
            "query" => "query",
            _ => continue,
        };

        parameter_order.push(param.name.clone());
        parameters.insert(
            param.name.clone(),
            MethodParameter {
                param_type: param
                    .schema
                    .as_ref()
                    .and_then(|s| s.schema_type.clone())
                    .or_else(|| Some("string".to_string())),
                description: param.description.clone(),
                location: Some(location.to_string()),
                required: param.required || location == "path",
                format: param.schema.as_ref().and_then(|s| s.format.clone()),
                default: None,
                enum_values: None,
                enum_descriptions: None,
                repeated: false,
                minimum: None,
                maximum: None,
                deprecated: false,
            },
        );
    }

    let request = op.request_body.as_ref().and_then(|rb| {
        rb.content.as_ref().and_then(|c| {
            c.get("application/json")
                .and_then(|mt| mt.schema.as_ref())
                .and_then(|s| {
                    s.schema_ref.as_deref().map(|r| SchemaRef {
                        schema_ref: Some(clean_schema_ref(r)),
                        parameter_name: None,
                    })
                })
        })
    });

    let response = op.responses.as_ref().and_then(|resps| {
        resps
            .get("200")
            .or_else(|| resps.get("201"))
            .and_then(|r| r.content.as_ref())
            .and_then(|c| {
                c.get("application/json")
                    .and_then(|mt| mt.schema.as_ref())
                    .and_then(|s| {
                        s.schema_ref.as_deref().map(|r| SchemaRef {
                            schema_ref: Some(clean_schema_ref(r)),
                            parameter_name: None,
                        })
                    })
            })
    });

    let method = RestMethod {
        id: Some(format!("{service_name}.{resource_name}.{method_name}")),
        description: op
            .summary
            .clone()
            .or_else(|| op.description.clone()),
        http_method: http_method.to_string(),
        path: clean_path.to_string(),
        parameters,
        parameter_order,
        request,
        response,
        scopes: Vec::new(),
        flat_path: Some(clean_path.to_string()),
        supports_media_download: false,
        supports_media_upload: false,
        media_upload: None,
    };

    let resource = doc
        .resources
        .entry(resource_name.to_string())
        .or_default();

    resource.methods.insert(method_name, method);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_openapi_fixture() {
        let doc = convert_openapi_to_rest_description(
            "cowork",
            DEFAULT_COWORK_OPENAPI_SPEC,
            Some("https://leon4gr45-openui-cowork.hf.space"),
        )
        .unwrap();

        assert_eq!(doc.name, "cowork");
        assert_eq!(doc.root_url, "https://leon4gr45-openui-cowork.hf.space/");

        // Verify no resource collision between files and documents!
        let files_res = doc.resources.get("files").expect("files resource missing");
        assert!(files_res.methods.contains_key("list"));
        assert!(files_res.methods.contains_key("create"));

        let docs_res = doc.resources.get("documents").expect("documents resource missing");
        assert!(docs_res.methods.contains_key("list"));
        assert!(docs_res.methods.contains_key("get"));
        assert!(docs_res.methods.contains_key("delete"));

        let health_res = doc.resources.get("health").expect("health resource missing");
        assert!(health_res.methods.contains_key("healthcheck"));
    }

    #[test]
    fn test_openapi_base_url_override() {
        let doc = convert_openapi_to_rest_description(
            "cowork",
            DEFAULT_COWORK_OPENAPI_SPEC,
            Some("https://custom-host.org/v1"),
        )
        .unwrap();
        assert_eq!(doc.root_url, "https://custom-host.org/v1/");
    }

    #[test]
    fn test_openapi_parameter_extraction() {
        let doc = convert_openapi_to_rest_description(
            "cowork",
            DEFAULT_COWORK_OPENAPI_SPEC,
            None,
        )
        .unwrap();

        let docs_res = doc.resources.get("documents").unwrap();
        let get_method = docs_res.methods.get("get").unwrap();
        assert!(get_method.parameters.contains_key("id"));
        assert_eq!(
            get_method.parameters["id"].location.as_deref(),
            Some("path")
        );
        assert!(get_method.parameters["id"].required);
    }
}
