extern crate console_error_panic_hook;
extern crate js_sys;
extern crate nalgebra as na;
extern crate serde_derive;
extern crate serde_json;
extern crate wasm_bindgen;

mod gp;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    pub fn alert(s: &str);
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
    process: gp::GaussianProcess,
}

#[wasm_bindgen]
impl GaussianProcess {
    pub fn new(
        x: Vec<f64>,
        y: Vec<f64>,
        length_scale_squared_exp: f64,
        length_scale_periodic_exp: f64,
        amplitude: f64,
        period: f64,
        noise: f64,
    ) -> Result<GaussianProcess, JsValue> {
        console_error_panic_hook::set_once();

        let inputs_x = na::DVector::<f64>::from_vec(x);
        let inputs_y = na::DVector::<f64>::from_vec(y);

        let params = gp::HyperParameters {
            length_scale_periodic_exp: length_scale_periodic_exp,
            length_scale_squared_exp: length_scale_squared_exp,
            period: period,
            amplitude: amplitude,
        };

        match gp::GaussianProcess::new(&inputs_x, &inputs_y, params, noise) {
            Some(p) => Ok(GaussianProcess { process: p }),
            None => Err(JsValue::from_str("Unable to create gaussian process")),
        }
    }

    pub fn optimize_params(x: Vec<f64>, y: Vec<f64>, noise: f64) {
        let x = na::DVector::<f64>::from_vec(x);
        let y = na::DVector::<f64>::from_vec(y);

        let res = gp::optimize(&x, &y, noise);
        alert(&format!(
            "{}, {}, {}, {}",
            res.amplitude, res.length_scale, res.length_scale_periodic, res.period
        ))
    }

    pub fn posterior(&self, x: Vec<f64>) -> Result<GPPosterior, JsValue> {
        let x: na::DVector<f64> = na::DVector::from_vec(x);
        let (mean, ci) = match self.process.posterior(&x) {
            Some(x) => x,
            None => return Err(JsValue::from_str("Unable to compute posterior")),
        };

        return Ok(GPPosterior {
            mean: mean.iter().map(|x| *x).collect(),
            ci_high: ci.column(0).iter().map(|x| *x).collect(),
            ci_low: ci.column(1).iter().map(|x| *x).collect(),
        });
    }
}
