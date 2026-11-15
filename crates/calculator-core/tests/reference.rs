use calculator_core::{Step, evaluate, format_number, format_time, parse_time};
#[test]
fn javascript_reference_fixtures() {
    let fixtures: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../../../tests/reference/fixtures.json")).unwrap();
    for f in fixtures {
        let input = &f["input"];
        let expected = &f["expected"];
        match f["kind"].as_str().unwrap() {
            "parse" => assert_eq!(
                parse_time(input.as_str().unwrap()),
                expected.as_f64().unwrap(),
                "{f}"
            ),
            "time" => assert_eq!(
                format_time(input.as_f64().unwrap()),
                expected.as_str().unwrap(),
                "{f}"
            ),
            "number" => assert_eq!(
                format_number(input.as_f64().unwrap()),
                expected.as_str().unwrap(),
                "{f}"
            ),
            "evaluate" => {
                let steps: Vec<_> = input
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|s| Step::Value {
                        op: s["op"].as_str().and_then(|s| s.chars().next()),
                        value: s["value"].as_f64().unwrap(),
                        scalar: false,
                    })
                    .collect();
                let actual = evaluate(&steps);
                let expected = expected.as_f64().unwrap();
                assert!(
                    (actual - expected).abs() <= expected.abs().max(1.0) * 1e-12,
                    "{f}: {actual}"
                );
            }
            _ => panic!("Unknown fixture"),
        }
    }
}
