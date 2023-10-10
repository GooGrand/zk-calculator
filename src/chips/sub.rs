use halo2_proofs::{
    arithmetic::FieldExt,
    circuit::{Chip, Layouter, Region},
    plonk::{Advice, Column, ConstraintSystem, Error, Selector},
    poly::Rotation,
};
use std::marker::PhantomData;

use crate::chips::arithmetic::Number;

pub trait SubInstructions<F: FieldExt>: Chip<F> {
    /// Numeric variable
    type Num;

    /// Substraction instruction
    fn sub(
        &self,
        layouter: &mut impl Layouter<F>,
        a: Self::Num,
        b: Self::Num,
    ) -> Result<Self::Num, Error>;
}

/// Substraction chip configuration
#[derive(Clone, Debug)]
pub struct SubConfig {
    /// Advice column for `input_a` and output
    a: Column<Advice>,
    /// Advice column for `input_b`
    b: Column<Advice>,
    /// Substraction selector
    sel_sub: Selector,
}

/// Substraction chip def
pub struct SubChip<F: FieldExt> {
    /// Substraction configuration
    config: SubConfig,
    /// Placeholder data
    _marker: PhantomData<F>,
}

impl<F: FieldExt> SubChip<F> {
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

        let sel_sub = meta.selector();

        meta.create_gate("add", |meta| {
            let lhs = meta.query_advice(a, Rotation::cur());
            let rhs = meta.query_advice(b, Rotation::cur());
            let out = meta.query_advice(a, Rotation::next());
            let sel_sub = meta.query_selector(sel_sub);
            vec![sel_sub * (lhs - rhs - out)]
        });

        SubConfig { a, b, sel_sub }
    }
}

impl<F: FieldExt> Chip<F> for SubChip<F> {
    type Config = SubConfig;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

impl<F: FieldExt> SubInstructions<F> for SubChip<F> {
    type Num = Number<F>;

    fn sub(
        &self,
        layouter: &mut impl Layouter<F>,
        a: Self::Num,
        b: Self::Num,
    ) -> Result<Self::Num, Error> {
        let config = self.config();

        layouter.assign_region(
            || "add",
            |mut region: Region<'_, F>| {
                config.sel_sub.enable(&mut region, 0)?;
                a.0.copy_advice(|| "lhs", &mut region, config.a, 0)?;
                b.0.copy_advice(|| "rhs", &mut region, config.b, 0)?;
                let c = a.0.value().copied() - b.0.value();

                region
                    .assign_advice(|| "lhs - rhs", config.a, 1, || c)
                    .map(Number)
            },
        )
    }
}
