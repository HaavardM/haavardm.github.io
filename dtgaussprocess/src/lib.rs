extern crate js_sys;
extern crate rusty_machine as rm;
extern crate serde_derive;
extern crate serde_json;
extern crate wasm_bindgen;

use rm::learning::gp;
use rm::learning::gp::MeanFunc;
use rm::learning::toolkit::kernel;
use rm::linalg::{Matrix, Vector};
use rm::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    pub fn alert(s: &str);
}

struct Mean {
    c: f64,
}

impl MeanFunc for Mean {
    fn func(&self, x: Matrix<f64>) -> Vector<f64> {
        return Vector::zeros(x.rows()) + self.c;
    }
}

#[wasm_bindgen]
pub struct GPPosterior {
    mean: Vec<f64>,
    ci_low: Vec<f64>,
    ci_high: Vec<f64>,
}

#[wasm_bindgen]
impl GPPosterior {
    pub fn mean(&self) -> Vec<f64> {
        return self.mean.clone();
    }
    pub fn ci_low(&self) -> Vec<f64> {
        return self.ci_low.clone();
    }

    pub fn ci_high(&self) -> Vec<f64> {
        return self.ci_high.clone();
    }
}

#[wasm_bindgen]
pub struct GaussianProcess {
    train_x: Vec<f64>,
    train_y: Vec<f64>,
}

#[wasm_bindgen]
impl GaussianProcess {
    pub fn new(x: Vec<f64>, y: Vec<f64>) -> GaussianProcess {
        let samples_x: Vec<f64> = x;
        let samples_y: Vec<f64> = y;
        assert_eq!(samples_x.len(), samples_y.len());
        return GaussianProcess {
            train_x: samples_x,
            train_y: samples_y,
        };
    }

    pub fn get_posterior(
        &self,
        x: &JsValue,
        length_scale: f64,
        amplitude: f64,
        noise_y: f64,
    ) -> Result<GPPosterior, JsValue> {
        let x: Vec<f64> = x.into_serde().unwrap();
        let train_x: Matrix<f64> = Matrix::new(self.train_x.len(), 1, self.train_x.clone());
        let train_y: Vector<f64> = Vector::new(self.train_y.clone());

        let mean_y: f64 = train_y.mean();

        let ker = kernel::SquaredExp::new(length_scale, amplitude);
        let mean = Mean { c: mean_y };
        let mut gaussp = gp::GaussianProcess::new(ker, mean, noise_y);
        let train_result = gaussp.train(&train_x, &train_y);
        if train_result.is_err() {
            return Err(JsValue::from_str("Training error"));
        }

        let x: Matrix<f64> = Matrix::new(x.len(), 1, x.clone());
        let res = match gaussp.get_posterior(&x) {
            Ok(res) => res,
            Err(_) => return Err(JsValue::from_str("Unable to calculate posterior")),
        };

        let mean: Vector<f64> = res.0;
        let cov: Matrix<f64> = res.1;
        let std: Vector<f64> = cov.diag().apply(&|e: f64| e.sqrt());

        let ci_high: Vector<f64> = &mean + &std * 1.95;
        let ci_low: Vector<f64> = &mean - &std * 1.95;

        return Ok(GPPosterior {
            mean: mean.into_vec(),
            ci_low: ci_low.into_vec(),
            ci_high: ci_high.into_vec(),
        });
    }
}
