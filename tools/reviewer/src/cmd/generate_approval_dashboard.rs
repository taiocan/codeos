use serde::Deserialize;

const PREAMBLE: &str = "\
> [INFERRED] fields were populated automatically from the feature registry — verify before\n\
> submitting. [FILL] fields require human or model authorship. This dashboard is a navigation\n\
> aid, not a decision record — the registry and change records remain authoritative.";

#[derive(Debug, Deserialize)]
struct Registry {
    #[serde(default)]
    features: Vec<FeatureEntry>,
}

#[derive(Debug, Deserialize)]
struct FeatureEntry {
    feature_id: String,
    slug: String,
    status: String,
    #[serde(default)]
    current_stage: Option<i64>,
    #[serde(default)]
    blockers: Vec<String>,
}

pub struct GenerateApprovalDashboardArgs<'a> {
    pub registry: &'a str,
}

pub fn run(args: GenerateApprovalDashboardArgs) -> i32 {
    let content = match std::fs::read_to_string(args.registry) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: cannot read registry file '{}': {}", args.registry, e);
            return crate::EXIT_USAGE;
        }
    };

    let registry: Registry = match serde_yaml::from_str(&content) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: cannot parse registry file '{}': {}", args.registry, e);
            return crate::EXIT_USAGE;
        }
    };

    let active: Vec<&FeatureEntry> = registry
        .features
        .iter()
        .filter(|f| f.status == "active")
        .collect();

    if active.is_empty() {
        eprintln!("error: no active features found in {}", args.registry);
        return crate::EXIT_SUCCESS;
    }

    let mut report = String::from("# Approval Dashboard\n");
    for feature in active {
        let stage = match feature.current_stage {
            Some(n) => format!("{} [INFERRED]", n),
            None => "not started [INFERRED]".to_string(),
        };
        let blockers = if feature.blockers.is_empty() {
            "(none) [INFERRED]".to_string()
        } else {
            format!("[INFERRED]\n{}", feature.blockers.join("\n"))
        };
        report.push_str(&format!(
            "\n## {feature_id}: {slug}\n\n\
Active features: {feature_id} [INFERRED]\n\
Current stage: {stage}\n\
Reviewer recommendation: [FILL]\n\
Open blockers: {blockers}\n\
Next human decision: [FILL]\n\
Risk: [FILL]\n",
            feature_id = feature.feature_id,
            slug = feature.slug,
            stage = stage,
            blockers = blockers,
        ));
    }

    println!("{}\n", PREAMBLE);
    print!("{}", report);
    crate::EXIT_SUCCESS
}
