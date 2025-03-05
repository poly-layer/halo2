use std::marker::PhantomData;

use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    dev::MockProver,
    plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Expression, Selector},
    poly::Rotation,
};

use ff::Field;

const STEPS: usize = 5;

struct TestCircuit<F: Field> {
    _ph: PhantomData<F>,
    values: Value<Vec<F>>,
}

#[derive(Clone, Debug)]
struct TestConfig<F: Field + Clone> {
    _ph: PhantomData<F>,
    q_enable: Selector,
    advice: Column<Advice>,
}

impl<F: Field> Circuit<F> for TestCircuit<F> {
    type Config = TestConfig<F>;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        TestCircuit {
            _ph: PhantomData,
            values: Value::unknown(),
        }
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let q_enable = meta.complex_selector();
        let advice = meta.advice_column();

        // define a new gate:
        // next = curr + 1 if q_enable is 1
        meta.create_gate("step", |meta| {
            let curr = meta.query_advice(advice, Rotation::cur());
            let next = meta.query_advice(advice, Rotation::next());
            let q_enable = meta.query_selector(q_enable);
            vec![q_enable * (curr - next + Expression::Constant(F::ONE))]
        });

        TestConfig {
            _ph: PhantomData,
            q_enable,
            advice,
        }
    }

    fn synthesize(
        &self,
        config: Self::Config, //
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "steps",
            |mut region| {
                // apply the "step" gate STEPS = 5 times
                for i in 0..STEPS {
                    // assign the witness value to the advice column
                    region.assign_advice(
                        || "assign advice",
                        config.advice,
                        i,
                        || self.values.as_ref().map(|values| values[i]),
                    )?;

                    // turn on the gate
                    config.q_enable.enable(&mut region, i)?;
                }

                // assign the final "next" value
                region.assign_advice(
                    || "assign advice",
                    config.advice,
                    STEPS,
                    || self.values.as_ref().map(|values| values[STEPS]),
                )?;

                Ok(())
            },
        )?;
        Ok(())
    }
}
fn main() {
    let values = vec![
        Fr::from(1),  // row 0
        Fr::from(2),  // row 1
        Fr::from(3),  // row 2
        Fr::from(4),  // row 3
        Fr::from(5),  // row 4
        Fr::from(6),  // row 5 (final "next" value)
    ];
    use halo2_proofs::halo2curves::bn256::Fr;
    let circuit = TestCircuit::<Fr> { _ph: PhantomData, values: Value::known(values) };
    let public_inputs = vec![];
    let prover = MockProver::run(8, &circuit, public_inputs).unwrap();
    match prover.verify() {
        Ok(()) => println!("Proof successfully verified!"),
        Err(e) => println!("Proof verification failed: {:?}", e),
    };
}