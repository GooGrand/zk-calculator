use std::io;
use halo2_proofs::{circuit::Value, dev::MockProver, pasta::Fp};

use crate::{
    calculator_circuit::CalculatorCircuit,
    errors::{CircuitError, ParserError}
};

trait FromToken<T, E> {
    fn from_token(token: &str) -> Result<T, E>;
}

#[derive(Clone, Copy)]
pub enum Operator {
    Add,
    Sub,
    Mul
}

type Operand = u64;

impl FromToken<Operator, ParserError> for Operator {
    fn from_token(token: &str) -> Result<Operator, ParserError> {
        match token {
            "+" => Ok(Operator::Add),
            "-" => Ok(Operator::Sub),
            "*" => Ok(Operator::Mul),
            _ => Err(ParserError::InvalidOperator)
        }
    }
}

impl FromToken<Operand, ParserError> for Operand {
    fn from_token(token: &str) -> Result<Operand, ParserError> {
        token.parse::<Operand>().map_err(|_| ParserError::InvalidOperand)
    }
}

struct Operation {
    pub a: Operand,
    pub b: Operand,
    pub operator: Operator
}

pub struct ZkCalculator {
    operation: Option<Operation>
}

impl ZkCalculator {
    pub fn new() -> Self {
        Self {
            operation: None
        }
    }

    pub fn run(&mut self) {
        let mut input = String::new();

        println!("\n\n/- ------------------------------ -/");
        println!("/- Enter calculation to perform -/");

        io::stdin().read_line(&mut input).expect("io failed");

        self.parse(input).expect("parse failed");

        let output = self.run_circuit().expect("circuit failed");

        println!("proof generation succeeded!\nresult: {:#?}", output);
    }

    fn parse(&mut self, input: String) -> Result<(), ParserError> {
        let mut tokens = input.split_whitespace();
        let a = match tokens.next() {
            Some(a) => Operand::from_token(a),
            None => Err(ParserError::NotEnoughInputs)
        }?;

        let operator = match tokens.next() {
            Some(op) => Operator::from_token(op),
            None => Err(ParserError::NotEnoughInputs)
        }?;

        let b = match tokens.next() {
            Some(b) => Operand::from_token(b),
            None => Err(ParserError::NotEnoughInputs)
        }?;

        if tokens.next().is_some() {
            return Err(ParserError::TooManyInputs)
        }
        self.operation = Some(Operation { a, b, operator});
        Ok(())
    }

    fn run_circuit(&self) -> Result<Fp, CircuitError> {
        let k = 4;

        let operation = self.operation.as_ref().ok_or(CircuitError::NoOperation)?;

        let operator = operation.operator;

        let a = Fp::from(operation.a);
        let b = Fp::from(operation.b);

        let c = match operator {
            Operator::Add => a + b,
            Operator::Sub => a - b,
            Operator::Mul => a * b,
        };

        let circuit = CalculatorCircuit {
            a: Value::known(a),
            b: Value::known(b),
            operator
        };
        
        let public_inputs = vec![c];

        let prover = MockProver::run(
            k,
            &circuit,
            vec![public_inputs.clone()]
        ).map_err(|e| CircuitError::ProverError(e))?;
        prover.verify().map(|_| c).map_err(|e|CircuitError::VerifierError(e))
    }
}