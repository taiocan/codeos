//! Workflow contracts as plain data: an ordered list of checkpoints, each with an applicability
//! condition and a list of requirements the checker knows how to evaluate. This is not a DSL and
//! not user-authored — the three functions below are the whole vocabulary.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Workflow {
    Bootstrap,
    Feature,
    Operation,
}

impl Workflow {
    pub fn slug(&self) -> &'static str {
        match self {
            Workflow::Bootstrap => "bootstrap",
            Workflow::Feature => "feature",
            Workflow::Operation => "operation",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "bootstrap" => Some(Workflow::Bootstrap),
            "feature" => Some(Workflow::Feature),
            "operation" => Some(Workflow::Operation),
            _ => None,
        }
    }
}

/// When a checkpoint applies to a given subject. `WhenGui` / `WhenPersistence` are resolved from
/// the Contract's Feature Impact Accounting and its GUI-visible-outcome declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Applies {
    Always,
    WhenGui,
    WhenPersistence,
    WhenArchApproval,
}

/// A single condition the checker can evaluate. Kept deliberately small.
#[derive(Debug, Clone)]
pub enum Requirement {
    /// Charter exists and records `approval.by` / `approval.at`.
    CharterApproved,
    /// Intent + Contract + Event Schema exist and record the joint package approval.
    SpecPackageApproved,
    /// Feature Workflow v3: the package carries one consistent semantic revision.
    SemanticPackageRevision,
    /// `codeos.yaml` exists and validates against the active configuration's mechanics policy.
    CodeosYamlValid,
    /// The named `## <heading>` section of the Contract is present and has no unfilled placeholder.
    ContractSection(&'static str),
    /// DBA-9: the Contract declares a valid preserve/advance contribution against the accepted
    /// primary-journey frontier.
    ProductJourneyContribution,
    /// A current (non-stale) receipt exists for `checkpoint` whose `result` is one of `results`.
    Receipt {
        checkpoint: &'static str,
        results: &'static [&'static str],
    },
    /// A reviewer assessment record for this feature at `stage` exists, is bound to current
    /// evidence, and permits progression.
    ReviewRecord { stage: &'static str },
    /// A verification `check` live-executes; `status`/`next` report it as unverified.
    LiveVerification {
        name: &'static str,
        mechanic: &'static str,
    },
    /// F5 under Feature Workflow v3: required only when the Contract declares reality-dependent claims.
    ProductionSourceEvidence,
    /// Operation O1: the latest `operation_route` receipt carries a non-empty observation statement.
    ObservationStated,
    /// Operation v2 O2: the immutable observation has exactly one route and explicit target.
    ObservationRouted,
    /// Operation O4: the resolution evidence appropriate to the recorded route exists.
    RouteResolution,
    /// F9 under Feature Workflow v16: every Charter obligation the feature owns is VERIFIED.
    ObligationsVerified,
}

/// Stable machine-readable cause of a reported next action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReasonCode {
    CharterApproval,
    SpecificationApproval,
    SemanticRevision,
    Configuration,
    ContractSection,
    JourneyContribution,
    Decision,
    IndependentReview,
    Verification,
    ProductionEvidence,
    ObservationStatement,
    ObservationRoute,
    ObservationResolution,
    BootstrapIncomplete,
    SemanticChangeRequired,
    ArchitectureChangeRequired,
    RevalidationRequired,
}

/// One atomic next operation. A type never names a workflow outcome or a multi-step objective.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    AuthorArtifact,
    RepairConfiguration,
    ReviseSpecification,
    ApproveArtifact,
    RecordDecision,
    FollowDecisionOutcome,
    RunVerification,
    CollectProductionEvidence,
    RepairFailure,
    PrepareCandidate,
    RunReview,
    ResolveReviewFinding,
    RestoreReviewHistory,
    ReassessArchitecture,
    /// Reserved schema-v3 value from WO-05; never emitted, because it names the multi-step
    /// revalidation outcome rather than its next operation (WO-10).
    RevalidateAcceptance,
    RecordObservation,
    RouteObservation,
    CloseObservation,
    ResolveRoutedTarget,
    /// Commit present implementation changes so a revalidation trigger can be identified.
    CommitChange,
}

impl Requirement {
    /// The action for this requirement's next action; `repair` marks a failure to repair.
    /// Evaluation sites that differ set an explicit override instead.
    pub fn action(&self, repair: bool) -> ActionType {
        use ActionType as A;
        match self {
            Self::CharterApproved if repair => A::AuthorArtifact,
            Self::CharterApproved | Self::SpecPackageApproved => A::ApproveArtifact,
            Self::CodeosYamlValid => A::RepairConfiguration,
            Self::SemanticPackageRevision => A::ReviseSpecification,
            Self::ContractSection(_) | Self::ProductJourneyContribution => A::AuthorArtifact,
            Self::Receipt { .. } if repair => A::FollowDecisionOutcome,
            Self::Receipt { .. } => A::RecordDecision,
            Self::ReviewRecord { .. } if repair => A::ResolveReviewFinding,
            Self::ReviewRecord { .. } => A::RunReview,
            Self::LiveVerification { .. } if repair => A::RepairFailure,
            Self::LiveVerification { .. } => A::RunVerification,
            Self::ProductionSourceEvidence => A::CollectProductionEvidence,
            Self::ObservationStated => A::RecordObservation,
            Self::ObservationRouted => A::RouteObservation,
            Self::RouteResolution => A::CloseObservation,
            Self::ObligationsVerified => A::RunVerification,
        }
    }

    pub fn reason(&self) -> ReasonCode {
        match self {
            Self::CharterApproved => ReasonCode::CharterApproval,
            Self::SpecPackageApproved => ReasonCode::SpecificationApproval,
            Self::SemanticPackageRevision => ReasonCode::SemanticRevision,
            Self::CodeosYamlValid => ReasonCode::Configuration,
            Self::ContractSection(_) => ReasonCode::ContractSection,
            Self::ProductJourneyContribution => ReasonCode::JourneyContribution,
            Self::Receipt { .. } => ReasonCode::Decision,
            Self::ReviewRecord { .. } => ReasonCode::IndependentReview,
            Self::LiveVerification { .. } => ReasonCode::Verification,
            Self::ProductionSourceEvidence => ReasonCode::ProductionEvidence,
            Self::ObservationStated => ReasonCode::ObservationStatement,
            Self::ObservationRouted => ReasonCode::ObservationRoute,
            Self::RouteResolution => ReasonCode::ObservationResolution,
            Self::ObligationsVerified => ReasonCode::Verification,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Checkpoint {
    pub id: &'static str,
    pub title: &'static str,
    pub applies: Applies,
    pub requirements: Vec<Requirement>,
}

pub fn contract_for(w: Workflow, feature_version: u8, operation_version: u8) -> Vec<Checkpoint> {
    match w {
        Workflow::Feature => feature_contract(feature_version),
        Workflow::Bootstrap => bootstrap_contract(),
        Workflow::Operation => operation_contract(operation_version),
    }
}

fn cp(
    id: &'static str,
    title: &'static str,
    applies: Applies,
    requirements: Vec<Requirement>,
) -> Checkpoint {
    Checkpoint {
        id,
        title,
        applies,
        requirements,
    }
}

/// Feature Development Workflow Contract v1 — F1..F9.
pub fn feature_contract(version: u8) -> Vec<Checkpoint> {
    use Requirement::*;
    vec![
        cp(
            "F1",
            "Specification",
            Applies::Always,
            if version >= 3 {
                vec![
                    SpecPackageApproved,
                    SemanticPackageRevision,
                    ContractSection("Validation Questions"),
                    ContractSection("Reality Basis"),
                ]
            } else {
                vec![SpecPackageApproved, ContractSection("Validation Questions")]
            },
        ),
        cp(
            "F2",
            "Preparation",
            Applies::Always,
            if version >= 2 {
                vec![
                    ContractSection("Feature Impact Accounting"),
                    ProductJourneyContribution,
                ]
            } else {
                vec![ContractSection("Feature Impact Accounting")]
            },
        ),
        cp(
            "F3",
            "Vertical implementation",
            Applies::Always,
            if version >= 2 {
                vec![
                    LiveVerification {
                        name: "integration smoke",
                        mechanic: "smoke",
                    },
                    LiveVerification {
                        name: "continuous product journey",
                        mechanic: "product_journey",
                    },
                ]
            } else {
                vec![LiveVerification {
                    name: "integration smoke",
                    mechanic: "smoke",
                }]
            },
        ),
        cp(
            "F4",
            "Early Development Preview",
            Applies::WhenGui,
            vec![Receipt {
                checkpoint: "early_preview",
                results: &["direction_confirmed"],
            }],
        ),
        cp("F5", "Full Verification", Applies::Always, {
            let mut requirements = vec![
                LiveVerification {
                    name: "approved behavior",
                    mechanic: "behavior",
                },
                LiveVerification {
                    name: "repeatability",
                    mechanic: "repeatability",
                },
            ];
            if version >= 3 {
                requirements.push(ProductionSourceEvidence);
            }
            requirements
        }),
        cp(
            "F5d",
            "Data-integrity verification",
            Applies::WhenPersistence,
            vec![LiveVerification {
                name: "data integrity vs real PostgreSQL",
                mechanic: "data_integrity",
            }],
        ),
        cp(
            "F5g",
            "Integrated browser verification",
            Applies::WhenGui,
            vec![LiveVerification {
                name: "integrated Playwright journey",
                mechanic: "playwright",
            }],
        ),
        cp(
            "F6",
            "Reconciliation",
            Applies::Always,
            vec![Receipt {
                checkpoint: "reconciliation",
                results: &["completed"],
            }],
        ),
        cp(
            "F7",
            "Independent Review",
            Applies::Always,
            vec![ReviewRecord { stage: "8" }],
        ),
        cp(
            "F8",
            "Final Human UX Validation",
            Applies::WhenGui,
            vec![Receipt {
                checkpoint: "final_ux_validation",
                results: &["accepted"],
            }],
        ),
        cp("F9", "Acceptance", Applies::Always, {
            let mut requirements = Vec::new();
            if version >= 9 {
                requirements.push(LiveVerification {
                    name: "project qualification",
                    mechanic: "project_qualification",
                });
            }
            if version >= 16 {
                requirements.push(ObligationsVerified);
            }
            requirements.push(Receipt {
                checkpoint: "acceptance",
                results: &["accepted"],
            });
            requirements
        }),
    ]
}

/// Solution Bootstrap Workflow Contract v1 — B1..B5.
pub fn bootstrap_contract() -> Vec<Checkpoint> {
    use Requirement::*;
    vec![
        cp("B1", "Purpose", Applies::Always, vec![CharterApproved]),
        cp(
            "B2",
            "Architecture and configuration",
            Applies::Always,
            vec![CodeosYamlValid],
        ),
        cp(
            "B3",
            "Integrated baseline",
            Applies::Always,
            vec![LiveVerification {
                name:
                    "clean migration + DB<->backend<->GUI integration + shipped tests + Playwright",
                mechanic: "baseline",
            }],
        ),
        cp(
            "B4",
            "Initial Product Preview",
            Applies::Always,
            vec![Receipt {
                checkpoint: "initial_product_preview",
                results: &["direction_confirmed"],
            }],
        ),
        cp("B5", "Ready for first feature", Applies::Always, vec![]),
    ]
}

/// Operation & Learning Workflow Contract v1 — O1..O5.
/// The six valid operational routes (doctrine's Post-Acceptance Learning table, plus no_action).
pub const OPERATION_ROUTES: &[&str] = &[
    "implementation_defect",
    "new_feature",
    "specification_change",
    "charter_change",
    "architecture_reassessment",
    "no_action",
];

/// Operation & Learning Workflow Contract v3 adds this route to the v2 observation routes.
pub const RETIREMENT_ROUTE: &str = "feature_retirement";

pub fn operation_contract(version: u8) -> Vec<Checkpoint> {
    use Requirement::*;
    vec![
        cp(
            "O1",
            "Observation",
            Applies::Always,
            vec![ObservationStated],
        ),
        cp(
            "O2",
            "Route decision",
            Applies::Always,
            if version >= 2 {
                vec![ObservationRouted]
            } else {
                vec![Receipt {
                    checkpoint: "operation_route",
                    results: OPERATION_ROUTES,
                }]
            },
        ),
        cp("O3", "Routed action", Applies::Always, vec![]),
        cp("O4", "Verification", Applies::Always, vec![RouteResolution]),
        cp("O5", "Close", Applies::Always, vec![]),
    ]
}
