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
  "openapi": "3.1.0",
  "info": {
    "title": "OpenUI Cowork API",
    "version": "v1",
    "description": "Machine-readable API over the OpenUI Cowork deployment: which document/slide apps are mounted, and service liveness. Each app also exposes its own native REST surface directly through this proxy (see each workspace's \"mount\") for capabilities not yet unified here."
  },
  "servers": [
    {
      "url": "/"
    }
  ],
  "security": [
    {
      "bearerAuth": []
    }
  ],
  "paths": {
    "/api/v1/health": {
      "get": {
        "operationId": "health_check",
        "summary": "Liveness probe",
        "description": "Returns service status without touching any backend app. Unauthenticated.",
        "security": [],
        "responses": {
          "200": {
            "description": "Service is up",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/HealthStatus"
                }
              }
            }
          }
        }
      }
    },
    "/api/v1/info": {
      "get": {
        "operationId": "info_get",
        "summary": "Deployment metadata and capabilities",
        "description": "Describes enabled capabilities and mounted workspaces. Requires authentication.",
        "responses": {
          "200": {
            "description": "Deployment info",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/Info"
                }
              }
            }
          },
          "401": {
            "$ref": "#/components/responses/Unauthorized"
          }
        }
      }
    },
    "/api/v1/workspaces": {
      "get": {
        "operationId": "workspaces_list",
        "summary": "List workspaces",
        "description": "Each workspace is one of the apps mounted behind this proxy (Casual Docs, Casual Slides).",
        "responses": {
          "200": {
            "description": "Workspace list",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/WorkspaceList"
                }
              }
            }
          },
          "401": {
            "$ref": "#/components/responses/Unauthorized"
          }
        }
      }
    },
    "/api/v1/workspaces/{workspace_id}": {
      "get": {
        "operationId": "workspaces_get",
        "summary": "Get a workspace by ID",
        "parameters": [
          {
            "name": "workspace_id",
            "in": "path",
            "required": true,
            "schema": {
              "type": "string"
            },
            "description": "Workspace ID, e.g. \"docs\" or \"slides\"."
          }
        ],
        "responses": {
          "200": {
            "description": "Workspace",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/Workspace"
                }
              }
            }
          },
          "401": {
            "$ref": "#/components/responses/Unauthorized"
          },
          "404": {
            "$ref": "#/components/responses/NotFound"
          }
        }
      }
    },
    "/api/v1/workspaces/{workspace_id}/service": {
      "get": {
        "operationId": "workspaces_service",
        "summary": "Get integration metadata for a workspace's underlying app",
        "description": "Factual integration metadata for the app backing this workspace: where its own native API lives, whether it publishes its own OpenAPI schema, and which capabilities have been audited and exposed so far. `openapi` and `capabilities` are empty until that app has been audited in a later pass — this endpoint never reports a capability the app has not actually been confirmed to support.",
        "parameters": [
          {
            "name": "workspace_id",
            "in": "path",
            "required": true,
            "schema": {
              "type": "string"
            },
            "description": "Workspace ID, e.g. \"docs\" or \"slides\"."
          }
        ],
        "responses": {
          "200": {
            "description": "Service integration metadata",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/WorkspaceService"
                }
              }
            }
          },
          "401": {
            "$ref": "#/components/responses/Unauthorized"
          },
          "404": {
            "$ref": "#/components/responses/NotFound"
          }
        }
      }
    },
    "/api/v1/workspaces/{workspace_id}/documents": {
      "post": {
        "operationId": "documents_create",
        "summary": "Create a document in a workspace",
        "description": "Currently implemented for the \"docs\" workspace only, backed by Casual Docs' own room creation. A document with a password can only be read back with that same password (see documents_download) — see docs/hf-space-docs-api-audit.md for why writing content is not yet exposed here.",
        "parameters": [
          {
            "name": "workspace_id",
            "in": "path",
            "required": true,
            "schema": {
              "type": "string"
            },
            "description": "Workspace ID. Only \"docs\" supports this operation today."
          }
        ],
        "requestBody": {
          "required": false,
          "content": {
            "application/json": {
              "schema": {
                "$ref": "#/components/schemas/DocumentCreateRequest"
              }
            }
          }
        },
        "responses": {
          "200": {
            "description": "Document created",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/Document"
                }
              }
            }
          },
          "401": {
            "$ref": "#/components/responses/Unauthorized"
          },
          "404": {
            "$ref": "#/components/responses/NotFound"
          },
          "503": {
            "$ref": "#/components/responses/ApiErrorResponse"
          }
        }
      }
    },
    "/api/v1/workspaces/{workspace_id}/documents/{document_id}": {
      "get": {
        "operationId": "documents_get",
        "summary": "Get a document's metadata",
        "description": "Currently implemented for the \"docs\" workspace only.",
        "parameters": [
          {
            "name": "workspace_id",
            "in": "path",
            "required": true,
            "schema": {
              "type": "string"
            },
            "description": "Workspace ID. Only \"docs\" supports this operation today."
          },
          {
            "name": "document_id",
            "in": "path",
            "required": true,
            "schema": {
              "type": "string"
            },
            "description": "Document ID, as returned by documents_create."
          }
        ],
        "responses": {
          "200": {
            "description": "Document metadata",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/Document"
                }
              }
            }
          },
          "401": {
            "$ref": "#/components/responses/Unauthorized"
          },
          "404": {
            "$ref": "#/components/responses/NotFound"
          }
        }
      }
    },
    "/api/v1/workspaces/{workspace_id}/documents/{document_id}/content": {
      "get": {
        "operationId": "documents_download",
        "summary": "Download a document's original content",
        "description": "Returns the document's *original* uploaded content, not its live collaboratively-edited state — Casual Docs has no HTTP endpoint for a room's current content; live edits only exist as CRDT updates over its WebSocket. A document with edits since creation will not reflect them here. Currently implemented for the \"docs\" workspace only. See docs/hf-space-docs-api-audit.md.",
        "parameters": [
          {
            "name": "workspace_id",
            "in": "path",
            "required": true,
            "schema": {
              "type": "string"
            },
            "description": "Workspace ID. Only \"docs\" supports this operation today."
          },
          {
            "name": "document_id",
            "in": "path",
            "required": true,
            "schema": {
              "type": "string"
            },
            "description": "Document ID, as returned by documents_create."
          },
          {
            "name": "password",
            "in": "query",
            "required": false,
            "schema": {
              "type": "string"
            },
            "description": "Required when documents_get reports needs_password: true for this document."
          }
        ],
        "responses": {
          "200": {
            "description": "Document content bytes",
            "content": {
              "application/vnd.openxmlformats-officedocument.wordprocessingml.document": {
                "schema": {
                  "type": "string",
                  "format": "binary"
                }
              }
            }
          },
          "401": {
            "$ref": "#/components/responses/Unauthorized"
          },
          "404": {
            "$ref": "#/components/responses/NotFound"
          }
        }
      }
    }
  },
  "components": {
    "securitySchemes": {
      "bearerAuth": {
        "type": "http",
        "scheme": "bearer"
      }
    },
    "schemas": {
      "HealthStatus": {
        "type": "object",
        "required": [
          "status",
          "service",
          "api_version"
        ],
        "properties": {
          "status": {
            "type": "string",
            "example": "ok"
          },
          "service": {
            "type": "string",
            "example": "cowork"
          },
          "api_version": {
            "type": "string",
            "example": "v1"
          }
        }
      },
      "Info": {
        "type": "object",
        "properties": {
          "service": {
            "type": "string"
          },
          "api_version": {
            "type": "string"
          },
          "capabilities": {
            "type": "array",
            "items": {
              "type": "string"
            }
          },
          "workspaces": {
            "type": "array",
            "items": {
              "$ref": "#/components/schemas/Workspace"
            }
          }
        }
      },
      "Workspace": {
        "type": "object",
        "required": [
          "id",
          "title",
          "kind",
          "mount"
        ],
        "properties": {
          "id": {
            "type": "string"
          },
          "title": {
            "type": "string"
          },
          "kind": {
            "type": "string"
          },
          "mount": {
            "type": "string"
          },
          "description": {
            "type": "string"
          }
        }
      },
      "WorkspaceList": {
        "type": "object",
        "required": [
          "items"
        ],
        "properties": {
          "items": {
            "type": "array",
            "items": {
              "$ref": "#/components/schemas/Workspace"
            }
          },
          "nextPageToken": {
            "type": "string",
            "nullable": true
          }
        }
      },
      "WorkspaceService": {
        "type": "object",
        "required": [
          "id",
          "base_path",
          "capabilities"
        ],
        "properties": {
          "id": {
            "type": "string"
          },
          "base_path": {
            "type": "string",
            "description": "Where the app itself is mounted behind this proxy."
          },
          "api_base": {
            "type": "string",
            "nullable": true,
            "description": "Where the app's own native API lives, if known and audited."
          },
          "openapi": {
            "type": "string",
            "nullable": true,
            "description": "Path to this app's own OpenAPI schema, once published."
          },
          "capabilities": {
            "type": "array",
            "items": {
              "type": "string"
            },
            "description": "Audited, exposed capabilities of this app's native API, e.g. \"documents_get\"."
          }
        }
      },
      "ApiError": {
        "type": "object",
        "required": [
          "error"
        ],
        "properties": {
          "error": {
            "type": "object",
            "required": [
              "code",
              "message"
            ],
            "properties": {
              "code": {
                "type": "string"
              },
              "message": {
                "type": "string"
              },
              "details": {
                "type": "object"
              }
            }
          }
        }
      },
      "DocumentCreateRequest": {
        "type": "object",
        "properties": {
          "password": {
            "type": "string",
            "description": "Optional. If set, documents_download requires this same password."
          }
        }
      },
      "Document": {
        "type": "object",
        "required": [
          "id",
          "needs_password"
        ],
        "properties": {
          "id": {
            "type": "string"
          },
          "needs_password": {
            "type": "boolean"
          },
          "has_initial_content": {
            "type": "boolean",
            "description": "Whether content has ever been uploaded for this document."
          },
          "has_snapshot": {
            "type": "boolean"
          },
          "active_clients": {
            "type": "integer"
          }
        }
      }
    },
    "responses": {
      "Unauthorized": {
        "description": "Missing or invalid bearer token",
        "content": {
          "application/json": {
            "schema": {
              "$ref": "#/components/schemas/ApiError"
            }
          }
        }
      },
      "NotFound": {
        "description": "Resource not found",
        "content": {
          "application/json": {
            "schema": {
              "$ref": "#/components/schemas/ApiError"
            }
          }
        }
      },
      "ApiErrorResponse": {
        "description": "Structured error",
        "content": {
          "application/json": {
            "schema": {
              "$ref": "#/components/schemas/ApiError"
            }
          }
        }
      }
    }
  }
}
"##;

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
        // DEFAULT_COWORK_OPENAPI_SPEC is a verified snapshot of the real,
        // deployed Cowork /openapi.json (audited against a live compiled
        // `gws`/`cws` binary run against a local Cowork test instance —
        // see JsonLord/genoffice's cws-compat.test.mjs for the JS-side half
        // of this compatibility contract). Every operationId is
        // `resource_method` with a single-word method, since this
        // function's own split-on-last-underscore logic below silently
        // absorbs a multi-word method into the resource name otherwise.
        let doc = convert_openapi_to_rest_description(
            "cowork",
            DEFAULT_COWORK_OPENAPI_SPEC,
            Some("https://leon4gr45-openui-cowork.hf.space"),
        )
        .unwrap();

        assert_eq!(doc.name, "cowork");
        assert_eq!(doc.root_url, "https://leon4gr45-openui-cowork.hf.space/");

        let workspaces_res = doc
            .resources
            .get("workspaces")
            .expect("workspaces resource missing");
        assert!(workspaces_res.methods.contains_key("list"));
        assert!(workspaces_res.methods.contains_key("get"));
        assert!(workspaces_res.methods.contains_key("service"));
        // Paths are absolute from root (/api/v1/...), NOT relative to
        // `servers[].url` — this adapter ignores servers[].url whenever a
        // base_url override is supplied (true for every configured
        // self-hosted service), so a servers-relative path here would
        // silently resolve to the wrong URL. Confirmed against a live 404
        // before this fixture was corrected.
        assert_eq!(workspaces_res.methods["list"].path, "api/v1/workspaces");

        let docs_res = doc
            .resources
            .get("documents")
            .expect("documents resource missing");
        assert!(docs_res.methods.contains_key("create"));
        assert!(docs_res.methods.contains_key("get"));
        assert!(docs_res.methods.contains_key("download"));
        // No write/delete capability — see genoffice's
        // docs/hf-space-docs-api-audit.md: Docs' own write routes skip the
        // room-password check their read routes enforce, so Cowork
        // deliberately does not expose one.
        assert!(!docs_res.methods.contains_key("update"));
        assert!(!docs_res.methods.contains_key("delete"));

        let health_res = doc.resources.get("health").expect("health resource missing");
        assert!(health_res.methods.contains_key("check"));

        let info_res = doc.resources.get("info").expect("info resource missing");
        assert!(info_res.methods.contains_key("get"));
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
        assert!(get_method.parameters.contains_key("workspace_id"));
        assert!(get_method.parameters.contains_key("document_id"));
        assert_eq!(
            get_method.parameters["document_id"].location.as_deref(),
            Some("path")
        );
        assert!(get_method.parameters["document_id"].required);
    }
}
