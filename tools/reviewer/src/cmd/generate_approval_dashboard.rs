use serde::Deserialize;

const PREAMBLE: &str = "\
> [INFERRED] fields were populated automatically from the feature registry — verify before\n\
> submitting. [FILL] fields require human or model authorship. This dashboard is a navigation\n\
> aid, not a decision record — the registry and change records remain authoritative.";

const VALID_STATUS_VALUES: &[&str] = &["hypothesized", "active", "suspended", "blocked", "complete"];

#[derive(Debug, Deserialize)]
struct Registry {
    schema_version: u32,
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
    #[serde(default)]
    notes: Option<String>,
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

    // Schema version pre-probe: check for schema_version: 2 before attempting strict parse
    let pre_probe: serde_yaml::Value = match serde_yaml::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error: cannot parse registry file '{}': {}", args.registry, e);
            return crate::EXIT_USAGE;
        }
    };

    let schema_version_value = pre_probe.get("schema_version");
    let schema_version_u64 = schema_version_value.and_then(|v| v.as_u64());

    if schema_version_u64 != Some(2) {
        let found_display = match schema_version_value {
            None => "missing".to_string(),
            Some(v) => match v.as_u64() {
                Some(n) => n.to_string(),
                None => format!("{:?} (not a number)", v),
            },
        };
        eprintln!(
            "error: registry '{}' does not declare schema_version: 2 (found: {})",
            args.registry, found_display
        );
        eprintln!("This registry predates the v2 schema or uses an incompatible version.");
        eprintln!("See docs/registry-v2-migration.md for migration instructions.");
        return crate::EXIT_USAGE;
    }

    let registry: Registry = match serde_yaml::from_str(&content) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: cannot parse registry file '{}': {}", args.registry, e);
            return crate::EXIT_USAGE;
        }
    };

    // Validate status values
    let mut invalid_entries = Vec::new();
    for feature in &registry.features {
        if !VALID_STATUS_VALUES.contains(&feature.status.as_str()) {
            invalid_entries.push((feature.feature_id.clone(), feature.status.clone()));
        }
    }

    if !invalid_entries.is_empty() {
        eprintln!("error: registry '{}' contains invalid status values:", args.registry);
        for (id, status) in &invalid_entries {
            eprintln!("  feature_id '{}' has status '{}' (not in valid set)", id, status);
        }
        eprintln!("\nValid status values: {}", VALID_STATUS_VALUES.join(", "));
        return crate::EXIT_USAGE;
    }

    let active: Vec<&FeatureEntry> = registry
        .features
        .iter()
        .filter(|f| f.status == "active" || f.status == "hypothesized")
        .collect();

    if active.is_empty() {
        eprintln!("error: no active or hypothesized features found in {}", args.registry);
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

        let status_note = if feature.status == "hypothesized" {
            "\n⚠️  HYPOTHESIZED — requires Stage 1 review before advancing\n"
        } else {
            ""
        };

        report.push_str(&format!(
            "\n## {feature_id}: {slug}{status_note}\n\
Active features: {feature_id} [INFERRED]\n\
Current stage: {stage}\n\
Reviewer recommendation: [FILL]\n\
Open blockers: {blockers}\n\
Next human decision: [FILL]\n\
Risk: [FILL]\n",
            feature_id = feature.feature_id,
            slug = feature.slug,
            status_note = status_note,
            stage = stage,
            blockers = blockers,
        ));
    }

    println!("{}\n", PREAMBLE);
    print!("{}", report);
    crate::EXIT_SUCCESS
}
