use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use tracing::info;

use crate::db::entities::{api_tokens, route_rules, upstreams};
use crate::service::token;

/// Initialize mock data for development environment
pub async fn init_mock_data(db: &DatabaseConnection) -> anyhow::Result<()> {
    info!("Initializing mock data for development environment...");

    // Create mock upstreams
    create_mock_upstreams(db).await?;

    // Create mock API tokens
    create_mock_tokens(db).await?;

    // Create mock route rules
    create_mock_routes(db).await?;

    info!("Mock data initialization completed");
    Ok(())
}

async fn create_mock_upstreams(db: &DatabaseConnection) -> anyhow::Result<()> {
    let mock_upstreams = vec![
        upstreams::ActiveModel {
            name: Set("OpenAI API".to_string()),
            base_url: Set("https://api.openai.com".to_string()),
            api_key: Set(Some("sk-mock-openai-key".to_string())),
            extra_headers: Set(None),
            timeout_secs: Set(30),
            weight: Set(10),
            priority: Set(1),
            lb_strategy: Set("round_robin".to_string()),
            enabled: Set(true),
            is_healthy: Set(true),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
            ..Default::default()
        },
        upstreams::ActiveModel {
            name: Set("Anthropic API".to_string()),
            base_url: Set("https://api.anthropic.com".to_string()),
            api_key: Set(Some("sk-ant-mock-key".to_string())),
            extra_headers: Set(None),
            timeout_secs: Set(60),
            weight: Set(8),
            priority: Set(2),
            lb_strategy: Set("weighted".to_string()),
            enabled: Set(true),
            is_healthy: Set(true),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
            ..Default::default()
        },
        upstreams::ActiveModel {
            name: Set("Local LLM".to_string()),
            base_url: Set("http://localhost:11434".to_string()),
            api_key: Set(None),
            extra_headers: Set(None),
            timeout_secs: Set(120),
            weight: Set(5),
            priority: Set(3),
            lb_strategy: Set("failover".to_string()),
            enabled: Set(true),
            is_healthy: Set(false),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
            ..Default::default()
        },
    ];

    for upstream in mock_upstreams {
        upstream.insert(db).await?;
    }

    info!("Created {} mock upstreams", 3);
    Ok(())
}

async fn create_mock_tokens(db: &DatabaseConnection) -> anyhow::Result<()> {
    let mock_tokens = vec![
        ("Production Bot", None, Some(1000), Some(100000)),
        ("Development Test", Some(chrono::Utc::now() + chrono::Duration::days(30)), Some(100), Some(10000)),
        ("Demo Account", None, Some(50), Some(5000)),
    ];

    for (label, expires_at, rpm_limit, tpm_limit) in mock_tokens {
        let raw_token = token::generate_token();
        let token_hash = token::hash_token(&raw_token);

        let model = api_tokens::ActiveModel {
            token_hash: Set(token_hash),
            label: Set(label.to_string()),
            expires_at: Set(expires_at),
            enabled: Set(true),
            rpm_limit: Set(rpm_limit),
            tpm_limit: Set(tpm_limit),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
            ..Default::default()
        };

        model.insert(db).await?;
        info!("Created mock token '{}': {}", label, raw_token);
    }

    Ok(())
}

async fn create_mock_routes(db: &DatabaseConnection) -> anyhow::Result<()> {
    let mock_routes = vec![
        route_rules::ActiveModel {
            name: Set("OpenAI Chat Completion".to_string()),
            inbound_path: Set("/v1/chat/completions".to_string()),
            outbound_path: Set("/v1/chat/completions".to_string()),
            match_type: Set("exact".to_string()),
            upstream_id: Set(Some(1)),
            upstream_ids: Set(None),
            priority: Set(10),
            extra_headers: Set(None),
            extra_query: Set(None),
            enabled: Set(true),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
            ..Default::default()
        },
        route_rules::ActiveModel {
            name: Set("Anthropic Messages".to_string()),
            inbound_path: Set("/v1/messages".to_string()),
            outbound_path: Set("/v1/messages".to_string()),
            match_type: Set("exact".to_string()),
            upstream_id: Set(Some(2)),
            upstream_ids: Set(None),
            priority: Set(10),
            extra_headers: Set(None),
            extra_query: Set(None),
            enabled: Set(true),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
            ..Default::default()
        },
        route_rules::ActiveModel {
            name: Set("Catch All API".to_string()),
            inbound_path: Set("/api/".to_string()),
            outbound_path: Set("/api/".to_string()),
            match_type: Set("prefix".to_string()),
            upstream_id: Set(None),
            upstream_ids: Set(None),
            priority: Set(1),
            extra_headers: Set(None),
            extra_query: Set(None),
            enabled: Set(true),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
            ..Default::default()
        },
    ];

    for route in mock_routes {
        route.insert(db).await?;
    }

    info!("Created {} mock route rules", 3);
    Ok(())
}
