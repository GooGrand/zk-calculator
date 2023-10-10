use crate::chips::{
    add::{AddChip, AddConfig, AddInstructions},
    mul::{MulChip, MulConfig, MulInstructions},
    sub::{SubChip, SubConfig, SubInstructions},
};
use halo2_proofs::{
    arithmetic::FieldExt,
    circuit::{AssignedCell, Chip, Layouter, Value},
    plonk::{Advice, Column, ConstraintSystem, Error, Instance},
};
use std::marker::PhantomData;

pub struct Number<F: FieldExt>(pub AssignedCell<F, F>);

pub trait ArithmeticInstructions<F: FieldExt>:
    AddInstructions<F> + MulInstructions<F> + SubInstructions<F>
{
    type Num;

    fn load_private(
        &self,
        layouter: impl Layouter<F>,
        value: Value<F>,
    ) -> Result<<Self as ArithmeticInstructions<F>>::Num, Error>;

    fn expose_public(
        &self,
        layouter: impl Layouter<F>,
        num: <Self as ArithmeticInstructions<F>>::Num,
        row: usize,
    ) -> Result<(), Error>;
}

#[derive(Clone, Debug)]
pub struct ArithmeticConfig {
    a: Column<Advice>,
    b: Column<Advice>,
    instance: Column<Instance>,
    add_config: AddConfig,
    sub_config: SubConfig,
    mul_config: MulConfig,
}

pub struct ArithmeticChip<F: FieldExt> {
    config: ArithmeticConfig,
    marker: PhantomData<F>,
}

impl<F: FieldExt> ArithmeticChip<F> {
    pub fn construct(
        config: <Self as Chip<F>>::Config,
        _loaded: <Self as Chip<F>>::Loaded,
    ) -> Self {
        Self {
            config,
            marker: PhantomData,
        }
    }

    pub fn configure(
        meta: &mut ConstraintSystem<F>,
        a: Column<Advice>,
        b: Column<Advice>,
        instance: Column<Instance>,
    ) -> <Self as Chip<F>>::Config {
        let add_config = AddChip::configure(meta, a, b);
        let sub_config = SubChip::configure(meta, a, b);
        let mul_config = MulChip::configure(meta, a, b);
        meta.enable_equality(instance);
        ArithmeticConfig {
            a,
            b,
            instance,
            add_config,
            sub_config,
            mul_config,
        }
    }
}

impl<F: FieldExt> Chip<F> for ArithmeticChip<F> {
    type Config = ArithmeticConfig;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

impl<F: FieldExt> ArithmeticInstructions<F> for ArithmeticChip<F> {
    type Num = Number<F>;

    fn load_private(
        &self,
        mut layouter: impl Layouter<F>,
        value: Value<F>,
    ) -> Result<<Self as ArithmeticInstructions<F>>::Num, Error> {
        let config = self.config();
        layouter.assign_region(
            || "load private",
            |mut region| {
                region
                    .assign_advice(|| "private input", config.a, 0, || value)
                    .map(Number)
            },
        )
    }

    fn expose_public(
        &self,
        mut layouter: impl Layouter<F>,
        num: <Self as ArithmeticInstructions<F>>::Num,
        row: usize,
    ) -> Result<(), Error> {
        let config = self.config();
        layouter.constrain_instance(num.0.cell(), config.instance, row)
    }
}

impl<F: FieldExt> AddInstructions<F> for ArithmeticChip<F> {
    type Num = Number<F>;

    fn add(
        &self,
        layouter: &mut impl Layouter<F>,
        a: Self::Num,
        b: Self::Num,
    ) -> Result<Self::Num, Error> {
        let config = self.config().add_config.clone();
        let add_chip = AddChip::<F>::construct(config, ());
        add_chip.add(layouter, a, b)
    }
}

impl<F: FieldExt> SubInstructions<F> for ArithmeticChip<F> {
    type Num = Number<F>;

    fn sub(
        &self,
        layouter: &mut impl Layouter<F>,
        a: Self::Num,
        b: Self::Num,
    ) -> Result<Self::Num, Error> {
        let config = self.config().sub_config.clone();
        let sub_chip = SubChip::<F>::construct(config, ());
        sub_chip.sub(layouter, a, b)
    }
}

impl<F: FieldExt> MulInstructions<F> for ArithmeticChip<F> {
    type Num = Number<F>;

    fn mul(
        &self,
        layouter: &mut impl Layouter<F>,
        a: Self::Num,
        b: Self::Num,
    ) -> Result<Self::Num, Error> {
        let config = self.config().mul_config.clone();
        let mul_chip = MulChip::<F>::construct(config, ());
        mul_chip.mul(layouter, a, b)
    }
}
