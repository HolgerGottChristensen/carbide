use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct CalculatorState {
    left: Option<f64>,
    right: Option<f64>,
    current_operation: Operation,
    string: String
}

impl CalculatorState {
    pub fn new() -> Self {
        CalculatorState {
            left: None,
            right: None,
            current_operation: Operation::None,
            string: "".to_string()
        }
    }

    fn calculate(&mut self) {
        match (self.left, self.right) {
            (Some(left), Some(right)) => {
                self.left = match &self.current_operation {
                    Operation::None => Some(left),
                    Operation::Add => Some (left + right),
                    Operation::Sub => Some(left - right),
                    Operation::Mul => Some(left * right),
                    Operation::Div => Some(left / right),
                    Operation::Mod => Some(left % right),
                };
                self.right = None
            }
            _ => println!("No calculation")
        }

    }

    pub fn append(&mut self, val: i64) {
        self.string.push_str(&i64::to_string(&val));
        println!("{:?}", self);
    }

    pub fn set_operation(&mut self, op: Operation) {

        if let None = self.left {
            self.left = Some(f64::from_str(&self.string).unwrap());
            self.string = String::new()
        } else {
            if !self.string.is_empty() {
                self.right = Some(f64::from_str(&self.string).unwrap());
                self.calculate();
                self.string = String::new()
            }

        }
        self.current_operation = op;
        println!("{:?}", &self);
    }
}


#[derive(Clone, Debug)]
pub enum Operation {
    None,
    Add,
    Sub,
    Mul,
    Div,
    Mod
}