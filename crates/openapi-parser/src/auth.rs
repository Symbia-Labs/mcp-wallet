//! Authentication scheme detection and handling

use crate::types::*;
use std::collections::HashMap;

/// Detected authentication scheme for an API
#[derive(Debug, Clone)]
pub enum AuthScheme {
    /// No authentication required
    None,
    /// Bearer token (Authorization: Bearer <token>)
    Bearer { format: Option<String> },
    /// API key in header, query, or cookie
    ApiKey {
        name: String,
        location: ApiKeyLocation,
    },
    /// Basic authentication
    Basic,
    /// OAuth2 authentication
    OAuth2 {
        authorization_url: Option<String>,
        token_url: Option<String>,
        scopes: Vec<String>,
    },
    /// Multiple auth schemes (any of)
    Multiple(Vec<AuthScheme>),
}

impl AuthScheme {
    /// Detect the primary authentication scheme from security schemes
    pub fn detect(
        security_schemes: &HashMap<String, SecurityScheme>,
        security_requirements: &[SecurityRequirement],
    ) -> Self {
        if security_requirements.is_empty() {
            // Check if there are any security schemes defined
            if security_schemes.is_empty() {
                return AuthScheme::None;
            }
            // Use the first defined scheme as default
            if let Some((_, scheme)) = security_schemes.iter().next() {
                return Self::from_scheme(scheme);
            }
            return AuthScheme::None;
        }

        // Collect all required auth schemes
        let mut schemes: Vec<AuthScheme> = Vec::new();

        for req in security_requirements {
            if let Some(scheme) = security_schemes.get(&req.scheme_name) {
                let mut auth = Self::from_scheme(scheme);

                // Add scopes for OAuth2
                if let AuthScheme::OAuth2 { ref mut scopes, .. } = auth {
                    *scopes = req.scopes.clone();
                }

                schemes.push(auth);
            }
        }

        match schemes.len() {
            0 => AuthScheme::None,
            1 => schemes.remove(0),
            _ => AuthScheme::Multiple(schemes),
        }
    }

    /// Convert a security scheme to an auth scheme
    fn from_scheme(scheme: &SecurityScheme) -> Self {
        match scheme {
            SecurityScheme::ApiKey { name, location } => AuthScheme::ApiKey {
                name: name.clone(),
                location: *location,
            },
            SecurityScheme::Http {
                scheme,
                bearer_format,
            } => match scheme.to_lowercase().as_str() {
                "bearer" => AuthScheme::Bearer {
                    format: bearer_format.clone(),
                },
                "basic" => AuthScheme::Basic,
                _ => AuthScheme::Bearer { format: None },
            },
            SecurityScheme::OAuth2 { flows } => {
                // Prefer authorization_code flow
                let (auth_url, token_url) = if let Some(flow) = &flows.authorization_code {
                    (flow.authorization_url.clone(), flow.token_url.clone())
                } else if let Some(flow) = &flows.client_credentials {
                    (None, flow.token_url.clone())
                } else if let Some(flow) = &flows.implicit {
                    (flow.authorization_url.clone(), None)
                } else if let Some(flow) = &flows.password {
                    (None, flow.token_url.clone())
                } else {
                    (None, None)
                };

                AuthScheme::OAuth2 {
                    authorization_url: auth_url,
                    token_url,
                    scopes: Vec::new(),
                }
            }
            SecurityScheme::OpenIdConnect { openid_connect_url } => {
                // Treat OpenID Connect as OAuth2-like
                AuthScheme::OAuth2 {
                    authorization_url: Some(openid_connect_url.clone()),
                    token_url: None,
                    scopes: Vec::new(),
                }
            }
        }
    }

    /// Get the header name for this auth scheme
    pub fn header_name(&self) -> Option<&str> {
        match self {
            AuthScheme::Bearer { .. } | AuthScheme::Basic => Some("Authorization"),
            AuthScheme::ApiKey { name, location } => {
                if *location == ApiKeyLocation::Header {
                    Some(name)
                } else {
                    None
                }
            }
            AuthScheme::OAuth2 { .. } => Some("Authorization"),
            _ => None,
        }
    }

    /// Format the authorization header value
    pub fn format_header(&self, credential: &str) -> Option<String> {
        match self {
            AuthScheme::Bearer { .. } => Some(format!("Bearer {}", credential)),
            AuthScheme::Basic => Some(format!("Basic {}", credential)),
            AuthScheme::ApiKey { location, .. } => {
                if *location == ApiKeyLocation::Header {
                    Some(credential.to_string())
                } else {
                    None
                }
            }
            AuthScheme::OAuth2 { .. } => Some(format!("Bearer {}", credential)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_bearer() {
        let mut schemes = HashMap::new();
        schemes.insert(
            "bearerAuth".to_string(),
            SecurityScheme::Http {
                scheme: "bearer".to_string(),
                bearer_format: Some("JWT".to_string()),
            },
        );

        let requirements = vec![SecurityRequirement {
            scheme_name: "bearerAuth".to_string(),
            scopes: vec![],
        }];

        let auth = AuthScheme::detect(&schemes, &requirements);

        match auth {
            AuthScheme::Bearer { format } => {
                assert_eq!(format, Some("JWT".to_string()));
            }
            _ => panic!("Expected Bearer auth"),
        }
    }

    #[test]
    fn test_detect_api_key() {
        let mut schemes = HashMap::new();
        schemes.insert(
            "apiKey".to_string(),
            SecurityScheme::ApiKey {
                name: "X-API-Key".to_string(),
                location: ApiKeyLocation::Header,
            },
        );

        let requirements = vec![SecurityRequirement {
            scheme_name: "apiKey".to_string(),
            scopes: vec![],
        }];

        let auth = AuthScheme::detect(&schemes, &requirements);

        match auth {
            AuthScheme::ApiKey { name, location } => {
                assert_eq!(name, "X-API-Key");
                assert_eq!(location, ApiKeyLocation::Header);
            }
            _ => panic!("Expected ApiKey auth"),
        }
    }

    #[test]
    fn test_format_bearer_header() {
        let auth = AuthScheme::Bearer { format: None };
        let header = auth.format_header("my-token");
        assert_eq!(header, Some("Bearer my-token".to_string()));
    }
}
