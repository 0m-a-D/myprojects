use candle_core::{Device, Tensor};
use polars::prelude::*;

fn polars_example() -> PolarsResult<DataFrame> {
    CsvReader::from_path("labels.csv")?
        .has_header(false)
        .finish()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dataset = polars_example();
    println!("{:?}", dataset);

    candle_example()
}

fn candle_example() -> Result<(), Box<dyn std::error::Error>> {
    let device = Device::Cpu;

    let a = Tensor::randn(0f32, 1., (2, 3), &device)?;
    let b = Tensor::randn(0f32, 1., (3, 4), &device)?;

    let c = a.matmul(&b)?;
    println!("{c}");
    Ok(())
}
