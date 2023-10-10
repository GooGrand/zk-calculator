use zk_calculator::ZkCalculator;

pub mod calculator_circuit;
pub mod chips;
pub mod errors;
pub mod zk_calculator;

fn main() {
    ZkCalculator::new().run();
}
