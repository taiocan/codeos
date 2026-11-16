//! Reviewer Tool Contract v22: the one semantically identified bulk-evidence projection. Any input
//! or field this module does not recognise is shown whole.
//!
//! The projection splices the source text: everything outside the projected array, and every
//! coordinate it shows, is the evidence file's exact bytes, and the array hash covers the array's
//! exact bytes. Nothing is re-serialized, so no number or key order can change in display.

use crate::packet::sha256_str;

pub const ROUTE_GEOMETRY: &str = "route-geometry/v1";
const GEOMETRY: [&str; 3] = ["visible_result", "route", "geometry"];
const SAMPLES: usize = 16;

type Span = (usize, usize);

/// The packet text for production evidence `schema: 1` or `2` with each GeoJSON LineString's
/// coordinates at `observations[].visible_result.route.geometry` projected, or `None` when it
/// holds nothing to project. CodeOS does not define `visible_result`: the payload declares the
/// semantics through `type: "LineString"`. The caller passes only an input the Review Input
/// Manifest included as production evidence.
pub fn route_geometry(raw: &str) -> Option<String> {
    let parsed: serde_json::Value = serde_json::from_str(raw).ok()?;
    let schema = parsed.get("schema")?;
    if *schema != serde_json::json!(1) && *schema != serde_json::json!(2) {
        return None;
    }
    let b = raw.as_bytes();
    let root = value_start(b, 0)?;
    let (observations, _) = member(b, root, "observations")?;
    let mut projections = Vec::new();
    for (start, _) in elements(b, observations)? {
        let mut geometry = Some((start, start));
        for key in GEOMETRY {
            geometry = geometry.and_then(|(at, _)| member(b, at, key));
        }
        let Some((geometry, _)) = geometry else {
            continue;
        };
        let line_string = member(b, geometry, "type")
            .is_some_and(|(start, end)| &raw[start..end] == "\"LineString\"");
        let Some(coordinates) = line_string
            .then(|| member(b, geometry, "coordinates"))
            .flatten()
        else {
            continue;
        };
        if let Some(projection) = project(raw, coordinates) {
            projections.push((coordinates, projection));
        }
    }
    if projections.is_empty() {
        return None;
    }
    let mut shown = raw.to_string();
    for ((start, end), projection) in projections.into_iter().rev() {
        shown.replace_range(start..end, &projection);
    }
    Some(shown)
}

/// LineString coordinates longer than the sample, every position two or three numbers (RFC 7946).
fn project(raw: &str, span: Span) -> Option<String> {
    let b = raw.as_bytes();
    let text = |(start, end): Span| &raw[start..end];
    let points = elements(b, span.0)?;
    if points.len() <= SAMPLES {
        return None;
    }
    // (source token, value) of x and y; the bounding box shows each extreme's source token.
    let mut xy = Vec::with_capacity(points.len());
    for &point in &points {
        let values = elements(b, point.0)?;
        if !(2..=3).contains(&values.len()) {
            return None;
        }
        let mut numbers = Vec::with_capacity(values.len());
        for &value in &values {
            let number = serde_json::from_str::<serde_json::Number>(text(value))
                .ok()?
                .as_f64()?;
            numbers.push((text(value), number));
        }
        xy.push([numbers[0], numbers[1]]);
    }
    let extreme = |axis: usize, max: bool| {
        xy.iter()
            .map(|point| point[axis])
            .reduce(|best, next| {
                let better = if max {
                    next.1 > best.1
                } else {
                    next.1 < best.1
                };
                if better {
                    next
                } else {
                    best
                }
            })
            .map(|(token, _)| token)
    };
    let last = points.len() - 1;
    let samples: Vec<&str> = (0..SAMPLES)
        .map(|i| text(points[i * last / (SAMPLES - 1)]))
        .collect();
    Some(format!(
        "{{\"projection\":\"{ROUTE_GEOMETRY}\",\"count\":{},\"sha256\":\"{}\",\"first\":{},\"last\":{},\"bbox\":[{},{},{},{}],\"samples\":[{}]}}",
        points.len(),
        sha256_str(text(span)),
        text(points[0]),
        text(points[last]),
        extreme(0, false)?,
        extreme(1, false)?,
        extreme(0, true)?,
        extreme(1, true)?,
        samples.join(",")
    ))
}

fn skip_ws(b: &[u8], mut i: usize) -> usize {
    while i < b.len() && b[i].is_ascii_whitespace() {
        i += 1;
    }
    i
}

fn value_start(b: &[u8], i: usize) -> Option<usize> {
    let i = skip_ws(b, i);
    (i < b.len()).then_some(i)
}

/// End (exclusive) of the string whose opening quote is at `i`.
fn string_end(b: &[u8], mut i: usize) -> Option<usize> {
    i += 1;
    while i < b.len() {
        match b[i] {
            b'\\' => i += 2,
            b'"' => return Some(i + 1),
            _ => i += 1,
        }
    }
    None
}

/// End (exclusive) of the JSON value starting at `i`.
fn value_end(b: &[u8], i: usize) -> Option<usize> {
    match *b.get(i)? {
        b'"' => string_end(b, i),
        b'{' | b'[' => {
            let mut depth = 0usize;
            let mut j = i;
            while j < b.len() {
                match b[j] {
                    b'"' => {
                        j = string_end(b, j)?;
                        continue;
                    }
                    b'{' | b'[' => depth += 1,
                    b'}' | b']' => {
                        depth -= 1;
                        if depth == 0 {
                            return Some(j + 1);
                        }
                    }
                    _ => {}
                }
                j += 1;
            }
            None
        }
        _ => {
            let mut j = i;
            while j < b.len() && !matches!(b[j], b',' | b']' | b'}') && !b[j].is_ascii_whitespace()
            {
                j += 1;
            }
            (j > i).then_some(j)
        }
    }
}

/// Spans of the elements of the array at `i`.
fn elements(b: &[u8], i: usize) -> Option<Vec<Span>> {
    if *b.get(i)? != b'[' {
        return None;
    }
    let mut out = Vec::new();
    let mut j = skip_ws(b, i + 1);
    if *b.get(j)? == b']' {
        return Some(out);
    }
    loop {
        let end = value_end(b, j)?;
        out.push((j, end));
        j = skip_ws(b, end);
        match *b.get(j)? {
            b',' => j = value_start(b, j + 1)?,
            b']' => return Some(out),
            _ => return None,
        }
    }
}

/// The span of `key`'s value in the object at `i`; `None` when absent or duplicated. A key
/// written with escapes never matches, so it is shown whole.
fn member(b: &[u8], i: usize, key: &str) -> Option<Span> {
    if *b.get(i)? != b'{' {
        return None;
    }
    let mut found = None;
    let mut j = skip_ws(b, i + 1);
    if *b.get(j)? == b'}' {
        return None;
    }
    loop {
        if *b.get(j)? != b'"' {
            return None;
        }
        let key_end = string_end(b, j)?;
        let name = &b[j + 1..key_end - 1];
        j = skip_ws(b, key_end);
        if *b.get(j)? != b':' {
            return None;
        }
        let start = value_start(b, j + 1)?;
        let end = value_end(b, start)?;
        if name == key.as_bytes() {
            if found.is_some() {
                return None;
            }
            found = Some((start, end));
        }
        j = skip_ws(b, end);
        match *b.get(j)? {
            b',' => j = value_start(b, j + 1)?,
            b'}' => return found,
            _ => return None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    fn route(n: usize) -> Value {
        Value::Array(
            (0..n)
                .map(|i| json!([14.0 + i as f64 / 1000.0, 46.0 - i as f64 / 2000.0]))
                .collect(),
        )
    }

    fn evidence(schema: u64, coordinates: Value) -> String {
        json!({
            "schema": schema,
            "subject": "F-1",
            "observations": [{
                "claim_ids": ["c"],
                "visible_result": {"route": {"geometry": {"coordinates": coordinates, "type": "LineString"}}},
                "details": {"samples": route(5000)},
            }],
        })
        .to_string()
    }

    /// The source text of the first `"coordinates"` value.
    fn coordinates_text(raw: &str) -> &str {
        let start = raw.find("\"coordinates\":").unwrap() + "\"coordinates\":".len();
        &raw[start..value_end(raw.as_bytes(), start).unwrap()]
    }

    #[test]
    fn route_coordinates_in_schema_1_evidence_are_projected_deterministically() {
        let raw = evidence(1, route(5000));
        let shown = route_geometry(&raw).expect("projected");
        assert_eq!(shown, route_geometry(&raw).unwrap(), "deterministic");
        // Everything outside the array is the source bytes.
        let source = coordinates_text(&raw);
        let projection_text = coordinates_text(&shown);
        assert_eq!(shown.replacen(projection_text, source, 1), raw);

        let projection: Value = serde_json::from_str(projection_text).unwrap();
        // Compared with the source array parsed the same way, so tokens are compared exactly.
        let points: Value = serde_json::from_str(source).unwrap();
        assert_eq!(projection["projection"], ROUTE_GEOMETRY);
        assert_eq!(projection["count"], 5000);
        assert_eq!(
            projection["sha256"],
            sha256_str(source),
            "exact array bytes"
        );
        assert_eq!(projection["first"], points[0]);
        assert_eq!(projection["last"], points[4999]);
        assert_eq!(
            projection["bbox"],
            json!([points[0][0], points[4999][1], points[4999][0], points[0][1]])
        );
        let samples = projection["samples"].as_array().unwrap();
        assert_eq!(samples.len(), 16);
        assert_eq!((&samples[0], &samples[15]), (&points[0], &points[4999]));
        assert_eq!(samples[1], points[333]);
    }

    #[test]
    fn schema_2_projects_and_unsupported_or_ambiguous_values_are_shown_whole() {
        assert!(route_geometry(&evidence(2, route(5000))).is_some());
        assert_eq!(
            route_geometry(&evidence(3, route(5000))),
            None,
            "unknown schema"
        );
        assert_eq!(
            route_geometry(&evidence(1, route(16))),
            None,
            "nothing to save"
        );
        let raw = evidence(1, route(100));
        for (from, to, why) in [
            (
                "\"type\":\"LineString\"",
                "\"type\":\"Polygon\"",
                "other GeoJSON type",
            ),
            (",\"type\":\"LineString\"", "", "missing type"),
            (
                "\"type\":\"LineString\"",
                "\"type\":\"linestring\"",
                "inexact type",
            ),
            (
                "\"geometry\":{",
                "\"geometry\":{\"coordinates\":[],",
                "duplicate key",
            ),
            ("\"route\":{", "\"path\":{", "other location"),
        ] {
            assert!(raw.contains(from), "{why}: fixture");
            assert_eq!(route_geometry(&raw.replacen(from, to, 1)), None, "{why}");
        }
        let mut mixed = route(100);
        mixed[50] = json!(["14.0", 46.0]);
        assert_eq!(
            route_geometry(&evidence(1, mixed)),
            None,
            "non-numeric position"
        );
        let mut four = route(100);
        four[50] = json!([14.0, 46.0, 1.0, 2.0]);
        assert_eq!(
            route_geometry(&evidence(1, four)),
            None,
            "four-number position"
        );
        assert_eq!(
            route_geometry(&evidence(1, Value::Array(vec![json!({"x": 1}); 100]))),
            None
        );
        assert_eq!(route_geometry("not json"), None);
    }
}
