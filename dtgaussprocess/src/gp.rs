extern crate nalgebra as na;
extern crate optimization as opt;

type Matrix = na::DMatrix<f64>;
type Vector = na::DVector<f64>;

struct ConstMean {
    c: f64,
}

impl ConstMean {
    fn func(&self, x: &Vector) -> Vector {
        return na::DVector::<f64>::from_element(x.nrows(), self.c);
    }
}

struct KernelDerivative {
    data: na::Vector4<f64>,
}

impl KernelDerivative {
    fn amplitude(&self) -> f64 {
        self.data.x
    }

    fn length_scale_squared_exp(&self) -> f64 {
        self.data.y
    }

    fn length_scale_periodic_exp(&self) -> f64 {
        self.data.z
    }

    fn period(&self) -> f64 {
        self.data.w
    }
}

impl From<na::Vector4<f64>> for KernelDerivative {
    fn from(v: na::Vector4<f64>) -> KernelDerivative {
        KernelDerivative { data: v }
    }
}

impl Into<na::Vector4<f64>> for KernelDerivative {
    fn into(self) -> na::Vector4<f64> {
        self.data
    }
}

pub struct Kernel {
    pub length_scale: f64,
    pub length_scale_periodic: f64,
    pub amplitude: f64,
    pub period: f64,
}

impl Kernel {
    fn new(amplitude: f64, ls: f64, lsp: f64, period: f64) -> Kernel {
        return Kernel {
            length_scale: ls,
            length_scale_periodic: lsp,
            period: period,
            amplitude: amplitude,
        };
    }

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

    fn derivative(&self, x1: &f64, x2: &f64) -> KernelDerivative {
        use std::f64::consts::PI;
        let diff = x2 - x1;
        let sdiff = diff * diff;
        let adiff = diff.abs();

        let da = (self.periodic_exp_inner(diff) + self.squared_exp_inner(diff)).exp();
        let dl1 = -sdiff * self.f(x1, x2) / (self.length_scale.powf(3.0));
        let dl2 = (4.0 * (PI * adiff / self.period).sin().powf(2.0)) * self.f(x1, x2)
            / self.length_scale_periodic.powf(3.0);
        let trig = PI * adiff / self.length_scale_periodic;
        let dp = 4.0 * PI * adiff * trig.sin() * trig.cos() * self.f(x1, x2)
            / (self.length_scale_periodic * self.length_scale_periodic * self.period * self.period);
        KernelDerivative::from(na::Vector4::new(da, dl1, dl2, dp))
    }

    fn matrix(&self, x1: &Vector, x2: &Vector) -> Matrix {
        let dim1 = x1.nrows();
        let dim2 = x2.nrows();
        unsafe {
            let mut m = Matrix::new_uninitialized(dim1, dim2);
            self.matrix_mut(x1, x2, &mut m);
            return m;
        }
    }

    fn matrix_mut(&self, m1: &Vector, m2: &Vector, out: &mut Matrix) {
        let mut data = m2
            .iter()
            .flat_map(|row1| m1.iter().map(move |row2| self.f(row1, row2)));
        // Builds matrix COLUMN BY COLUMN
        out.iter_mut().for_each(|x| *x = data.next().unwrap());
    }
}

pub struct HyperParameters {
    pub amplitude: f64,
    pub length_scale_squared_exp: f64,
    pub length_scale_periodic_exp: f64,
    pub period: f64,
}

pub struct GaussianProcess {
    mean: ConstMean,
    kernel: Kernel,
    train_mat: na::Cholesky<f64, na::Dynamic>,
    train_x: Vector,
    alpha: Vector,
}

impl GaussianProcess {
    pub fn new(
        inputs_x: &Vector,
        inputs_y: &Vector,
        params: HyperParameters,
        noise: f64,
    ) -> Option<GaussianProcess> {
        let ker = Kernel::new(
            params.length_scale_squared_exp,
            params.length_scale_periodic_exp,
            params.amplitude,
            params.period,
        );
        let mean = ConstMean { c: inputs_y.mean() };

        let noise_mat = Matrix::identity(inputs_x.nrows(), inputs_x.nrows()) * noise;
        let ker_mat = ker.matrix(&inputs_x, &inputs_x) + noise_mat;

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
        x: &Vector,
    ) -> Option<(Vector, na::MatrixMN<f64, na::Dynamic, na::U2>)> {
        let prior_ker_mat = self.kernel.matrix(&x, &self.train_x);
        let post_mean = self.mean.func(&x) + &prior_ker_mat * &self.alpha;
        let v_mat = self
            .train_mat
            .l_dirty()
            .solve_lower_triangular(&prior_ker_mat.transpose())
            .expect("Unable to solve")
            .transpose();
        let cov = self.kernel.matrix(&x, &x) - &v_mat * v_mat.transpose();

        let std: Vector = cov.map_diagonal(|e| e.sqrt());
        let ci_high: Vector = &post_mean + &std * 1.95;
        let ci_low: Vector = &post_mean - &std * 1.95;

        return Some((
            post_mean,
            na::MatrixMN::<f64, na::Dynamic, na::U2>::from_columns(&[ci_high, ci_low]),
        ));
    }
}

fn loglikelihood(x: &Vector, y: &Vector, kernel: &Kernel, noise: f64) -> Option<f64> {
    let n = x.nrows();
    let kernel = kernel.matrix(x, x) + Matrix::identity(n, n) * noise;
    let kernel_det = kernel.determinant();
    let kernel = match na::Cholesky::new(kernel) {
        Some(m) => m,
        None => return None,
    };
    let alpha = match kernel.l_dirty().solve_lower_triangular(y) {
        Some(a) => a,
        None => return None,
    };
    use std::f64::consts::PI;
    let ll = -(alpha.transpose() * alpha).x;
    let ll = ll - (2.0 * PI * kernel_det).ln();
    let ll = ll - (n as f64) * (2.0 * PI).ln();
    return Some(ll / 2.0);
}

pub fn optimize(x: &Vector, y: &Vector, noise: f64) -> Kernel {
    let f = opt::NumericalDifferentiation::new(opt::Func(|p| {
        let kernel = Kernel::new(p[0], p[1], p[2], p[3]);
        let nll = -loglikelihood(x, y, &kernel, noise).expect("Unable to calculate likelihood");
        println!("{}", nll);
        return nll;
    }));
    let gd = opt::GradientDescent::new();
    let gd = gd.max_iterations(Some(1000));
    use opt::Minimizer;
    let res = gd.minimize(&f, vec![1.0, 1.0, 1.0, 1.0]);

    return Kernel::new(
        res.position[0],
        res.position[1],
        res.position[2],
        res.position[3],
    );
}

fn dloglikelihood(ker: &Kernel, x: &Vector, y: &Vector, noise: f64) -> na::Vector4<f64> {
    let dim = x.nrows();

    let data = x
        .iter()
        .flat_map(|row1| x.iter().map(move |row2| ker.derivative(row1, row2)))
        .map(|x| {
            (
                (x.amplitude(), x.length_scale_squared_exp()),
                (x.length_scale_periodic_exp(), x.period()),
            )
        });
    let (temp1, temp2): (Vec<(f64, f64)>, Vec<(f64, f64)>) = data.unzip();
    let (d0, d1): (Vec<f64>, Vec<f64>) = temp1.iter().map(|x| *x).unzip();
    let (d2, d3): (Vec<f64>, Vec<f64>) = temp2.iter().map(|x| *x).unzip();

    let da = Matrix::from_iterator(dim, dim, d0);
    let dl1 = Matrix::from_iterator(dim, dim, d1);
    let dl2 = Matrix::from_iterator(dim, dim, d2);
    let dp = Matrix::from_iterator(dim, dim, d3);

    let kernel = ker.matrix(x, x) + Matrix::identity(dim, dim) * noise;
    let kernel = na::Cholesky::new(kernel).expect("Unable to compute cholesky");

    // %\frac{ln(PI*det(A(x))}{dx} = \frac{PI*trace(adj(A(x))*\frac{dA(x)}{dx})}{PI*det(A(x))} = trace(A^{-1}(x)*\frac{A(x)}{dx}))
    let da: f64 =
        -(y.transpose() * &da * y).get(0).expect("Not a 1v1 matrix") - (kernel.solve(&da)).trace();
    let dl1 = -(y.transpose() * &dl1 * y).get(0).expect("Not a 1v1 matrix")
        - (kernel.solve(&dl1)).trace();
    let dl2 = -(y.transpose() * &dl2 * y).get(0).expect("Not a 1v1 matrix")
        - (kernel.solve(&dl2)).trace();
    let dp =
        -(y.transpose() * &dp * y).get(0).expect("Not a 1v1 matrix") - (kernel.solve(&dp)).trace();

    na::Vector4::new(da, dl1, dl2, dp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optimize() {
        let x = na::DVector::from_vec(vec![1.0, 2.0, 3.0, 4.0]);
        let y = na::DVector::from_vec(vec![1.0, 10.0, 20.0, 1.0]);

        let res = super::optimize(&x, &y, 1e-6);
        println!(
            "{}, {}, {}, {}",
            res.amplitude, res.length_scale, res.length_scale_periodic, res.period
        );
    }

    #[test]
    fn gp() {
        let x = na::DVector::from_vec(vec![1.0, 2.0, 3.0, 4.0]);
        let y = na::DVector::from_vec(vec![1.0, 10.0, 20.0, 1.0]);
        let xs = na::DVector::from_vec(vec![1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0]);

        let ker = Kernel::new(1.0, 1.0, 1.0, 1.0);

        let noise_mat = Matrix::identity(x.nrows(), x.nrows()) * 1.0;
        let kmat = ker.matrix(&x, &x) + noise_mat;
        let chol = na::Cholesky::new(kmat.clone()).expect("Unable to compute cholesky");
        let alpha = chol.solve(&y);

        let s_ker_mat = ker.matrix(&xs, &x);
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
