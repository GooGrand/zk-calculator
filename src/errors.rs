use std::fmt;

use halo2_proofs::{dev::VerifyFailure, plonk::Error};

#[derive(Debug)]
pub enum ParserError {
    InvalidOperator,
    InvalidOperand,
    TooManyInputs,
    NotEnoughInputs
}

#[derive(Debug)]
pub enum CircuitError {
    ProverError(Error),
    VerifierError(Vec<VerifyFailure>),
    NoOperation
}