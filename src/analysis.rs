//! Analysis JSON schema — the primary interface between an external LLM and the wiki.
//!
//! See `docs/design/design.md` for the full `analysis.json` contract.
//! The JSON Schema is generated from these types via `cargo run --bin gen-schema`.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Document type — the external LLM classifies the source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum DocType {
    ResearchPaper,
    BlogPost,
    Transcript,
    Thread,
    Note,
    BookChapter,
}

/// Confidence level for a claim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    High,
    Medium,
    Low,
}

/// Wiki page category.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum PageType {
    Concept,
    SourceSummary,
    QueryResult,
    Contradiction,
}

/// Action to take when integrating a [`SuggestedPage`] into the wiki.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    /// Create a new page. Fails if the slug already exists.
    Create,
    /// Replace the body of an existing page, merging frontmatter fields.
    Update,
    /// Add a new section to the end of an existing page's body.
    Append,
}

/// The dimension along which two claims contradict each other.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Dimension {
    /// Each claim holds in a different context or domain.
    Context,
    /// One claim was superseded by the other over time.
    Time,
    /// Each claim applies at a different scale.
    Scale,
    /// Different measurement methodologies yield different results.
    Methodology,
    /// The field has not resolved this dispute.
    OpenDispute,
}

/// Lifecycle status of a contradiction node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Status {
    /// The contradiction has been analysed and its resolution documented.
    Resolved,
    /// The contradiction is unresolved and awaiting enrichment.
    Active,
    /// An external LLM is currently enriching this contradiction.
    UnderAnalysis,
}

/// A single factual claim extracted from the source document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Claim {
    /// The claim statement.
    pub text: String,
    /// How confident the external LLM is in this claim.
    pub confidence: Confidence,
    /// Section of the source document where the claim appears.
    pub section: String,
}

/// A wiki page to create, update, or append to.
///
/// The wiki writes the page as `{slug}.md` with auto-generated frontmatter.
/// `body` is plain Markdown without frontmatter — the wiki generates frontmatter
/// from the other fields so the external LLM never needs to write YAML.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SuggestedPage {
    /// Canonical file path relative to the wiki root (e.g. `concepts/mixture-of-experts`).
    pub slug: String,
    /// Human-readable page title.
    pub title: String,
    /// Page category.
    #[serde(rename = "type")]
    pub page_type: PageType,
    /// Whether to create, replace, or extend the page.
    pub action: Action,
    /// One-sentence summary surfaced in search results and frontmatter.
    pub tldr: String,
    /// Full Markdown body without frontmatter.
    pub body: String,
    /// Tags for search and cross-referencing.
    pub tags: Vec<String>,
    /// Conditions under which an LLM should retrieve this page.
    pub read_when: Vec<String>,
}

/// A contradiction between two claims from different sources.
///
/// Contradictions are first-class knowledge nodes — they are never deleted,
/// only enriched. A resolved contradiction still carries the analysis that
/// explains *why* the sources disagreed; that explanation is the knowledge.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Contradiction {
    /// Short descriptive title for the contradiction page.
    pub title: String,
    /// The first claim.
    pub claim_a: String,
    /// Slug of the source page for claim A.
    pub source_a: String,
    /// The second claim.
    pub claim_b: String,
    /// Slug of the source page for claim B.
    pub source_b: String,
    /// The dimension along which the claims contradict.
    pub dimension: Dimension,
    /// What the tension reveals that neither source captures alone.
    pub epistemic_value: String,
    /// Current lifecycle status.
    pub status: Status,
    /// Explanation of the resolution. Present only when `status` is `Resolved`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
}

/// The primary interface between an external LLM and the wiki.
///
/// Produced by the external LLM after reading a source document and
/// (optionally) calling `wiki context` to check for existing pages.
/// Consumed by `wiki ingest`. See `docs/design/design.md` for the full contract.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Analysis {
    /// Path or URL to the original source document.
    pub source: String,
    /// Document type classification.
    pub doc_type: DocType,
    /// Title of the source document.
    pub title: String,
    /// BCP 47 language code (e.g. `"en"`).
    pub language: String,
    /// Factual claims extracted from the source.
    pub claims: Vec<Claim>,
    /// Key concepts mentioned in the source.
    pub concepts: Vec<String>,
    /// Verbatim quotations worth preserving.
    pub key_quotes: Vec<String>,
    /// Gaps in the source's coverage or evaluation.
    pub data_gaps: Vec<String>,
    /// Pages to create, update, or append to.
    pub suggested_pages: Vec<SuggestedPage>,
    /// Contradictions detected against existing wiki pages.
    /// Should be empty if the LLM did not call `wiki context` first.
    pub contradictions: Vec<Contradiction>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analysis_json_round_trip() {
        let analysis = Analysis {
            source: "test-source.pdf".into(),
            doc_type: DocType::ResearchPaper,
            title: "Test Paper".into(),
            language: "en".into(),
            claims: vec![Claim {
                text: "sparse MoE reduces effective compute 8x".into(),
                confidence: Confidence::High,
                section: "Results".into(),
            }],
            concepts: vec!["mixture-of-experts".into()],
            key_quotes: vec!["key quote here".into()],
            data_gaps: vec!["no fine-tuning evaluation".into()],
            suggested_pages: vec![SuggestedPage {
                slug: "concepts/mixture-of-experts".into(),
                title: "Mixture of Experts".into(),
                page_type: PageType::Concept,
                action: Action::Create,
                tldr: "Sparse routing of tokens to expert subnetworks.".into(),
                body: "## Overview\n\nMoE routes tokens to experts.".into(),
                tags: vec!["transformers".into(), "scaling".into()],
                read_when: vec!["Reasoning about MoE architecture".into()],
            }],
            contradictions: vec![],
        };

        let json = serde_json::to_string(&analysis).expect("serialise");
        let round_tripped: Analysis = serde_json::from_str(&json).expect("deserialise");
        assert_eq!(analysis, round_tripped);
    }
}
