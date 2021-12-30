use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct CalculatorState {
    left: Option<f64>,
    right: Option<f64>,
    result: Option<f64>,
    current_operation: Operation,
    old_operation: Operation,
    string: String,
}

impl CalculatorState {
    pub fn new() -> Self {
        CalculatorState {
            left: None,
            right: None,
            result: None,
            current_operation: Operation::None,
            old_operation: Operation::None,
            string: "".to_string(),
        }
    }

    pub fn get_upper_display(&self) -> String {
        let mut res = String::from("");
        if let Some(n) = self.left {
            res.push_str(&n.to_string())
        }
        if self.current_operation == Operation::Eq {
            match self.old_operation {
                Operation::None => res.push_str(""),
                Operation::Add => res.push_str(" + "),
                Operation::Sub => res.push_str(" - "),
                Operation::Mul => res.push_str(" × "),
                Operation::Div => res.push_str(" / "),
                Operation::Eq => (),
            }
            if let Some(n) = self.right {
                res.push_str(&n.to_string())
            }
        }

        match self.current_operation {
            Operation::None => res.push_str(""),
            Operation::Add => res.push_str(" + "),
            Operation::Sub => res.push_str(" - "),
            Operation::Mul => res.push_str(" × "),
            Operation::Div => res.push_str(" / "),
            Operation::Eq => (),
        }

        if self.current_operation == Operation::Eq {
            res.push_str(" = ")
        }

        res
    }

    pub fn get_display(&self) -> String {
        if !self.string.is_empty() {
            if self.string == "-" {
                return "-0".to_string()
            }
            return self.string.to_string();
        } else if self.current_operation == Operation::Eq {
            if let Some(val) = self.result {
                return val.to_string();
            }
        }
        "0".to_string()
    }

    fn calculate(&mut self) {
        match (self.left, self.right) {
            (Some(left), Some(right)) => {
                self.result = match &self.current_operation {
                    Operation::None => Some(left),
                    Operation::Eq => Some(left),
                    Operation::Add => Some(left + right),
                    Operation::Sub => Some(left - right),
                    Operation::Mul => Some(left * right),
                    Operation::Div => Some(left / right),
                };
            }
            _ => println!("No calculation"),
        }
    }

    pub fn append(&mut self, val: i64) {
        if self.current_operation == Operation::Eq {
            self.left = None;
            self.right = None;
            self.result = None;
            self.current_operation = Operation::None;
            self.old_operation = Operation::None;
            self.string = "".to_string();
        }
        self.string.push_str(&i64::to_string(&val));
        println!("{:?}", self);
    }

    pub fn push_separator(&mut self) {
        if self.current_operation == Operation::Eq {
            self.left = None;
            self.right = None;
            self.result = None;
            self.current_operation = Operation::None;
            self.old_operation = Operation::None;
            self.string = "".to_string();
        }

        if self.string.is_empty() {
            self.string.push_str("0,");
        } else {
            if !self.string.contains(",") {
                self.string.push_str(",");
            }
        }
        println!("{:?}", self);
    }

    pub fn negate_current(&mut self) {
        if self.string.starts_with("-") {
            self.string.remove(0);
        } else {
            self.string.insert(0, '-');
        }
    }

    pub fn percent_to_decimal(&mut self) {
        let res = f64::from_str(&self.string.replace(",", ".")).unwrap_or(0.0);

        self.string = (res / 100.0).to_string()
    }

    pub fn clear(&mut self) {
        self.string = "".to_string();
    }

    pub fn clear_all(&mut self) {
        self.left = None;
        self.right = None;
        self.result = None;
        self.current_operation = Operation::None;
        self.old_operation = Operation::None;
        self.string = "".to_string();
    }

    pub fn pop_char(&mut self) {
        self.string.pop();
        ()
    }

    pub fn set_operation(&mut self, op: Operation) {
        if self.current_operation == Operation::Eq {
            self.left = self.result;
            self.right = None;
        }

        if let None = self.left {
            if !self.string.is_empty() {
                self.left = Some(f64::from_str(&self.string.replace(",", ".")).unwrap());
                self.string = String::new()
            }
        } else {
            if !self.string.is_empty() {
                self.right = Some(f64::from_str(&self.string.replace(",", ".")).unwrap());
                self.calculate();
                self.string = String::new()
            }
        }
        self.old_operation = self.current_operation;
        self.current_operation = op;
        println!("{:?}", &self);
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Operation {
    None,
    Eq,
    Add,
    Sub,
    Mul,
    Div,
}

impl Operation {
    pub fn to_symbol(&self) -> &'static str {
        match self {
            Operation::None => "",
            Operation::Eq => "=",
            Operation::Add => "+",
            Operation::Sub => "-",
            Operation::Mul => "×",
            Operation::Div => "/",
        }
    }
}
