//! codeos-workflow — mechanical checkpoint governance for the three DBA-6 workflows.
//!
//! The tool derives each checkpoint's state from canonical evidence and receipts, blocks a later
//! checkpoint while a required earlier one is unresolved, and reports the exact missing condition
//! and one next action. It makes no semantic judgment: it never decides whether a direction is
//! good, whether UX is understandable, whether evidence is persuasive, or which operational route
//! an observation belongs to.

pub mod architecture;
pub mod candidate;
pub mod checker;
pub mod claims;
pub mod contract;
pub mod event_evidence;
pub mod evidence;
pub mod evidence_set;
pub mod execution;
pub mod hashing;
pub mod health;
pub mod history;
pub mod instrument;
pub mod monitor;
pub mod obligations;
pub mod observations;
pub mod platform_assurance;
pub mod preflight;
pub mod project;
pub mod qualification;
pub mod receipts;
pub mod report;
pub mod retirement;
pub mod revalidation;
pub mod session;
pub mod store_cache;
pub mod validity;
pub mod test_evidence;
pub mod verification;

pub const EXIT_SUCCESS: i32 = 0;
pub const EXIT_USAGE: i32 = 1;
pub const EXIT_BLOCKED: i32 = 2;
pub const EXIT_ERROR: i32 = 3;
