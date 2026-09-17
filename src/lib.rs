//! Analyse de sentiment, par le modèle qui tourne déjà sur la machine.
//!
//! Aucun modèle ModernBERT n'est embarqué. C'est le moteur d'inférence local
//! (compatible OpenAI, `/v1/chat/completions`) qui classe le texte, avec une
//! consigne qui n'accepte que trois réponses possibles et rien d'autre — le
//! même principe que le morph de traduction, pour la même raison : un modèle
//! livré à lui-même commente, nuance, ajoute des guillemets.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Adresse du serveur compatible OpenAI.
    #[serde(default = "endpoint_par_defaut")]
    pub endpoint: String,
    /// Modèle à employer. Vide : celui que le serveur a déjà chargé.
    #[serde(default)]
    pub model: String,
}

fn endpoint_par_defaut() -> String {
    "http://127.0.0.1:8080".into()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            endpoint: endpoint_par_defaut(),
            model: String::new(),
        }
    }
}

pub fn config() -> Config {
    let Some(p) = std::env::var("LOCARYN_EXTENSION_CONFIG_FILE")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map(PathBuf::from)
    else {
        return Config::default();
    };
    std::fs::read_to_string(p)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

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

const LABELS: [&str; 3] = ["positif", "négatif", "neutre"];

/// Classer la tonalité d'un texte, par le modèle d'inférence local.
///
/// La consigne n'accepte qu'un mot parmi [`LABELS`] ; une réponse qui n'en
/// est pas un est une erreur, pas une devinette. Le score n'est pas une
/// probabilité calibrée par classe — aucun modèle de classification n'est
/// embarqué — mais la confiance réelle du modèle dans les tokens qu'il a
/// rendus, lue sur ses propres logprobs (`exp` de leur moyenne). Un modèle
/// hésitant entre deux tonalités rend un score bas ; ce n'est pas inventé.
pub async fn analyze_sentiment(req: SentimentRequest) -> Result<SentimentResult, String> {
    if req.text.trim().is_empty() {
        return Err("Texte vide : rien à analyser.".into());
    }

    let cfg = config();
    let consigne = format!(
        "Tu classes la tonalite du texte de l'utilisateur. Tu reponds par exactement un mot, \
         choisi parmi : {}. Rien d'autre : pas de ponctuation, pas d'explication, pas de \
         majuscule.",
        LABELS.join(", ")
    );

    let corps = serde_json::json!({
        "model": if cfg.model.trim().is_empty() { "local" } else { cfg.model.trim() },
        "temperature": 0.0,
        "max_tokens": 6,
        "logprobs": true,
        "top_logprobs": 1,
        "messages": [
            { "role": "system", "content": consigne },
            { "role": "user", "content": req.text }
        ]
    });

    let client = reqwest::Client::new();
    let resp = client
        .post(format!(
            "{}/v1/chat/completions",
            cfg.endpoint.trim_end_matches('/')
        ))
        .timeout(std::time::Duration::from_secs(120))
        .json(&corps)
        .send()
        .await
        .map_err(|_| {
            "Le moteur d'inférence ne répond pas. Démarrez-le, puis relancez l'analyse.".to_string()
        })?;

    if !resp.status().is_success() {
        let statut = resp.status();
        let detail = resp.text().await.unwrap_or_default();
        return Err(format!(
            "Le moteur a refusé la demande ({statut}){}",
            if detail.trim().is_empty() {
                String::new()
            } else {
                format!(" : {}", detail.chars().take(200).collect::<String>())
            }
        ));
    }

    let v: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("réponse illisible du moteur : {e}"))?;
    let choix = v
        .get("choices")
        .and_then(|c| c.get(0))
        .ok_or("Le moteur n'a rendu aucun choix.")?;

    let brut = choix
        .get("message")
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or("Le moteur n'a rien classé.")?;

    let normalise = brut
        .trim()
        .trim_matches(|c: char| c.is_ascii_punctuation())
        .to_ascii_lowercase();
    let sentiment = LABELS
        .iter()
        .find(|l| normalise == **l || normalise.starts_with(*l))
        .ok_or_else(|| {
            format!("Le moteur n'a pas rendu une tonalité reconnue (« {brut} »), rien parmi {LABELS:?}.")
        })?
        .to_string();

    // exp(moyenne des logprobs) : la vraisemblance géométrique moyenne des
    // tokens rendus. Absente (serveur qui ignore `logprobs`) : le score reste
    // à 1.0 plutôt que d'en inventer un.
    let logprobs: Vec<f32> = choix
        .get("logprobs")
        .and_then(|l| l.get("content"))
        .and_then(|c| c.as_array())
        .map(|tokens| {
            tokens
                .iter()
                .filter_map(|t| t.get("logprob").and_then(|lp| lp.as_f64()))
                .map(|lp| lp as f32)
                .collect()
        })
        .unwrap_or_default();
    let score = if logprobs.is_empty() {
        1.0
    } else {
        (logprobs.iter().sum::<f32>() / logprobs.len() as f32).exp()
    };

    Ok(SentimentResult {
        sentiment,
        score,
        // Pas de classification multi-classe embarquée : seule la tonalité
        // retenue a une confiance mesurée. Une ventilation par classe ici
        // serait inventée ; on ne la rend pas plutôt que de la fabriquer.
        breakdown: Vec::new(),
    })
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
