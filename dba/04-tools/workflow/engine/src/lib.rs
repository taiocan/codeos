//! codeos-workflow — mechanical checkpoint governance for the three DBA-6 workflows.
//!
//! The tool derives each checkpoint's state from canonical evidence and receipts, blocks a later
//! checkpoint while a required earlier one is unresolved, and reports the exact missing condition
//! and one next action. It makes no semantic judgment: it never decides whether a direction is
//! good, whether UX is understandable, whether evidence is persuasive, or which operational route
//! an observation belongs to.

pub mod checker;
pub mod contract;
pub mod evidence;
pub mod hashing;
pub mod project;
pub mod receipts;
pub mod report;
pub mod verification;

pub const EXIT_SUCCESS: i32 = 0;
pub const EXIT_USAGE: i32 = 1;
pub const EXIT_BLOCKED: i32 = 2;
pub const EXIT_ERROR: i32 = 3;
