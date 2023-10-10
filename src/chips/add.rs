use halo2_proofs::{
    arithmetic::FieldExt,
    circuit::{Chip, Layouter, Region},
    plonk::{Advice, Column, ConstraintSystem, Error, Selector},
    poly::Rotation,
};
use std::marker::PhantomData;

use crate::chips::arithmetic::Number;

pub trait AddInstructions<F: FieldExt>: Chip<F> {
    /// Numeric variable
    type Num;

    /// Addition instruction
    fn add(
        &self,
        layouter: &mut impl Layouter<F>,
        a: Self::Num,
        b: Self::Num,
    ) -> Result<Self::Num, Error>;
}

/// Addition chip configuration
#[derive(Clone, Debug)]
pub struct AddConfig {
    /// Advice column for `input_a` and output
    a: Column<Advice>,
    /// Advice column for `input_b`
    b: Column<Advice>,
    /// Addition selector
    sel_add: Selector,
}

/// Addition chip def
pub struct AddChip<F: FieldExt> {
    /// Addition configuration
    config: AddConfig,
    /// Placeholder data
    _marker: PhantomData<F>,
}

impl<F: FieldExt> AddChip<F> {
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

        let sel_add = meta.selector();

        meta.create_gate("add", |meta| {
            let lhs = meta.query_advice(a, Rotation::cur());
            let rhs = meta.query_advice(b, Rotation::cur());
            let out = meta.query_advice(a, Rotation::next());
            let sel_add = meta.query_selector(sel_add);
            vec![sel_add * (lhs + rhs - out)]
        });

        AddConfig { a, b, sel_add }
    }
}

impl<F: FieldExt> Chip<F> for AddChip<F> {
    type Config = AddConfig;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

impl<F: FieldExt> AddInstructions<F> for AddChip<F> {
    type Num = Number<F>;

    fn add(
        &self,
        layouter: &mut impl Layouter<F>,
        a: Self::Num,
        b: Self::Num,
    ) -> Result<Self::Num, Error> {
        let config = self.config();

        layouter.assign_region(
            || "add",
            |mut region: Region<'_, F>| {
                config.sel_add.enable(&mut region, 0)?;
                a.0.copy_advice(|| "lhs", &mut region, config.a, 0)?;
                b.0.copy_advice(|| "rhs", &mut region, config.b, 0)?;
                let c = a.0.value().copied() + b.0.value();

                region
                    .assign_advice(|| "lhs + rhs", config.a, 1, || c)
                    .map(Number)
            },
        )
    }
}
