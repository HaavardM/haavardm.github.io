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

struct Kernel {
    length_scale: f64,
    amplitude: f64,
}

impl Kernel {
    pub fn new(ls: f64, amplitude: f64) -> Kernel {
        return Kernel {
            length_scale: ls,
            amplitude: amplitude,
        }
    }
}

impl kernel::Kernel for Kernel {
    fn kernel(&self, x: &[f64], y: &[f64]) -> f64 {
        return kernel::SquaredExp::new(self.length_scale, self.amplitude).kernel(x, y);
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
    process: gp::GaussianProcess<Kernel, Mean>,
}

#[wasm_bindgen]
impl GaussianProcess {
    pub fn new(
        x: Vec<f64>, 
        y: Vec<f64>,
        length_scale: f64,
        amplitude: f64,
        noise_y: f64,
    ) -> Result<GaussianProcess, JsValue> {
        let train_x: Matrix<f64> = Matrix::new(x.len(), 1, x);
        let train_y: Vector<f64> = Vector::new(y);

        let mean_y: f64 = train_y.mean();
        let ker = Kernel::new(length_scale, amplitude);
        let mean = Mean { c: mean_y };
        let mut gaussp = gp::GaussianProcess::new(ker, mean, noise_y);
        let train_result = gaussp.train(&train_x, &train_y);
        if train_result.is_err() {
            return Err(JsValue::from_str("Training error"));
        }
        return Ok(GaussianProcess {
            process: gaussp,
        });
    }

    pub fn posterior(
        &self,
        x: Vec<f64>,
    ) -> Result<GPPosterior, JsValue> {

        let x: Matrix<f64> = Matrix::new(x.len(), 1, x);
        let res = match self.process.get_posterior(&x) {
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

    pub fn mean(&self, x: Vec<f64>) -> Result<Vec<f64>, JsValue> {
        let input = Matrix::new(x.len(), 1, x);
        let res = self.process.predict(&input);
        return match res {
            Ok(res) => Ok(res.into_vec()),
            Err(_) => Err(JsValue::from_str("Unable to predict mean")),
        };
    }
}
