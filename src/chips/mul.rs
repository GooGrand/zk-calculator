use halo2_proofs::{
    arithmetic::FieldExt,
    circuit::{Chip, Layouter, Region},
    plonk::{Advice, Column, ConstraintSystem, Error, Selector},
    poly::Rotation,
};
use std::marker::PhantomData;

use crate::chips::arithmetic::Number;

pub trait MulInstructions<F: FieldExt>: Chip<F> {
    /// Numeric variable
    type Num;

    /// Multiplication instruction
    fn mul(
        &self,
        layouter: &mut impl Layouter<F>,
        a: Self::Num,
        b: Self::Num,
    ) -> Result<Self::Num, Error>;
}

/// Multiplication chip configuration
#[derive(Clone, Debug)]
pub struct MulConfig {
    /// Advice column for `input_a` and output
    a: Column<Advice>,
    /// Advice column for `input_b`
    b: Column<Advice>,
    /// Multiplication selector
    sel_mul: Selector,
}

/// Multiplication chip def
pub struct MulChip<F: FieldExt> {
    /// Multiplication configuration
    config: MulConfig,
    /// Placeholder data
    _marker: PhantomData<F>,
}

impl<F: FieldExt> MulChip<F> {
    pub fn construct(
        config: <Self as Chip<F>>::Config,
        _loaded: <Self as Chip<F>>::Loaded,
    ) -> Self {
        Self {
            config,
            _marker: PhantomData,
        }
    }

    pub fn configure(
        meta: &mut ConstraintSystem<F>,
        a: Column<Advice>,
        b: Column<Advice>,
    ) -> <Self as Chip<F>>::Config {
        meta.enable_equality(a);
        meta.enable_equality(b);

        let sel_mul = meta.selector();

        meta.create_gate("add", |meta| {
            let lhs = meta.query_advice(a, Rotation::cur());
            let rhs = meta.query_advice(b, Rotation::cur());
            let out = meta.query_advice(a, Rotation::next());
            let sel_mul = meta.query_selector(sel_mul);
            vec![sel_mul * (lhs * rhs - out)]
        });

        MulConfig { a, b, sel_mul }
    }
}

impl<F: FieldExt> Chip<F> for MulChip<F> {
    type Config = MulConfig;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

impl<F: FieldExt> MulInstructions<F> for MulChip<F> {
    type Num = Number<F>;

    fn mul(
        &self,
        layouter: &mut impl Layouter<F>,
        a: Self::Num,
        b: Self::Num,
    ) -> Result<Self::Num, Error> {
        let config = self.config();

        layouter.assign_region(
            || "add",
            |mut region: Region<'_, F>| {
                config.sel_mul.enable(&mut region, 0)?;
                a.0.copy_advice(|| "lhs", &mut region, config.a, 0)?;
                b.0.copy_advice(|| "rhs", &mut region, config.b, 0)?;
                let c = a.0.value().copied() * b.0.value();

                region
                    .assign_advice(|| "lhs * rhs", config.a, 1, || c)
                    .map(Number)
            },
        )
    }
}
