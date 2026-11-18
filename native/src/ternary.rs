//! Exact balanced ternary SAT domain and independent DIMACS source parser.
//! Minus=False, Zero=Unknown, Plus=True; no floating-point representation.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i8)]
pub enum Trit {
    Minus = -1,
    Zero = 0,
    Plus = 1,
}

impl Trit {
    pub fn from_literal(lit: i32) -> Self {
        if lit > 0 {
            Self::Plus
        } else {
            Self::Minus
        }
    }

    pub fn negate(self) -> Self {
        match self {
            Self::Minus => Self::Plus,
            Self::Zero => Self::Zero,
            Self::Plus => Self::Minus,
        }
    }

    pub fn symbol(self) -> char {
        match self {
            Self::Minus => '-',
            Self::Zero => '0',
            Self::Plus => '+',
        }
    }

    pub fn from_symbol(symbol: char) -> Option<Self> {
        match symbol {
            '-' => Some(Self::Minus),
            '0' => Some(Self::Zero),
            '+' => Some(Self::Plus),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Cnf {
    pub variables: usize,
    pub clauses: Vec<Vec<i32>>,
}

impl Cnf {
    pub fn parse(input: &str) -> Result<Self, String> {
        let mut declared: Option<(usize, usize)> = None;
        let mut clauses: Vec<Vec<i32>> = Vec::new();
        let mut clause: Vec<i32> = Vec::new();
        for (line_no, line) in input.lines().enumerate() {
            let mut words = line.split_ascii_whitespace();
            let Some(first) = words.next() else { continue };
            if first == "c" {
                continue;
            }
            if first == "p" {
                if declared.is_some() || !clauses.is_empty() || !clause.is_empty() {
                    return Err(format!("duplicate or misplaced header at {}", line_no + 1));
                }
                if words.next() != Some("cnf") {
                    return Err("only DIMACS cnf is accepted".to_owned());
                }
                let vars = words.next().ok_or("missing variable count")?
                    .parse::<usize>().map_err(|_| "invalid variable count")?;
                let total = words.next().ok_or("missing clause count")?
                    .parse::<usize>().map_err(|_| "invalid clause count")?;
                if words.next().is_some() || vars == 0 || vars > i32::MAX as usize {
                    return Err("invalid DIMACS header".to_owned());
                }
                declared = Some((vars, total));
                continue;
            }
            let (vars, expected) = declared.ok_or("DIMACS header absent")?;
            for word in std::iter::once(first).chain(words) {
                let value = word.parse::<i32>()
                    .map_err(|_| format!("invalid literal at {}", line_no + 1))?;
                if value == 0 {
                    clauses.push(std::mem::take(&mut clause));
                    if clauses.len() > expected {
                        return Err("more clauses than header declares".to_owned());
                    }
                } else {
                    let variable = value.checked_abs()
                        .ok_or("out-of-range signed literal")? as usize;
                    if variable == 0 || variable > vars {
                        return Err("literal outside header range".to_owned());
                    }
                    clause.push(value);
                }
            }
        }
        let (vars, expected) = declared.ok_or("missing DIMACS header")?;
        if !clause.is_empty() || clauses.len() != expected {
            return Err("unterminated clause or mismatched clause count".to_owned());
        }
        Ok(Self { variables: vars, clauses })
    }

    pub fn validate_model(&self, model: &[Trit]) -> bool {
        if model.len() != self.variables {
            return false;
        }
        self.clauses.iter().all(|clause| {
            clause.iter().any(|&lit| {
                let state = model[lit.unsigned_abs() as usize - 1];
                state != Trit::Zero && state == Trit::from_literal(lit)
            })
        })
    }

    pub fn valid_complete_model(&self, model: &[Trit]) -> bool {
        model.iter().all(|x| *x != Trit::Zero) && self.validate_model(model)
    }
}

pub fn model_to_text(model: &[Trit]) -> String {
    model.iter().map(|state| state.symbol()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn balanced_trit_closure() {
        for state in [Trit::Minus, Trit::Zero, Trit::Plus] {
            assert_eq!(state.negate().negate(), state);
            assert_eq!(Trit::from_symbol(state.symbol()), Some(state));
        }
        assert_eq!(Trit::Zero.negate(), Trit::Zero);
    }

    #[test]
    fn strict_dimacs_and_signed_literal_models() {
        let formula = Cnf::parse("c sample\np cnf 3 2\n1 -2\n 0\n2 3 0\n")
            .expect("source");
        assert_eq!(formula.clauses, vec![vec![1, -2], vec![2, 3]]);
        assert!(formula.valid_complete_model(
            &[Trit::Plus, Trit::Plus, Trit::Minus]));
        assert!(!formula.validate_model(
            &[Trit::Minus, Trit::Minus, Trit::Minus]));
        for bad in ["p cnf 2 1\n3 0", "p cnf 1 1\n1",
                    "p cnf 1 2\n1 0", "p cnf 1 1\n1 0 1 0",
                    "p cnf 1 1\n-2147483648 0"] {
            assert!(Cnf::parse(bad).is_err(), "{bad}");
        }
        assert_eq!(Cnf::parse("p cnf 1 1\n0").expect("empty").clauses,
                   vec![Vec::<i32>::new()]);
    }
}
