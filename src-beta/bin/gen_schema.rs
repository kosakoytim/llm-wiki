//! Generate `docs/design/analysis.schema.json` from the `Analysis` Rust type.
//!
//! Run:
//!   cargo run --bin gen_schema > docs/design/analysis.schema.json
//!
//! The output is a JSON Schema (draft-07) document describing the full
//! `analysis.json` contract. Keep it in sync by re-running after any
//! change to `src/analysis.rs`.

use llm_wiki::analysis::Analysis;
use schemars::schema_for;

fn main() {
    let schema = schema_for!(Analysis);
    println!("{}", serde_json::to_string_pretty(&schema).expect("serialise schema"));
}
