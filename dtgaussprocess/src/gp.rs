extern crate nalgebra as na;
//use na::ClosedSub;
use na::{storage::Storage, Matrix, Vector};

struct Mean {
    c: f64,
}

impl Mean {
    fn func(&self, x: &na::DMatrix<f64>) -> na::DVector<f64> {
        return na::DVector::<f64>::from_element(x.ncols(), self.c);
    }
}

struct Kernel {
    length_scale: f64,
    amplitude: f64,
}

impl Kernel {
    pub fn new(amplitude: f64, ls: f64) -> Kernel {
        return Kernel {
            length_scale: ls,
            amplitude: amplitude,
        };
    }
}

impl Kernel {
    fn squared_exp_inner(&self, diff: &na::DVector<f64>) -> f64 {
        return -diff.dot(diff) / (2.0 * self.length_scale * self.length_scale);
    }

    fn f(&self, x1: &na::DVectorSlice<f64>, x2: &na::DVectorSlice<f64>) -> f64 {
        let diff = x2 - x1;
        return self.amplitude * self.squared_exp_inner(&diff).exp();
    }
}
pub struct HyperParameters {
    pub amplitude: f64,
    pub length_scale_squared_exp: f64,
}

pub struct GaussianProcess {
    mean: Mean,
    kernel: Kernel,
    train_mat: na::Cholesky<f64, na::Dynamic>,
    train_x: na::DMatrix<f64>,
    alpha: na::DVector<f64>,
}

impl GaussianProcess {
    pub fn new(
        inputs_x: &na::DMatrix<f64>,
        inputs_y: &na::DVector<f64>,
        params: HyperParameters,
        noise: f64,
    ) -> Option<GaussianProcess> {
        let ker = Kernel::new(params.length_scale_squared_exp, params.amplitude);
        let mean = Mean { c: inputs_y.mean() };

        let noise_mat = na::DMatrix::<f64>::identity(inputs_x.ncols(), inputs_x.ncols()) * noise;
        let ker_mat = ker_mat(&ker, &inputs_x, &inputs_x) + noise_mat;

        let train_mat = match na::Cholesky::new(ker_mat) {
            Some(c) => c,
            None => return None,
        };
        let mut b = inputs_y - mean.func(&inputs_x);
        train_mat.solve_mut(&mut b);
        return Some(GaussianProcess {
            kernel: ker,
            mean: mean,
            train_mat: train_mat,
            train_x: inputs_x.clone(),
            alpha: b,
        });
    }
    pub fn posterior(
        &self,
        x: &na::DMatrix<f64>,
    ) -> Option<(na::DVector<f64>, na::MatrixMN<f64, na::Dynamic, na::U2>)> {
        let prior_ker_mat = ker_mat(&self.kernel, x, &self.train_x);
        let post_mean = self.mean.func(x) + &prior_ker_mat * &self.alpha;
        let v_mat = self
            .train_mat
            .l_dirty()
            .solve_lower_triangular(&prior_ker_mat.transpose())
            .expect("Unable to solve")
            .transpose();
        let cov = ker_mat(&self.kernel, x, x) - &v_mat * v_mat.transpose();

        let std: na::DVector<f64> = cov.map_diagonal(|e| e.sqrt());
        let ci_high: na::DVector<f64> = &post_mean + &std * 1.95;
        let ci_low: na::DVector<f64> = &post_mean - &std * 1.95;

        return Some((
            post_mean,
            na::MatrixMN::<f64, na::Dynamic, na::U2>::from_columns(&[ci_high, ci_low]),
        ));
    }
}

fn ker_mat(ker: &Kernel, m1: &na::DMatrix<f64>, m2: &na::DMatrix<f64>) -> na::DMatrix<f64> {
    let dim1 = m1.ncols();
    let dim2 = m2.ncols();
    unsafe {
        let mut m = na::DMatrix::<f64>::new_uninitialized(dim1, dim2);
        ker_mat_mut(ker, m1, m2, &mut m);
        return m;
    }
}

fn ker_mat_mut(
    ker: &Kernel,
    m1: &na::DMatrix<f64>,
    m2: &na::DMatrix<f64>,
    out: &mut na::DMatrix<f64>,
) {
    let mut data = m2
        .column_iter()
        .flat_map(|row1| m1.column_iter().map(move |row2| ker.f(&row1, &row2)));
    // Builds matrix COLUMN BY COLUMN
    out.iter_mut().for_each(|x| *x = data.next().unwrap());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gp() {
        let x = na::DMatrix::from_rows(vec![1.0, 2.0, 3.0, 4.0]);
        let y = na::DVector::from_vec(vec![1.0, 10.0, 20.0, 1.0]).transpose();
        let xs = na::DMatrix::from_rows(vec![1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0]);

        let ker = Kernel::new(1.0, 1.0);

        let noise_mat = na::DMatrix::<f64>::identity(x.nrows(), x.nrows()) * 1.0;
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
    #[test]
    fn cholesky() {
        let a = na::Matrix2::from_vec(vec![3f64, 2f64, 2f64, 3f64]);
        let ac = na::Cholesky::new(a).expect("Unable to compute cholesky");
        let b = na::Vector2::new(1.0f64, 1.0f64);
        let x = ac.solve(&b);
        assert_eq!(a * x, b);
    }
}
