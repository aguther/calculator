//! Platform-independent time and number calculator.
//!
//! The history is an expression tape rather than a stream of evaluated values.
//! [`evaluate`] applies operator precedence after the most recent subtotal.
//! Time operands are stored as seconds; an unseparated operand after multiplication
//! or division is a dimensionless scalar. Each [`Mode`] retains an independent
//! [`ModeState`]. Serialized field names are part of the persisted data contract.
//!
//! # Example
//!
//! ```
//! use calculator_core::{Action, Calculator};
//!
//! let mut calculator = Calculator::default();
//! for character in "130+45=".chars() {
//!     calculator.apply(Action::from_char(character).unwrap());
//! }
//! assert_eq!(calculator.current_text(), "2:15");
//! ```
#![warn(missing_docs)]
use serde::{Deserialize, Serialize};

/// Maximum number of characters in a single editable entry.
pub const ENTRY_LIMIT: usize = 64;
/// Maximum number of committed tape steps in either calculator mode.
pub const HISTORY_LIMIT: usize = 1000;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
/// Interpretation of numeric input, with independent saved state per mode.
pub enum Mode {
    #[default]
    /// Interpret entries as durations.
    Time,
    /// Interpret entries as decimal numbers.
    Number,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// A serialized expression-tape item. Values use seconds in time mode.
pub enum Step {
    /// A committed operand and the operator preceding it.
    Value {
        /// Preceding arithmetic operator, if any.
        op: Option<char>,
        /// Operand in seconds or numeric units.
        value: f64,
        /// Whether this operand is a dimensionless multiplier or divisor.
        scalar: bool,
    },
    /// Start a group, optionally preceded by an operator.
    Open(Option<char>),
    /// End a group.
    Close,
    /// A completed subtotal that starts a new tape expression.
    Sum(f64),
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
/// Editable entry, expression tape and flags for one calculator mode.
///
/// Field names are stable persistence keys. Do not rename them without a
/// migration or an explicit Serde rename preserving the existing representation.
pub struct ModeState {
    /// Raw editable ASCII entry, before committing it to the tape.
    pub entry: String,
    /// Committed values, operators, groups and subtotal boundaries.
    pub steps: Vec<Step>,
    /// Operator waiting for the next operand.
    pub pending: Option<char>,
    /// Whether the entry displays a generated subtotal rather than fresh input.
    pub result: bool,
    /// Whether evaluation failed and input must begin a new expression.
    pub error: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
/// Calculator state machine shared by native and WebAssembly frontends.
pub struct Calculator {
    /// Currently selected interpretation of input.
    pub mode: Mode,
    /// Preserved time-calculator state.
    pub time: ModeState,
    /// Preserved number-calculator state.
    pub number: ModeState,
    #[serde(skip)]
    /// Transient English capacity notice; excluded from serialization.
    pub notice: String,
}
impl Default for Calculator {
    fn default() -> Self {
        Self {
            mode: Mode::Time,
            time: ModeState::default(),
            number: ModeState::default(),
            notice: String::new(),
        }
    }
}
#[derive(Clone, Copy, Debug)]
/// A single user intent, independent of mouse, touch or keyboard events.
pub enum Action {
    /// Append an ASCII digit.
    Digit(char),
    /// Append the active mode separator.
    Separator,
    /// Select one of `+`, `-`, `*` or `/`.
    Operator(char),
    /// Evaluate and append a subtotal.
    Equals,
    /// Start a parenthesized group.
    Open,
    /// Close the innermost group.
    Close,
    /// Delete the last editable character.
    Backspace,
    /// Clear the active mode entirely.
    Clear,
    /// Clear only the current editable entry.
    ClearEntry,
    /// Reverse the latest tape action or pending edit.
    Undo,
    /// Switch modes without losing either calculation.
    Mode(Mode),
}
impl Action {
    /// Maps a supported input character to an action; other characters are ignored.
    pub fn from_char(c: char) -> Option<Self> {
        Some(match c {
            '0'..='9' => Self::Digit(c),
            ':' | '.' | ',' => Self::Separator,
            '+' | '-' | '*' | '/' => Self::Operator(c),
            '=' | '\n' => Self::Equals,
            '(' => Self::Open,
            ')' => Self::Close,
            _ => return None,
        })
    }
}
fn round_like_javascript(value: f64) -> f64 {
    let floor = value.floor();
    if value - floor >= 0.5 {
        floor + 1.0
    } else {
        floor
    }
}
fn group_digit_runs(text: &str) -> String {
    let mut grouped = String::new();
    let characters: Vec<_> = text.chars().collect();
    let mut index = 0;
    while index < characters.len() {
        if !characters[index].is_ascii_digit() {
            grouped.push(characters[index]);
            index += 1;
            continue;
        }
        let start = index;
        while index < characters.len() && characters[index].is_ascii_digit() {
            index += 1;
        }
        for (offset, character) in characters[start..index].iter().enumerate() {
            if offset > 0 && (index - start - offset).is_multiple_of(3) {
                grouped.push(' ');
            }
            grouped.push(*character);
        }
    }
    grouped
}
/// Converts normalized ASCII `HHMM`, `HH:MM` or `HH:MM:SS` input to seconds.
///
/// Empty or missing components are zero. Callers must supply ASCII entries,
/// as produced by [`Action`] or sanitized by [`Calculator::validate`].
pub fn parse_time(entry: &str) -> f64 {
    let sign = if entry.starts_with('-') { -1.0 } else { 1.0 };
    let unsigned_entry = entry.trim_start_matches('-');
    let parse_component = |component: &str| component.parse::<f64>().unwrap_or(0.0);
    if unsigned_entry.contains(':') {
        let parts: Vec<_> = unsigned_entry.split(':').collect();
        sign * (parse_component(parts[0]) * 3600.0
            + parse_component(parts.get(1).unwrap_or(&"")) * 60.0
            + parse_component(parts.get(2).unwrap_or(&"")))
    } else {
        let split = unsigned_entry.len().saturating_sub(2);
        sign * (parse_component(&unsigned_entry[..split]) * 3600.0
            + parse_component(&unsigned_entry[split..]) * 60.0)
    }
}
/// Formats seconds as grouped hours and two-digit minutes, adding nonzero seconds.
/// Non-finite values are displayed as an em dash. Rounding matches JavaScript.
pub fn format_time(value: f64) -> String {
    if !value.is_finite() {
        return "—".into();
    }
    let sign = if value < 0.0 { "−" } else { "" };
    let total = round_like_javascript(value).abs();
    let hours = (total / 3600.0).floor();
    let minutes = ((total % 3600.0) / 60.0).floor();
    let seconds = total % 60.0;
    let mut formatted_time = format!(
        "{sign}{}:{minutes:02.0}",
        group_digit_runs(ryu_js::Buffer::new().format(hours))
    );
    if seconds != 0.0 {
        formatted_time.push_str(&format!(":{seconds:02.0}"));
    }
    formatted_time
}
/// Rounds to six decimal places and groups digit runs with spaces.
/// Non-finite values are displayed as an em dash and negative zero as `0`.
pub fn format_number(value: f64) -> String {
    if !value.is_finite() {
        return "—".into();
    }
    let rounded_value = round_like_javascript(value * 1e6) / 1e6;
    if rounded_value == 0.0 {
        return "0".into();
    }
    let formatted_number = ryu_js::Buffer::new().format(rounded_value).to_string();
    let (integer_part, fractional_part) = formatted_number
        .split_once('.')
        .unwrap_or((&formatted_number, ""));
    let mut formatted_output = group_digit_runs(integer_part);
    if !fractional_part.is_empty() {
        formatted_output.push('.');
        formatted_output.push_str(fractional_part);
    }
    formatted_output
}
fn entry_from_value(value: f64, mode: Mode) -> String {
    if mode == Mode::Time {
        format_time(value).replace('−', "-").replace(' ', "")
    } else {
        format_number(value).replace(' ', "")
    }
}
fn format_time_entry(entry: &str) -> String {
    let sign = if entry.starts_with('-') { "−" } else { "" };
    let unsigned_entry = entry.trim_start_matches('-');
    let parts: Vec<_> = unsigned_entry.split(':').collect();
    if parts.len() == 1 {
        let split = unsigned_entry.len().saturating_sub(2);
        let hours = unsigned_entry[..split].parse::<u64>().unwrap_or(0);
        let minutes = unsigned_entry[split..].parse::<u32>().unwrap_or(0);
        return format!(
            "{sign}{}:{minutes:02}",
            group_digit_runs(&hours.to_string())
        );
    }
    let hours = parts[0].parse::<u64>().unwrap_or(0);
    let mut formatted_time = format!("{sign}{}", group_digit_runs(&hours.to_string()));
    for part in &parts[1..] {
        formatted_time.push(':');
        formatted_time.push_str(&format!("{:02}", part.parse::<u32>().unwrap_or(0)));
    }
    formatted_time
}
fn precedence(op: char) -> u8 {
    if op == '*' || op == '/' { 2 } else { 1 }
}
fn apply_operator(values: &mut Vec<f64>, operator: char) -> bool {
    let Some(right_operand) = values.pop() else {
        return false;
    };
    let Some(left_operand) = values.pop() else {
        return false;
    };
    values.push(match operator {
        '+' => left_operand + right_operand,
        '-' => left_operand - right_operand,
        '*' => left_operand * right_operand,
        '/' if right_operand != 0.0 => left_operand / right_operand,
        _ => f64::NAN,
    });
    true
}
/// Evaluates the expression after the most recent subtotal using operator stacks.
///
/// A leading operator continues from the preceding subtotal. Invalid expressions,
/// unmatched parentheses and division by zero produce [`f64::NAN`].
pub fn evaluate(steps: &[Step]) -> f64 {
    let last_sum_index = steps.iter().rposition(|step| matches!(step, Step::Sum(_)));
    let previous_sum = last_sum_index.and_then(|index| {
        if let Step::Sum(subtotal) = steps[index] {
            Some(subtotal)
        } else {
            None
        }
    });
    let mut operands = Vec::new();
    let mut operators = Vec::new();
    let mut has_expression_tokens = false;
    let push_operator = |op: char, operands: &mut Vec<f64>, operators: &mut Vec<char>| {
        while let Some(&stack_operator) = operators.last() {
            if stack_operator == '(' || precedence(stack_operator) < precedence(op) {
                break;
            }
            operators.pop();
            if !apply_operator(operands, stack_operator) {
                return false;
            }
        }
        operators.push(op);
        true
    };
    for step in &steps[last_sum_index.map_or(0, |index| index + 1)..] {
        match *step {
            Step::Value { op, value, .. } => {
                if !has_expression_tokens {
                    if let (Some(subtotal), Some(op)) = (previous_sum, op) {
                        operands.push(subtotal);
                        if !push_operator(op, &mut operands, &mut operators) {
                            return f64::NAN;
                        }
                    }
                } else if let Some(op) = op
                    && !push_operator(op, &mut operands, &mut operators)
                {
                    return f64::NAN;
                }
                operands.push(value);
                has_expression_tokens = true;
            }
            Step::Open(op) => {
                if let Some(op) = op
                    && !push_operator(op, &mut operands, &mut operators)
                {
                    return f64::NAN;
                }
                operators.push('(');
                has_expression_tokens = true;
            }
            Step::Close => {
                while operators.last().is_some_and(|op| *op != '(') {
                    let op = operators.pop().unwrap();
                    if !apply_operator(&mut operands, op) {
                        return f64::NAN;
                    }
                }
                if operators.pop() != Some('(') {
                    return f64::NAN;
                }
                has_expression_tokens = true;
            }
            Step::Sum(_) => {}
        }
    }
    if !has_expression_tokens {
        return previous_sum.unwrap_or(0.0);
    }
    while let Some(op) = operators.pop() {
        if op == '(' || !apply_operator(&mut operands, op) {
            return f64::NAN;
        }
    }
    if operands.len() == 1 {
        operands[0]
    } else {
        f64::NAN
    }
}
impl ModeState {
    fn active_expression(&self) -> &[Step] {
        let start = self
            .steps
            .iter()
            .rposition(|s| matches!(s, Step::Sum(_)))
            .map_or(0, |i| i + 1);
        &self.steps[start..]
    }
    fn last_expression_step(&self) -> Option<&Step> {
        self.active_expression().last()
    }
    /// Returns the number of unmatched opening parentheses in the active expression.
    pub fn depth(&self) -> i32 {
        self.active_expression()
            .iter()
            .map(|s| match s {
                Step::Open(_) => 1,
                Step::Close => -1,
                _ => 0,
            })
            .sum()
    }
    fn next_operator(&self) -> Option<char> {
        match self.last_expression_step() {
            None => self.pending,
            Some(Step::Open(_)) => None,
            _ => Some(self.pending.unwrap_or('+')),
        }
    }
    fn can_start_operand(&self) -> bool {
        self.last_expression_step().is_none()
            || self.pending.is_some()
            || matches!(self.last_expression_step(), Some(Step::Open(_)))
    }
    fn can_finish_expression(&self) -> bool {
        !self.error
            && !self.result
            && (!self.entry.is_empty()
                || matches!(
                    self.last_expression_step(),
                    Some(Step::Value { .. } | Step::Close)
                ))
    }
    /// Commit an entry, treating unseparated time multipliers/divisors as scalars.
    fn commit_entry(&mut self, mode: Mode, override_value: Option<f64>) {
        let op = self.next_operator();
        let scalar = mode == Mode::Time
            && matches!(op, Some('*' | '/'))
            && !self.entry.is_empty()
            && !self.entry.contains(':');
        let value = override_value.unwrap_or_else(|| {
            if mode == Mode::Time && !scalar {
                parse_time(&self.entry)
            } else {
                self.entry.parse().unwrap_or(0.0)
            }
        });
        self.steps.push(Step::Value { op, value, scalar });
        self.entry.clear();
        self.pending = None;
    }
}
impl Calculator {
    /// Borrows the state of the currently selected mode.
    pub fn state(&self) -> &ModeState {
        if self.mode == Mode::Time {
            &self.time
        } else {
            &self.number
        }
    }
    fn state_mut(&mut self) -> &mut ModeState {
        if self.mode == Mode::Time {
            &mut self.time
        } else {
            &mut self.number
        }
    }
    /// Evaluates the active tape without changing the editable entry.
    pub fn total(&self) -> f64 {
        evaluate(&self.state().steps)
    }
    /// Formats a tape value according to the active mode and scalar flag.
    pub fn label(&self, value: f64, scalar: bool) -> String {
        if self.mode == Mode::Time && !scalar {
            format_time(value)
        } else {
            format_number(value)
        }
    }
    /// Formats the current entry or canonical English error for the frontend.
    pub fn current_text(&self) -> String {
        let state = self.state();
        if state.error {
            return "Undefined".into();
        }
        if self.mode == Mode::Time {
            if matches!(state.next_operator(), Some('*' | '/'))
                && !state.entry.is_empty()
                && !state.entry.contains(':')
            {
                return format_number(state.entry.parse().unwrap_or(0.0));
            }
            return format_time_entry(&state.entry);
        }
        let (integer_part, fractional_part) =
            state.entry.split_once('.').unwrap_or((&state.entry, ""));
        let integer_prefix: String = integer_part
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '-')
            .collect();
        let value = integer_prefix.parse::<f64>().unwrap_or(0.0);
        let mut display_text = group_digit_runs(ryu_js::Buffer::new().format(value));
        if state.entry.contains('.') {
            display_text.push('.');
            display_text.push_str(fractional_part);
        }
        display_text
    }
    /// Reports whether the active expression can be evaluated with all groups closed.
    pub fn can_equal(&self) -> bool {
        self.state().can_finish_expression() && self.state().depth() == 0
    }
    /// Reports whether a complete operand can close an outstanding parenthesis.
    pub fn can_close(&self) -> bool {
        self.state().can_finish_expression() && self.state().depth() > 0
    }
    /// Applies an action transactionally and reports whether it was accepted.
    ///
    /// Rejected edits restore the previous state. Capacity failures also populate
    /// [`Self::notice`]; numerical failures set the active mode error flag.
    pub fn apply(&mut self, action: Action) -> bool {
        self.notice.clear();
        if let Action::Mode(mode) = action {
            self.mode = mode;
            return true;
        }
        // Apply transactionally: rejected input never partially changes the calculation.
        let previous_state = self.state().clone();
        let accepted = self.apply_to_active_state(action);
        if self.state().entry.len() > ENTRY_LIMIT || self.state().steps.len() > HISTORY_LIMIT {
            *self.state_mut() = previous_state;
            self.notice = "Entry or history limit reached. Clear an entry or undo a step.".into();
            return false;
        }
        if !accepted {
            *self.state_mut() = previous_state;
            return false;
        }
        // Avoid persisting infinities or invalid floating-point values after overflow.
        let state = self.state_mut();
        if state
            .steps
            .iter()
            .any(|step| matches!(step,Step::Value{value,..}|Step::Sum(value) if !value.is_finite()))
        {
            *state = previous_state;
            state.error = true;
            state.entry.clear();
            state.pending = None;
            state.result = false;
        }
        accepted
    }
    fn apply_to_active_state(&mut self, action: Action) -> bool {
        let mode = self.mode;
        if matches!(action, Action::Equals) && !self.can_equal() {
            return false;
        }
        if matches!(action, Action::Close) && !self.can_close() {
            return false;
        }
        let state = self.state_mut();
        match action {
            Action::Digit(c) => {
                if !c.is_ascii_digit() {
                    return false;
                }
                if state.error || state.result {
                    *state = ModeState::default();
                }
                if !state.can_start_operand() {
                    return false;
                }
                if mode == Mode::Time {
                    let unsigned = state.entry.trim_start_matches('-');
                    let parts: Vec<_> = unsigned.split(':').collect();
                    if (parts.len() > 1 && parts.last().unwrap().len() >= 2)
                        || (parts.len() == 1 && unsigned.len() >= 9)
                    {
                        return false;
                    }
                    state.entry.push(c);
                } else if state.entry == "0" {
                    state.entry = c.to_string();
                } else {
                    state.entry.push(c);
                }
            }
            Action::Separator => {
                if state.error || state.result {
                    *state = ModeState::default();
                }
                if !state.can_start_operand() {
                    return false;
                }
                if mode == Mode::Time {
                    if state.entry.matches(':').count() >= 2 {
                        return false;
                    }
                    state.entry.push(':');
                } else {
                    if state.entry.contains('.') {
                        return false;
                    }
                    if state.entry.is_empty() {
                        state.entry.push('0');
                    }
                    state.entry.push('.');
                }
            }
            Action::Operator(op) => {
                if !matches!(op, '+' | '-' | '*' | '/') || state.error {
                    return false;
                }
                if state.result {
                    state.entry.clear();
                    state.result = false;
                    state.pending = Some(op);
                    return true;
                }
                if !state.entry.is_empty() {
                    state.commit_entry(mode, None);
                }
                match state.last_expression_step() {
                    None => state.commit_entry(mode, Some(0.0)),
                    Some(Step::Open(_)) if op == '-' => state.commit_entry(mode, Some(0.0)),
                    Some(Step::Open(_)) => return false,
                    _ => {}
                }
                state.pending = Some(op);
            }
            Action::Equals => {
                if !state.entry.is_empty() {
                    state.commit_entry(mode, None);
                }
                let result = evaluate(&state.steps);
                if !result.is_finite() {
                    state.error = true;
                    state.entry.clear();
                    state.pending = None;
                    state.result = false;
                    return true;
                }
                state.steps.push(Step::Sum(result));
                state.entry = entry_from_value(result, mode);
                state.pending = None;
                state.result = true;
            }
            Action::Open => {
                if state.error {
                    return false;
                }
                if state.result {
                    *state = ModeState::default();
                }
                if !state.entry.is_empty() {
                    state.commit_entry(mode, None);
                }
                let op = if matches!(
                    state.last_expression_step(),
                    Some(Step::Value { .. } | Step::Close)
                ) {
                    Some(state.pending.unwrap_or('*'))
                } else {
                    state.pending
                };
                state.steps.push(Step::Open(op));
                state.pending = None;
            }
            Action::Close => {
                if !state.entry.is_empty() {
                    state.commit_entry(mode, None);
                }
                state.steps.push(Step::Close);
                state.pending = None;
            }
            Action::Clear => *state = ModeState::default(),
            Action::ClearEntry => {
                if state.error || state.result {
                    *state = ModeState::default();
                } else if state.entry.is_empty() {
                    return false;
                } else {
                    state.entry.clear();
                }
            }
            Action::Backspace => {
                if state.error {
                    *state = ModeState::default();
                } else if !state.entry.is_empty() {
                    state.entry.pop();
                } else if state.pending.is_some() {
                    state.pending = None;
                } else if !state.steps.is_empty() {
                    state.steps.pop();
                } else {
                    return false;
                }
            }
            Action::Undo => {
                if state.error {
                    *state = ModeState::default();
                } else if state.result {
                    state.result = false;
                    state.entry.clear();
                    state.pending = None;
                    if matches!(state.steps.last(), Some(Step::Sum(_))) {
                        state.steps.pop();
                    }
                } else if !state.entry.is_empty() {
                    state.entry.clear();
                } else if state.pending.is_some() {
                    state.pending = None;
                } else if !state.steps.is_empty() {
                    state.steps.pop();
                    state.pending = None;
                } else {
                    return false;
                }
            }
            Action::Mode(_) => unreachable!(),
        }
        true
    }
    /// Sanitizes restored data, resetting invalid modes without discarding the other mode.
    pub fn validate(&mut self) {
        for (mode, state) in [
            (Mode::Time, &mut self.time),
            (Mode::Number, &mut self.number),
        ] {
            let entry_is_valid = if mode == Mode::Time {
                let raw = state.entry.trim_start_matches('-');
                let parts: Vec<_> = raw.split(':').collect();
                parts.len() <= 3
                    && parts[0].len() <= 9
                    && parts.iter().all(|v| v.bytes().all(|b| b.is_ascii_digit()))
                    && parts.iter().skip(1).all(|v| v.len() <= 2)
            } else {
                let raw = state.entry.strip_prefix('-').unwrap_or(&state.entry);
                raw.matches('.').count() <= 1
                    && raw.bytes().all(|b| b.is_ascii_digit() || b == b'.')
            };
            let is_valid_operator =
                |op: Option<char>| op.is_none_or(|c| matches!(c, '+' | '-' | '*' | '/'));
            let steps_are_valid = state.steps.len() <= HISTORY_LIMIT
                && state.steps.iter().all(|step| match step {
                    Step::Value { op, value, .. } => is_valid_operator(*op) && value.is_finite(),
                    Step::Sum(v) => v.is_finite(),
                    Step::Open(op) => is_valid_operator(*op),
                    Step::Close => true,
                });
            let has_valid_generated_result = state.result
                && matches!(state.steps.last(), Some(Step::Sum(value)) if value.is_finite() && state.entry == entry_from_value(*value,mode));
            if (!entry_is_valid && !has_valid_generated_result)
                || state.entry.len() > ENTRY_LIMIT
                || !steps_are_valid
                || !is_valid_operator(state.pending)
                || state.depth() < 0
            {
                *state = ModeState::default();
            }
        }
    }
}
