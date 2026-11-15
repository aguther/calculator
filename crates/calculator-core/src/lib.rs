//! Platform-independent calculator, ported from simple-calculator (MIT).
use serde::{Deserialize, Serialize};

pub const ENTRY_LIMIT: usize = 64;
pub const HISTORY_LIMIT: usize = 1000;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    #[default]
    Time,
    Number,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Step {
    Value {
        op: Option<char>,
        value: f64,
        scalar: bool,
    },
    Open(Option<char>),
    Close,
    Sum(f64),
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ModeState {
    pub entry: String,
    pub steps: Vec<Step>,
    pub pending: Option<char>,
    pub result: bool,
    pub error: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Calculator {
    pub mode: Mode,
    pub time: ModeState,
    pub number: ModeState,
    #[serde(skip)]
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
pub enum Action {
    Digit(char),
    Separator,
    Operator(char),
    Equals,
    Open,
    Close,
    Backspace,
    Clear,
    ClearEntry,
    Undo,
    Mode(Mode),
}
impl Action {
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
fn js_round(n: f64) -> f64 {
    (n + 0.5).floor()
}
fn group(s: &str) -> String {
    let mut out = String::new();
    let chars: Vec<_> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if !chars[i].is_ascii_digit() {
            out.push(chars[i]);
            i += 1;
            continue;
        }
        let start = i;
        while i < chars.len() && chars[i].is_ascii_digit() {
            i += 1;
        }
        for (j, c) in chars[start..i].iter().enumerate() {
            if j > 0 && (i - start - j).is_multiple_of(3) {
                out.push(' ');
            }
            out.push(*c);
        }
    }
    out
}
pub fn parse_time(entry: &str) -> f64 {
    let sign = if entry.starts_with('-') { -1.0 } else { 1.0 };
    let s = entry.trim_start_matches('-');
    let num = |s: &str| s.parse::<f64>().unwrap_or(0.0);
    if s.contains(':') {
        let parts: Vec<_> = s.split(':').collect();
        sign * (num(parts[0]) * 3600.0
            + num(parts.get(1).unwrap_or(&"")) * 60.0
            + num(parts.get(2).unwrap_or(&"")))
    } else {
        let split = s.len().saturating_sub(2);
        sign * (num(&s[..split]) * 3600.0 + num(&s[split..]) * 60.0)
    }
}
pub fn format_time(value: f64) -> String {
    if !value.is_finite() {
        return "—".into();
    }
    let sign = if value < 0.0 { "−" } else { "" };
    let total = js_round(value).abs();
    let hours = (total / 3600.0).floor();
    let minutes = ((total % 3600.0) / 60.0).floor();
    let seconds = total % 60.0;
    let mut out = format!(
        "{sign}{}:{minutes:02.0}",
        group(ryu_js::Buffer::new().format(hours))
    );
    if seconds != 0.0 {
        out.push_str(&format!(":{seconds:02.0}"));
    }
    out
}
pub fn format_number(value: f64) -> String {
    if !value.is_finite() {
        return "—".into();
    }
    let n = js_round(value * 1e6) / 1e6;
    if n == 0.0 {
        return "0".into();
    }
    let s = ryu_js::Buffer::new().format(n).to_string();
    let (int, frac) = s.split_once('.').unwrap_or((&s, ""));
    let mut out = group(int);
    if !frac.is_empty() {
        out.push('.');
        out.push_str(frac);
    }
    out
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
    let s = entry.trim_start_matches('-');
    let parts: Vec<_> = s.split(':').collect();
    if parts.len() == 1 {
        let split = s.len().saturating_sub(2);
        let hours = s[..split].parse::<u64>().unwrap_or(0);
        let minutes = s[split..].parse::<u32>().unwrap_or(0);
        return format!("{sign}{}:{minutes:02}", group(&hours.to_string()));
    }
    let hours = parts[0].parse::<u64>().unwrap_or(0);
    let mut out = format!("{sign}{}", group(&hours.to_string()));
    for part in &parts[1..] {
        out.push(':');
        out.push_str(&format!("{:02}", part.parse::<u32>().unwrap_or(0)));
    }
    out
}
fn precedence(op: char) -> u8 {
    if op == '*' || op == '/' { 2 } else { 1 }
}
fn apply_op(values: &mut Vec<f64>, op: char) -> bool {
    let Some(b) = values.pop() else { return false };
    let Some(a) = values.pop() else { return false };
    values.push(match op {
        '+' => a + b,
        '-' => a - b,
        '*' => a * b,
        '/' if b != 0.0 => a / b,
        _ => f64::NAN,
    });
    true
}
pub fn evaluate(steps: &[Step]) -> f64 {
    let start = steps.iter().rposition(|s| matches!(s, Step::Sum(_)));
    let sum = start.and_then(|i| {
        if let Step::Sum(n) = steps[i] {
            Some(n)
        } else {
            None
        }
    });
    let mut values = Vec::new();
    let mut ops = Vec::new();
    let mut has_tokens = false;
    let feed_op = |op: char, values: &mut Vec<f64>, ops: &mut Vec<char>| {
        while let Some(&top) = ops.last() {
            if top == '(' || precedence(top) < precedence(op) {
                break;
            }
            ops.pop();
            if !apply_op(values, top) {
                return false;
            }
        }
        ops.push(op);
        true
    };
    for step in &steps[start.map_or(0, |i| i + 1)..] {
        match *step {
            Step::Value { op, value, .. } => {
                if !has_tokens {
                    if let (Some(n), Some(op)) = (sum, op) {
                        values.push(n);
                        if !feed_op(op, &mut values, &mut ops) {
                            return f64::NAN;
                        }
                    }
                } else if let Some(op) = op
                    && !feed_op(op, &mut values, &mut ops)
                {
                    return f64::NAN;
                }
                values.push(value);
                has_tokens = true;
            }
            Step::Open(op) => {
                if let Some(op) = op
                    && !feed_op(op, &mut values, &mut ops)
                {
                    return f64::NAN;
                }
                ops.push('(');
                has_tokens = true;
            }
            Step::Close => {
                while ops.last().is_some_and(|op| *op != '(') {
                    let op = ops.pop().unwrap();
                    if !apply_op(&mut values, op) {
                        return f64::NAN;
                    }
                }
                if ops.pop() != Some('(') {
                    return f64::NAN;
                }
                has_tokens = true;
            }
            Step::Sum(_) => {}
        }
    }
    if !has_tokens {
        return sum.unwrap_or(0.0);
    }
    while let Some(op) = ops.pop() {
        if op == '(' || !apply_op(&mut values, op) {
            return f64::NAN;
        }
    }
    if values.len() == 1 {
        values[0]
    } else {
        f64::NAN
    }
}
impl ModeState {
    fn expression(&self) -> &[Step] {
        let start = self
            .steps
            .iter()
            .rposition(|s| matches!(s, Step::Sum(_)))
            .map_or(0, |i| i + 1);
        &self.steps[start..]
    }
    fn last(&self) -> Option<&Step> {
        self.expression().last()
    }
    pub fn depth(&self) -> i32 {
        self.expression()
            .iter()
            .map(|s| match s {
                Step::Open(_) => 1,
                Step::Close => -1,
                _ => 0,
            })
            .sum()
    }
    fn next_op(&self) -> Option<char> {
        match self.last() {
            None => self.pending,
            Some(Step::Open(_)) => None,
            _ => Some(self.pending.unwrap_or('+')),
        }
    }
    fn can_start(&self) -> bool {
        self.last().is_none()
            || self.pending.is_some()
            || matches!(self.last(), Some(Step::Open(_)))
    }
    fn can_finish(&self) -> bool {
        !self.error
            && !self.result
            && (!self.entry.is_empty()
                || matches!(self.last(), Some(Step::Value { .. } | Step::Close)))
    }
    fn commit(&mut self, mode: Mode, override_value: Option<f64>) {
        let op = self.next_op();
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
    pub fn total(&self) -> f64 {
        evaluate(&self.state().steps)
    }
    pub fn label(&self, value: f64, scalar: bool) -> String {
        if self.mode == Mode::Time && !scalar {
            format_time(value)
        } else {
            format_number(value)
        }
    }
    pub fn current_text(&self) -> String {
        let s = self.state();
        if s.error {
            return "Nicht definiert".into();
        }
        if self.mode == Mode::Time {
            if matches!(s.next_op(), Some('*' | '/'))
                && !s.entry.is_empty()
                && !s.entry.contains(':')
            {
                return format_number(s.entry.parse().unwrap_or(0.0));
            }
            return format_time_entry(&s.entry);
        }
        let (int, frac) = s.entry.split_once('.').unwrap_or((&s.entry, ""));
        let prefix: String = int
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '-')
            .collect();
        let value = prefix.parse::<f64>().unwrap_or(0.0);
        let mut out = group(ryu_js::Buffer::new().format(value));
        if s.entry.contains('.') {
            out.push('.');
            out.push_str(frac);
        }
        out
    }
    pub fn can_equal(&self) -> bool {
        self.state().can_finish() && self.state().depth() == 0
    }
    pub fn can_close(&self) -> bool {
        self.state().can_finish() && self.state().depth() > 0
    }
    pub fn apply(&mut self, action: Action) -> bool {
        self.notice.clear();
        if let Action::Mode(mode) = action {
            self.mode = mode;
            return true;
        }
        // Apply transactionally: rejected input never partially changes the calculation.
        let before = self.state().clone();
        let accepted = self.apply_inner(action);
        if self.state().entry.len() > ENTRY_LIMIT || self.state().steps.len() > HISTORY_LIMIT {
            *self.state_mut() = before;
            self.notice =
                "Eingabe- oder Verlaufslimit erreicht. Bitte löschen oder rückgängig machen."
                    .into();
            return false;
        }
        if !accepted {
            *self.state_mut() = before;
            return false;
        }
        // Avoid persisting infinities or invalid floating-point values after overflow.
        let s = self.state_mut();
        if s.steps
            .iter()
            .any(|step| matches!(step,Step::Value{value,..}|Step::Sum(value) if !value.is_finite()))
        {
            *s = before;
            s.error = true;
            s.entry.clear();
            s.pending = None;
            s.result = false;
        }
        accepted
    }
    fn apply_inner(&mut self, action: Action) -> bool {
        let mode = self.mode;
        if matches!(action, Action::Equals) && !self.can_equal() {
            return false;
        }
        if matches!(action, Action::Close) && !self.can_close() {
            return false;
        }
        let s = self.state_mut();
        match action {
            Action::Digit(c) => {
                if !c.is_ascii_digit() {
                    return false;
                }
                if s.error || s.result {
                    *s = ModeState::default();
                }
                if !s.can_start() {
                    return false;
                }
                if mode == Mode::Time {
                    let unsigned = s.entry.trim_start_matches('-');
                    let parts: Vec<_> = unsigned.split(':').collect();
                    if (parts.len() > 1 && parts.last().unwrap().len() >= 2)
                        || (parts.len() == 1 && unsigned.len() >= 9)
                    {
                        return false;
                    }
                    s.entry.push(c);
                } else if s.entry == "0" {
                    s.entry = c.to_string();
                } else {
                    s.entry.push(c);
                }
            }
            Action::Separator => {
                if s.error || s.result {
                    *s = ModeState::default();
                }
                if !s.can_start() {
                    return false;
                }
                if mode == Mode::Time {
                    if s.entry.matches(':').count() >= 2 {
                        return false;
                    }
                    s.entry.push(':');
                } else {
                    if s.entry.contains('.') {
                        return false;
                    }
                    if s.entry.is_empty() {
                        s.entry.push('0');
                    }
                    s.entry.push('.');
                }
            }
            Action::Operator(op) => {
                if !matches!(op, '+' | '-' | '*' | '/') || s.error {
                    return false;
                }
                if s.result {
                    s.entry.clear();
                    s.result = false;
                    s.pending = Some(op);
                    return true;
                }
                if !s.entry.is_empty() {
                    s.commit(mode, None);
                }
                match s.last() {
                    None => s.commit(mode, Some(0.0)),
                    Some(Step::Open(_)) if op == '-' => s.commit(mode, Some(0.0)),
                    Some(Step::Open(_)) => return false,
                    _ => {}
                }
                s.pending = Some(op);
            }
            Action::Equals => {
                if !s.entry.is_empty() {
                    s.commit(mode, None);
                }
                let result = evaluate(&s.steps);
                if !result.is_finite() {
                    s.error = true;
                    s.entry.clear();
                    s.pending = None;
                    s.result = false;
                    return true;
                }
                s.steps.push(Step::Sum(result));
                s.entry = entry_from_value(result, mode);
                s.pending = None;
                s.result = true;
            }
            Action::Open => {
                if s.error {
                    return false;
                }
                if s.result {
                    *s = ModeState::default();
                }
                if !s.entry.is_empty() {
                    s.commit(mode, None);
                }
                let op = if matches!(s.last(), Some(Step::Value { .. } | Step::Close)) {
                    Some(s.pending.unwrap_or('*'))
                } else {
                    s.pending
                };
                s.steps.push(Step::Open(op));
                s.pending = None;
            }
            Action::Close => {
                if !s.entry.is_empty() {
                    s.commit(mode, None);
                }
                s.steps.push(Step::Close);
                s.pending = None;
            }
            Action::Clear => *s = ModeState::default(),
            Action::ClearEntry => {
                if s.error || s.result {
                    *s = ModeState::default();
                } else if s.entry.is_empty() {
                    return false;
                } else {
                    s.entry.clear();
                }
            }
            Action::Backspace => {
                if s.error {
                    *s = ModeState::default();
                } else if !s.entry.is_empty() {
                    s.entry.pop();
                } else if s.pending.is_some() {
                    s.pending = None;
                } else if !s.steps.is_empty() {
                    s.steps.pop();
                } else {
                    return false;
                }
            }
            Action::Undo => {
                if s.error {
                    *s = ModeState::default();
                } else if s.result {
                    s.result = false;
                    s.entry.clear();
                    s.pending = None;
                    if matches!(s.steps.last(), Some(Step::Sum(_))) {
                        s.steps.pop();
                    }
                } else if !s.entry.is_empty() {
                    s.entry.clear();
                } else if s.pending.is_some() {
                    s.pending = None;
                } else if !s.steps.is_empty() {
                    s.steps.pop();
                    s.pending = None;
                } else {
                    return false;
                }
            }
            Action::Mode(_) => unreachable!(),
        }
        true
    }
    pub fn validate(&mut self) {
        for (mode, s) in [
            (Mode::Time, &mut self.time),
            (Mode::Number, &mut self.number),
        ] {
            let entry_ok = if mode == Mode::Time {
                let raw = s.entry.trim_start_matches('-');
                let parts: Vec<_> = raw.split(':').collect();
                parts.len() <= 3
                    && parts[0].len() <= 9
                    && parts.iter().all(|v| v.bytes().all(|b| b.is_ascii_digit()))
                    && parts.iter().skip(1).all(|v| v.len() <= 2)
            } else {
                let raw = s.entry.strip_prefix('-').unwrap_or(&s.entry);
                raw.matches('.').count() <= 1
                    && raw.bytes().all(|b| b.is_ascii_digit() || b == b'.')
            };
            let valid_op = |op: Option<char>| op.is_none_or(|c| matches!(c, '+' | '-' | '*' | '/'));
            let steps_ok = s.steps.len() <= HISTORY_LIMIT
                && s.steps.iter().all(|step| match step {
                    Step::Value { op, value, .. } => valid_op(*op) && value.is_finite(),
                    Step::Sum(v) => v.is_finite(),
                    Step::Open(op) => valid_op(*op),
                    Step::Close => true,
                });
            let generated_result = s.result
                && matches!(s.steps.last(), Some(Step::Sum(value)) if value.is_finite() && s.entry == entry_from_value(*value,mode));
            if (!entry_ok && !generated_result)
                || s.entry.len() > ENTRY_LIMIT
                || !steps_ok
                || !valid_op(s.pending)
                || s.depth() < 0
            {
                *s = ModeState::default();
            }
        }
    }
}
