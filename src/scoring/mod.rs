//! Unified scoring module.
//!
//! Provides MR/PR review scoring from expert findings severity.

pub mod review;

pub use review::{
    compute_overall_with_config, expert_score, expert_score_with_config, score_to_risk_level_with_config,
    weighted_overall_score, ExpertScoreRecord, ReviewScoreRecord,
};
