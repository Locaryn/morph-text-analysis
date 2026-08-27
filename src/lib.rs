//! Locaryn Text Analysis Plugin
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentRequest {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentResult {
    pub sentiment: String,
    pub score: f32,
    pub breakdown: Vec<(String, f32)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityExtractRequest {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedEntity {
    pub text: String,
    pub label: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityExtractResult {
    pub entities: Vec<ExtractedEntity>,
}

/// Non implemente. La signature est conservee pour que l'interface et le
/// serveur MCP gardent leur forme, mais l'appel echoue franchement plutot
/// que de fabriquer un resultat.
pub async fn analyze_sentiment(_req: SentimentRequest) -> Result<SentimentResult, String> {
    Err(
        "L'analyse de sentiment n'est pas implementee : ce morph n'analyse pas le texte recu."
            .into(),
    )
}

/// Non implemente. La signature est conservee pour que l'interface et le
/// serveur MCP gardent leur forme, mais l'appel echoue franchement plutot
/// que de fabriquer un resultat.
pub async fn extract_entities(_req: EntityExtractRequest) -> Result<EntityExtractResult, String> {
    Err(
        "L'extraction d'entites n'est pas implementee : ce morph n'analyse pas le texte recu."
            .into(),
    )
}
