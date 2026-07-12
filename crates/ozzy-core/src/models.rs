use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Profile {
    pub id: String,
    pub github_id: i64,
    pub login: String,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Project {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub owner_profile_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum MemberRole {
    Read,
    Write,
    Admin,
}

impl MemberRole {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "read" => Some(Self::Read),
            "write" => Some(Self::Write),
            "admin" => Some(Self::Admin),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Write => "write",
            Self::Admin => "admin",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ApiKeyPermission {
    Read,
    Write,
}

impl ApiKeyPermission {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "read" => Some(Self::Read),
            "write" => Some(Self::Write),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Write => "write",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct SecretMeta {
    pub key_name: String,
    pub version: i64,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct SecretValue {
    pub key_name: String,
    pub value: String,
    pub version: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ApiKeyScope {
    pub project_id: String,
    pub permission: ApiKeyPermission,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ApiKeyInfo {
    pub id: String,
    pub name: String,
    pub key_prefix: String,
    pub scopes: Vec<ApiKeyScope>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CreateApiKeyResponse {
    pub id: String,
    pub name: String,
    pub key: String,
    pub key_prefix: String,
    pub scopes: Vec<ApiKeyScope>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_member_role() {
        assert_eq!(MemberRole::parse("read"), Some(MemberRole::Read));
        assert_eq!(MemberRole::parse("write"), Some(MemberRole::Write));
        assert_eq!(MemberRole::parse("admin"), Some(MemberRole::Admin));
        assert_eq!(MemberRole::parse("invalid"), None);

        assert_eq!(MemberRole::Read.as_str(), "read");
        assert_eq!(MemberRole::Write.as_str(), "write");
        assert_eq!(MemberRole::Admin.as_str(), "admin");
    }

    #[test]
    fn test_api_key_permission() {
        assert_eq!(ApiKeyPermission::parse("read"), Some(ApiKeyPermission::Read));
        assert_eq!(ApiKeyPermission::parse("write"), Some(ApiKeyPermission::Write));
        assert_eq!(ApiKeyPermission::parse("invalid"), None);

        assert_eq!(ApiKeyPermission::Read.as_str(), "read");
        assert_eq!(ApiKeyPermission::Write.as_str(), "write");
    }

    #[test]
    fn test_structs_serialization() {
        // Profile
        let profile = Profile {
            id: "123".to_string(),
            github_id: 456,
            login: "willy".to_string(),
            name: Some("William".to_string()),
            avatar_url: Some("https://avatar".to_string()),
        };
        let serialized = serde_json::to_string(&profile).unwrap();
        let deserialized: Profile = serde_json::from_str(&serialized).unwrap();
        assert_eq!(profile, deserialized);
        assert!(format!("{:?}", profile).contains("willy"));

        // Project
        let project = Project {
            id: "proj".to_string(),
            slug: "slug".to_string(),
            name: "name".to_string(),
            owner_profile_id: "owner".to_string(),
        };
        let serialized = serde_json::to_string(&project).unwrap();
        let deserialized: Project = serde_json::from_str(&serialized).unwrap();
        assert_eq!(project, deserialized);

        // SecretMeta
        let secret_meta = SecretMeta {
            key_name: "SECRET_KEY".to_string(),
            version: 1,
            updated_at: "2026-07-12".to_string(),
        };
        let serialized = serde_json::to_string(&secret_meta).unwrap();
        let deserialized: SecretMeta = serde_json::from_str(&serialized).unwrap();
        assert_eq!(secret_meta, deserialized);

        // SecretValue
        let secret_value = SecretValue {
            key_name: "SECRET_KEY".to_string(),
            value: "secret_value".to_string(),
            version: 1,
        };
        let serialized = serde_json::to_string(&secret_value).unwrap();
        let deserialized: SecretValue = serde_json::from_str(&serialized).unwrap();
        assert_eq!(secret_value, deserialized);

        // ApiKeyScope
        let api_key_scope = ApiKeyScope {
            project_id: "proj".to_string(),
            permission: ApiKeyPermission::Read,
        };
        let serialized = serde_json::to_string(&api_key_scope).unwrap();
        let deserialized: ApiKeyScope = serde_json::from_str(&serialized).unwrap();
        assert_eq!(api_key_scope, deserialized);

        // ApiKeyInfo
        let api_key_info = ApiKeyInfo {
            id: "key_id".to_string(),
            name: "key_name".to_string(),
            key_prefix: "prefix".to_string(),
            scopes: vec![api_key_scope.clone()],
        };
        let serialized = serde_json::to_string(&api_key_info).unwrap();
        let deserialized: ApiKeyInfo = serde_json::from_str(&serialized).unwrap();
        assert_eq!(api_key_info, deserialized);

        // CreateApiKeyResponse
        let create_api_key_resp = CreateApiKeyResponse {
            id: "key_id".to_string(),
            name: "key_name".to_string(),
            key: "key_secret".to_string(),
            key_prefix: "prefix".to_string(),
            scopes: vec![api_key_scope],
        };
        let serialized = serde_json::to_string(&create_api_key_resp).unwrap();
        let deserialized: CreateApiKeyResponse = serde_json::from_str(&serialized).unwrap();
        assert_eq!(create_api_key_resp, deserialized);
    }
}
