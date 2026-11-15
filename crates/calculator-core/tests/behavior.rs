use calculator_core::{Action, Calculator, Mode, format_number, format_time, parse_time};
fn run(mode: Mode, keys: &str) -> Calculator {
    let mut c = Calculator {
        mode,
        ..Default::default()
    };
    for key in keys.chars() {
        c.apply(Action::from_char(key).unwrap());
    }
    c
}
#[test]
fn shorthand_and_seconds() {
    assert_eq!(parse_time("145"), 6300.0);
    assert_eq!(parse_time("1::15"), 3615.0);
    assert_eq!(format_time(-2700.0), "−0:45");
}
#[test]
fn negative_time() {
    assert_eq!(run(Mode::Time, "45-130=").current_text(), "−0:45");
}
#[test]
fn scalar_time() {
    assert_eq!(run(Mode::Time, "1:30*2=").current_text(), "3:00");
}
#[test]
fn precedence_and_groups() {
    assert_eq!(run(Mode::Number, "2+3*4=").current_text(), "14");
    assert_eq!(run(Mode::Number, "(2+3)*4=").current_text(), "20");
}
#[test]
fn intermediate_sum() {
    assert_eq!(run(Mode::Number, "2+3=*4=").current_text(), "20");
}
#[test]
fn division_error_and_recovery() {
    let mut c = run(Mode::Number, "1/0=");
    assert_eq!(c.current_text(), "Nicht definiert");
    c.apply(Action::Digit('7'));
    assert_eq!(c.current_text(), "7");
}
#[test]
fn undo_result() {
    let mut c = run(Mode::Number, "2+3=");
    c.apply(Action::Undo);
    assert_eq!(c.state().steps.len(), 2);
    assert!(!c.state().result);
}
#[test]
fn modes_are_independent() {
    let mut c = run(Mode::Time, "145");
    c.apply(Action::Mode(Mode::Number));
    c.apply(Action::Digit('7'));
    c.apply(Action::Mode(Mode::Time));
    assert_eq!(c.current_text(), "1:45");
}
#[test]
fn capacity_preserves_state() {
    let mut c = run(Mode::Number, &"1".repeat(64));
    assert!(!c.apply(Action::Digit('1')));
    assert_eq!(c.state().entry.len(), 64);
    let mut c = run(Mode::Number, "1+");
    for _ in 0..999 {
        c.apply(Action::Digit('1'));
        c.apply(Action::Operator('+'));
    }
    assert_eq!(c.state().steps.len(), 1000);
    c.apply(Action::Digit('2'));
    let before = c.state().clone();
    assert!(!c.apply(Action::Equals));
    assert_eq!(*c.state(), before);
}
#[test]
fn js_rounding() {
    assert_eq!(format_time(-0.5), "−0:00");
    assert_eq!(format_number(-1.2345675), "-1.234567");
}
#[test]
fn restored_state_is_validated() {
    let mut c = run(Mode::Number, "2+3=");
    let encoded = serde_json::to_string(&c).unwrap();
    let mut restored: Calculator = serde_json::from_str(&encoded).unwrap();
    restored.validate();
    assert_eq!(restored.current_text(), "5");
    c.mode = Mode::Time;
}

#[test]
fn generated_boundary_results_survive_reload() {
    for (mode, keys) in [
        (Mode::Number, "1000000000000000000000="),
        (Mode::Time, "999999999*999999999="),
    ] {
        let c = run(mode, keys);
        let json = serde_json::to_string(&c).unwrap();
        let mut restored: Calculator = serde_json::from_str(&json).unwrap();
        restored.validate();
        assert_eq!(restored.state(), c.state(), "{keys}");
    }
}
