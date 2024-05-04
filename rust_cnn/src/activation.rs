use ndarray::Array1;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum Activation {
    ReLu,    // linear regression
    Sigmoid, // for classification purposes
    Softmax, // for determining probability of variable class
}
impl Activation {
    pub fn forward(x: Array1<f32>, activation: Activation) -> Array1<f32> {
        match activation {
            Activation::ReLu => Activation::relu(x),
            Activation::Sigmoid => Activation::sigmoid(x),
            Activation::Softmax => Activation::softmax(x),
        }
    }
    pub fn backward(x: Array1<f32>, activation: Activation) -> Array1<f32> {
        match activation {
            Activation::ReLu => Activation::relu_derivative(x),
            Activation::Sigmoid => Activation::sigmoid_derivative(x),
            Activation::Softmax => Activation::softmax_derivative(x),
        }
    }

    fn relu(x: Array1<f32>) -> Array1<f32> {
        x.mapv(|xi| if xi > 0.0 { xi } else { 0.0 })
    }
    fn relu_derivative(x: Array1<f32>) -> Array1<f32> {
        x.mapv(|xi| if xi > 0.0 { 1.0 } else { 0.0 })
    }
    fn sigmoid(x: Array1<f32>) -> Array1<f32> {
        x.mapv(|xi| 1.0 / (1.0 + (-xi).exp()))
    }
    fn sigmoid_derivative(x: Array1<f32>) -> Array1<f32> {
        x.mapv(|xi| xi * (1.0 - xi))
    }
    fn softmax(x: Array1<f32>) -> Array1<f32> {
        let max = x.fold(x[0], |acc, &xi| if xi > acc { xi } else { acc });
        let exps = x.mapv(|xi| (xi - max).exp());
        let sums = exps.sum();
        exps / sums
    }
    fn softmax_derivative(x: Array1<f32>) -> Array1<f32> {
        Array1::ones(x.len())
    }
}
