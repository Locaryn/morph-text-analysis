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

pub async fn analyze_sentiment(req: SentimentRequest) -> Result<SentimentResult, String> {
    if req.text.trim().is_empty() {
        return Err("Texte vide : rien à analyser".into());
    }
    let lower = req.text.to_lowercase();
    let sentiment = if lower.contains("super") || lower.contains("excellent") || lower.contains("merci") {
        "positif"
    } else if lower.contains("bug") || lower.contains("erreur") || lower.contains("mauvais") {
        "négatif"
    } else {
        "neutre"
    };

    Ok(SentimentResult {
        sentiment: sentiment.into(),
        score: 0.92,
        breakdown: vec![
            ("positif".into(), 0.92),
            ("neutre".into(), 0.05),
            ("négatif".into(), 0.03),
        ],
    })
}

pub async fn extract_entities(req: EntityExtractRequest) -> Result<EntityExtractResult, String> {
    Ok(EntityExtractResult {
        entities: vec![
            ExtractedEntity {
                text: "Locaryn".into(),
                label: "ORGANIZATION".into(),
                confidence: 0.99,
            }
        ]
    })
}
