use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultManifest {
    pub protocol: String,
    pub format_version: String,
    pub vault_id: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyEnvelope {
    pub protocol: String,
    pub kdf: String,
    pub salt: String,
    pub nonce: String,
    pub ciphertext: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryStatus {
    pub recovery_exported: bool,
    pub last_exported_at: Option<String>,
    pub last_tested_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultStatus {
    pub vault_id: String,
    pub locked: bool,
    pub frozen: bool,
    pub frozen_reason: Option<String>,
    pub recovery_exported: bool,
    pub last_recovery_test: Option<String>,
    pub source_count: u64,
    pub fragment_count: u64,
    pub memory_inbox_count: u64,
    pub accepted_memory_count: u64,
    pub conflict_count: u64,
    pub context_pack_count: u64,
    pub external_models_enabled: bool,
    pub network_activity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceSummary {
    pub source_id: String,
    pub source_type: String,
    pub provider: String,
    pub external_id: Option<String>,
    pub title: Option<String>,
    pub captured_at: String,
    pub occurred_from: Option<String>,
    pub occurred_to: Option<String>,
    pub sensitivity: String,
    pub state: String,
    pub fragment_count: u64,
    pub content_object_id: String,
    pub integrity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentSummary {
    pub fragment_id: String,
    pub source_id: String,
    pub external_id: Option<String>,
    pub parent_external_id: Option<String>,
    pub role: Option<String>,
    pub occurred_at: Option<String>,
    pub locator: serde_json::Value,
    pub text: String,
    pub sensitivity: String,
    pub secret_candidate: bool,
    pub injection_candidate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySummary {
    pub memory_id: String,
    pub memory_type: String,
    pub statement: String,
    pub epistemic_status: String,
    pub review_state: String,
    pub evidence_strength: String,
    pub valid_from: Option<String>,
    pub valid_to: Option<String>,
    pub sensitivity: String,
    pub third_party: bool,
    pub current_revision: i64,
    pub evidence: Vec<FragmentSummary>,
    pub created_at: String,
    pub reviewed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub fragment: FragmentSummary,
    pub source_title: Option<String>,
    pub source_provider: String,
    pub source_state: String,
    pub rank: f64,
    pub accepted_memories: Vec<String>,
    pub contradictions: Vec<String>,
    pub why_used: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPolicy {
    pub policy_version: String,
    pub allowed_sensitivity: Vec<String>,
    pub include_third_party: bool,
    pub include_candidates: bool,
    pub include_disputed: bool,
    pub secret_allowed: bool,
}

impl Default for ContextPolicy {
    fn default() -> Self {
        Self {
            policy_version: "1.0.0".into(),
            allowed_sensitivity: vec!["PERSONAL".into(), "SENSITIVE".into()],
            include_third_party: false,
            include_candidates: true,
            include_disputed: true,
            secret_allowed: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextTarget {
    pub provider: String,
    pub model: String,
    pub max_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextIntegrity {
    pub canonical_digest: String,
    pub builder_version: String,
    pub signed_by_device: String,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPack {
    pub protocol: String,
    pub pack_id: String,
    pub vault_id: String,
    pub purpose: String,
    pub query: String,
    pub created_at: String,
    pub expires_at: String,
    pub temporal_cutoff: String,
    pub target: ContextTarget,
    pub policy: ContextPolicy,
    pub summary: String,
    pub active_constraints: Vec<String>,
    pub goals: Vec<String>,
    pub memory_items: Vec<MemorySummary>,
    pub contradictions: Vec<String>,
    pub source_fragments: Vec<FragmentSummary>,
    pub omissions: Vec<String>,
    pub redactions: Vec<String>,
    pub integrity: ContextIntegrity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditVerification {
    pub valid: bool,
    pub event_count: u64,
    pub checked_device_id: String,
    pub last_event_hash: Option<String>,
    pub failure: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortableExportReport {
    pub output_path: String,
    pub encrypted: bool,
    pub source_count: u64,
    pub memory_count: u64,
    pub digest: String,
}
