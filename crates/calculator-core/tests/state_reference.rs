use calculator_core::{Action, Calculator, Mode};
#[test]
fn javascript_state_transitions() {
    let fixtures: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../../../tests/reference/states.json")).unwrap();
    for f in fixtures {
        let mut c = Calculator {
            mode: if f["mode"] == "time" {
                Mode::Time
            } else {
                Mode::Number
            },
            ..Default::default()
        };
        for key in f["keys"].as_str().unwrap().chars() {
            let action = match key {
                'b' => Action::Backspace,
                'u' => Action::Undo,
                'c' => Action::ClearEntry,
                'C' => Action::Clear,
                _ => Action::from_char(key).unwrap(),
            };
            c.apply(action);
        }
        assert_eq!(c.current_text(), f["current"].as_str().unwrap(), "{f}");
        assert_eq!(
            c.state().steps.len(),
            f["rows"].as_u64().unwrap() as usize,
            "{f}"
        );
        assert_eq!(c.state().result, f["result"].as_bool().unwrap(), "{f}");
        assert_eq!(c.state().error, f["error"].as_bool().unwrap(), "{f}");
        assert_eq!(
            c.state().pending.map(|c| c.to_string()),
            f["pending"].as_str().map(str::to_owned),
            "{f}"
        );
        if let Some(expected) = f["total"].as_f64() {
            assert!(
                (c.total() - expected).abs() <= expected.abs().max(1.0) * 1e-12,
                "{f}: {}",
                c.total()
            );
        } else {
            assert!(!c.total().is_finite(), "{f}");
        }
    }
}
