const PREAMBLE: &str = "\
> [INFERRED] fields were populated automatically by extracting risk bullets from the source\n\
> document — verify before submitting. [FILL] fields require human or model authorship.\n\
> ADR candidates are non-authoritative until routed through the Specification Package,\n\
> an architecture scope, normal engineering, or no action.";

const RISKS_HEADING: &str = "## Architectural Risks";

pub struct GenerateAdrCandidatesArgs<'a> {
    pub source: &'a str,
}

pub fn run(args: GenerateAdrCandidatesArgs) -> i32 {
    let content = match std::fs::read_to_string(args.source) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: cannot read source file '{}': {}", args.source, e);
            return crate::EXIT_USAGE;
        }
    };

    let lines: Vec<&str> = content.lines().collect();

    let heading_idx = match lines.iter().position(|l| l.trim_end() == RISKS_HEADING) {
        Some(i) => i,
        None => {
            eprintln!(
                "error: no \"{}\" section found in {}",
                RISKS_HEADING, args.source
            );
            return crate::EXIT_SUCCESS;
        }
    };

    let end_idx = lines[heading_idx + 1..]
        .iter()
        .position(|l| l.starts_with("## "))
        .map(|rel| heading_idx + 1 + rel)
        .unwrap_or(lines.len());

    let risks: Vec<&str> = lines[heading_idx + 1..end_idx]
        .iter()
        .filter_map(|l| {
            if l.starts_with("- ") || l.starts_with("* ") {
                let text = l[2..].trim();
                if text.is_empty() {
                    None
                } else {
                    Some(text)
                }
            } else {
                None
            }
        })
        .collect();

    if risks.is_empty() {
        eprintln!(
            "error: \"{}\" section in {} contains no risk bullets",
            RISKS_HEADING, args.source
        );
        return crate::EXIT_SUCCESS;
    }

    let mut report = String::from("# ADR Candidates\n");
    for (i, risk) in risks.iter().enumerate() {
        report.push_str(&format!(
            "\n## Candidate {n}\n\n\
Decision needed: {risk} [INFERRED]\n\
Why now: [FILL]\n\
Features affected: [FILL]\n\
Options: [FILL]\n\
Risk if deferred: [FILL]\n\
Does this affect behavior: [FILL]\n\
Recommended route: [FILL]\n\
- Specification Package / refinement\n\
- architecture scope\n\
- normal engineering\n\
- no action yet\n",
            n = i + 1,
            risk = risk,
        ));
    }

    println!("{}\n", PREAMBLE);
    print!("{}", report);
    crate::EXIT_SUCCESS
}
