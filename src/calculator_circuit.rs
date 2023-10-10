use halo2_proofs::{
    arithmetic::FieldExt,
    circuit::{Layouter, SimpleFloorPlanner, Value},
    plonk::{Circuit, Constraints, Error},
};

use crate::{
    chips::{
        add::AddInstructions,
        arithmetic::{ArithmeticChip, ArithmeticConfig, ArithmeticInstructions},
        mul::MulInstructions,
        sub::SubInstructions,
    },
    zk_calculator::Operator,
};

pub struct CalculatorCircuit<F: FieldExt> {
    pub a: Value<F>,
    pub b: Value<F>,
    pub operator: Operator,
}

impl<F: FieldExt> Circuit<F> for CalculatorCircuit<F> {
    type Config = ArithmeticConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self {
            a: Value::default(),
            b: Value::default(),
            operator: self.operator.clone(),
        }
    }

    fn configure(meta: &mut halo2_proofs::plonk::ConstraintSystem<F>) -> Self::Config {
        let a = meta.advice_column();
        let b = meta.advice_column();

        let instance = meta.instance_column();

        ArithmeticChip::configure(meta, a, b, instance)
    }

    fn synthesize(&self, config: Self::Config, mut layouter: impl Layouter<F>) -> Result<(), Error> {
        let arithmetic_chip = ArithmeticChip::<F>::construct(config, ());

        let a = arithmetic_chip.load_private(layouter.namespace(|| "load a"), self.a)?;
        let b = arithmetic_chip.load_private(layouter.namespace(|| "load b"), self.b)?;
        
        let c = match &self.operator {
            Operator::Add => arithmetic_chip.add(&mut layouter, a, b),
            Operator::Sub => arithmetic_chip.sub(&mut layouter, a, b),
            Operator::Mul => arithmetic_chip.mul(&mut layouter, a, b),
        }?;

        arithmetic_chip.expose_public(layouter.namespace(|| "expose c"), c, 0)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use halo2_proofs::{dev::MockProver, pasta::Fp};

    #[test]
    fn test_add() {
        let k = 4;

        let a = Fp::from(2);
        let b = Fp::from(3);
        let c = a + b;

        let circuit = CalculatorCircuit {
            operator: Operator::Add,
            a: Value::known(a),
            b: Value::known(b),
        };

        let mut public_inputs = vec![c];

        let prover = MockProver::run(k, &circuit, vec![public_inputs.clone()]).unwrap();
    
        assert_eq!(prover.verify(), Ok(()));

        public_inputs[0] += Fp::one();

        let prover = MockProver::run(k, &circuit, vec![public_inputs.clone()]).unwrap();
    
        assert!(prover.verify().is_err());
    }
}