extern crate console_error_panic_hook;
extern crate js_sys;
extern crate nalgebra as na;
extern crate serde_derive;
extern crate serde_json;
extern crate wasm_bindgen;

use na::DMatrix;
use na::DVector;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    pub fn alert(s: &str);
}

struct Mean {
    c: f64,
}

impl Mean {
    fn func(&self, x: &DVector<f64>) -> DVector<f64> {
        return DVector::<f64>::from_element(x.nrows(), self.c);
    }
}

struct Kernel {
    length_scale: f64,
    length_scale_periodic: f64,
    amplitude: f64,
    period: f64,
}

impl Kernel {
    pub fn new(ls: f64, lsp: f64, amplitude: f64, period: f64) -> Kernel {
        return Kernel {
            length_scale: ls,
            length_scale_periodic: lsp,
            period: period,
            amplitude: amplitude,
        };
    }
}

impl Kernel {
    fn periodic_exp_inner(&self, diff: f64) -> f64 {
        use std::f64::consts::PI;
        return (-2f64 / self.length_scale_periodic.powf(2.0))
            * (PI * (diff * diff).sqrt() / self.period).sin().powf(2f64);
    }

    fn squared_exp_inner(&self, diff: f64) -> f64 {
        return -diff * diff / (2.0 * self.length_scale * self.length_scale);
    }

    fn f(&self, x1: &f64, x2: &f64) -> f64 {
        let diff = x2 - x1;
        return self.amplitude
            * (self.periodic_exp_inner(diff) + self.squared_exp_inner(diff)).exp();
    }

    fn derivative(&self, x1: &f64, x2: &f64) -> na::Vector4<f64> {
        let diff = x2 - x1;
        let sdiff = diff * diff;
        let adiff = diff.abs();

        let da = (self.periodic_exp_inner(diff) + self.squared_exp_inner(diff)).exp();
        let dl1 = (self.length_scale * self.length_scale + sdiff) * self.f(x1, x2);
        let dl2 = 2.0 * (adiff / self.period).sin().powf(2.0) * self.f(x1, x2)
            / (self.length_scale_periodic * self.length_scale_periodic);
        let temp = adiff / self.length_scale_periodic;
        let dp = 4.0 * adiff * temp.sin() * temp.cos() * self.f(x1, x2)
            / (self.length_scale_periodic * self.period * self.period);
        na::Vector4::new(da, dl1, dl2, dp)
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
    mean: Mean,
    kernel: Kernel,
    train_mat: na::Cholesky<f64, na::Dynamic>,
    train_x: DVector<f64>,
    alpha: DVector<f64>,
}

#[wasm_bindgen]
pub struct HyperParameters {
    pub amplitude: f64,
    pub length_scale_squared_exp: f64,
    pub length_scale_periodic_exp: f64,
    pub period: f64,
}

fn ker_mat(ker: &Kernel, m1: &DVector<f64>, m2: &DVector<f64>) -> DMatrix<f64> {
    let dim1 = m1.nrows();
    let dim2 = m2.nrows();

    let data = m2
        .iter()
        .flat_map(|row1| m1.iter().map(move |row2| ker.f(row1, row2)));
    // Builds matrix COLUMN BY COLUMN
    return DMatrix::<f64>::from_iterator(dim1, dim2, data);
}

fn dloglikelihood(ker: &Kernel, m1: &DVector<f64>) -> na::Vector4<f64> {
    let dim = m1.nrows();

    let data = m1
        .iter()
        .flat_map(|row1| m1.iter().map(move |row2| ker.derivative(row1, row2)));
    let data = data.map(|x| {
        (
            (
                *x.get(0).expect("Not a 4 vector"),
                *x.get(1).expect("Not a 4 vector"),
            ),
            (
                *x.get(2).expect("Not a 4 vector"),
                *x.get(3).expect("Not a 4 vector"),
            ),
        )
    });
    let (temp1, temp2): (Vec<(f64, f64)>, Vec<(f64, f64)>) = data.unzip();
    let (d0, d1): (Vec<f64>, Vec<f64>) = temp1.iter().map(|x| *x).unzip();
    let (d2, d3): (Vec<f64>, Vec<f64>) = temp2.iter().map(|x| *x).unzip();

    let da = na::DMatrix::<f64>::from_iterator(dim, dim, d0);
    let dl1 = na::DMatrix::<f64>::from_iterator(dim, dim, d1);
    let dl2 = na::DMatrix::<f64>::from_iterator(dim, dim, d2);
    let dp = na::DMatrix::<f64>::from_iterator(dim, dim, d3);

    let kernel = ker_mat(ker, m1, m1);
    let kernel_det = kernel.determinant();
    let kernel = na::Cholesky::new(kernel).expect("Unable to compute cholesky");

    //TODO Is trace(aV) / a = trace(V)?
    let da: f64 = -(m1.transpose() * &da * m1)
        .get(0)
        .expect("Not a 1v1 matrix")
        - (kernel.solve(&da) * kernel_det).trace();
    let dl1 = -(m1.transpose() * &dl1 * m1)
        .get(0)
        .expect("Not a 1v1 matrix")
        - (kernel.solve(&dl1) * kernel_det).trace();
    let dl2 = -(m1.transpose() * &dl2 * m1)
        .get(0)
        .expect("Not a 1v1 matrix")
        - (kernel.solve(&dl2) * kernel_det).trace();
    let dp = -(m1.transpose() * &dp * m1)
        .get(0)
        .expect("Not a 1v1 matrix")
        - (kernel.solve(&dp) * kernel_det).trace();

    na::Vector4::new(da, dl1, dl2, dp)
}

#[wasm_bindgen]
impl GaussianProcess {
    pub fn new(
        x: Vec<f64>,
        y: Vec<f64>,
        params: HyperParameters,
        noise: f64,
    ) -> Result<GaussianProcess, JsValue> {
        console_error_panic_hook::set_once();

        let inputs_x = DVector::<f64>::from_vec(x);
        let inputs_y = DVector::<f64>::from_vec(y);
        let ker = Kernel::new(
            params.length_scale_squared_exp,
            params.length_scale_periodic_exp,
            params.amplitude,
            params.period,
        );
        let mean = Mean { c: inputs_y.mean() };

        let noise_mat = DMatrix::<f64>::identity(inputs_x.nrows(), inputs_x.nrows()) * noise;
        let ker_mat = ker_mat(&ker, &inputs_x, &inputs_x) + noise_mat;

        let train_mat = match na::Cholesky::new(ker_mat) {
            Some(c) => c,
            None => return Err(JsValue::from_str("Unable to compute cholesky")),
        };
        let mut b = inputs_y - mean.func(&inputs_x);
        train_mat.solve_mut(&mut b);
        return Ok(GaussianProcess {
            kernel: ker,
            mean: mean,
            train_mat: train_mat,
            train_x: inputs_x,
            alpha: b,
        });
    }

    pub fn optimize_params(x: Vec<f64>, y: Vec<f64>) -> Result<HyperParameters, JsValue> {
        Ok(HyperParameters {
            length_scale_squared_exp: 1.0,
            length_scale_periodic_exp: 1.0,
            amplitude: 1.0,
            period: 1.0,
        })
    }

    pub fn posterior(&self, x: Vec<f64>) -> Result<GPPosterior, JsValue> {
        let x: DVector<f64> = DVector::from_vec(x);
        let prior_ker_mat = ker_mat(&self.kernel, &x, &self.train_x);
        let post_mean = self.mean.func(&x) + &prior_ker_mat * &self.alpha;
        let v_mat = self
            .train_mat
            .l_dirty()
            .solve_lower_triangular(&prior_ker_mat.transpose())
            .expect("Unable to solve")
            .transpose();
        let cov = ker_mat(&self.kernel, &x, &x) - &v_mat * v_mat.transpose();

        let std: DVector<f64> = cov.map_diagonal(|e| e.sqrt());
        let ci_high: DVector<f64> = &post_mean + &std * 1.95;
        let ci_low: DVector<f64> = &post_mean - &std * 1.95;

        return Ok(GPPosterior {
            mean: post_mean.iter().map(|e| *e).collect(),
            ci_low: ci_low.iter().map(|e| *e).collect(),
            ci_high: ci_high.iter().map(|e| *e).collect(),
        });
    }

    pub fn mean(&self, x: Vec<f64>) -> Result<Vec<f64>, JsValue> {
        let input = DVector::from_vec(x);
        let mean =
            self.mean.func(&input) + ker_mat(&self.kernel, &input, &self.train_x) * &self.alpha;
        Ok(mean.iter().map(|x| *x).collect())
    }
}

#[test]
fn cholesky() {
    let A = na::Matrix2::from_vec(vec![3f64, 2f64, 2f64, 3f64]);
    let Ac = na::Cholesky::new(A).expect("Unable to compute cholesky");
    let b = na::Vector2::new(1.0f64, 1.0f64);
    let x = Ac.solve(&b);
    assert_eq!(A * x, b);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gp() {
        let x = na::DVector::from_vec(vec![1.0, 2.0, 3.0, 4.0]);
        let y = na::DVector::from_vec(vec![1.0, 10.0, 20.0, 1.0]);
        let xs = na::DVector::from_vec(vec![1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0]);

        let ker = Kernel::new(1.0, 1.0, 1.0, 1.0);

        let noise_mat = DMatrix::<f64>::identity(x.nrows(), x.nrows()) * 1.0;
        let kmat = ker_mat(&ker, &x, &x) + noise_mat;
        let chol = na::Cholesky::new(kmat.clone()).expect("Unable to compute cholesky");
        let alpha = chol.solve(&y);

        let s_ker_mat = ker_mat(&ker, &xs, &x);
        assert!((kmat.clone() * &alpha).relative_eq(&y, 1e-9, 1e-9));
        println!("{}", &kmat.to_string());
        println!("{}", &chol.l().to_string());
        println!("{}", &(&s_ker_mat * alpha).to_string());
        println!("{}", &(s_ker_mat.to_string()));
    }
}
