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

fn build_kernel_matrix_mut(
    x1: &Vector,
    x2: &Vector,
    f: &impl Fn(&f64, &f64) -> f64,
    out: &mut Matrix,
) {
    let mut data = x2
        .iter()
        .flat_map(|row1| x1.iter().map(move |row2| f(row1, row2)));
    // Builds matrix COLUMN BY COLUMN
    out.iter_mut().for_each(|x| *x = data.next().unwrap());
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

    fn df_da(&self, x1: &f64, x2: &f64) -> f64 {
        let diff = x2 - x1;
        let sdiff = diff * diff;
        let adiff = diff.abs();
        (self.periodic_exp_inner(diff) + self.squared_exp_inner(diff)).exp()
    }

    fn df_dls(&self, x1: &f64, x2: &f64) -> f64 {
        let diff = x2 - x1;
        let sdiff = diff * diff;
        sdiff * self.f(x1, x2) / (self.length_scale.powf(3.0))
    }

    fn df_dlp(&self, x1: &f64, x2: &f64) -> f64 {
        use std::f64::consts::PI;
        let diff = x2 - x1;
        let adiff = diff.abs();
        (4.0 * (PI * adiff / self.period).sin().powf(2.0)) * self.f(x1, x2)
            / self.length_scale_periodic.powf(3.0)
    }
    fn df_dp(&self, x1: &f64, x2: &f64) -> f64 {
        use std::f64::consts::PI;
        let diff = x2 - x1;
        let adiff = diff.abs();
        let trig = PI * adiff / self.length_scale_periodic;
        4.0 * PI * adiff * trig.sin() * trig.cos() * self.f(x1, x2)
            / (self.length_scale_periodic * self.length_scale_periodic * self.period * self.period)
    }

    fn f_matrix(&self, x1: &Vector, x2: &Vector) -> Matrix {
        let dim1 = x1.nrows();
        let dim2 = x2.nrows();
        unsafe {
            let mut m = Matrix::new_uninitialized(dim1, dim2);
            self.f_matrix_mut(x1, x2, &mut m);
            return m;
        }
    }

    fn f_matrix_mut(&self, m1: &Vector, m2: &Vector, out: &mut Matrix) {
        build_kernel_matrix_mut(m1, m2, &|x1, x2| self.f(x1, x2), out)
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
    beta: Vector,
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
        let ker_mat = ker.f_matrix(&inputs_x, &inputs_x) + noise_mat;

        let train_mat = match na::Cholesky::new(ker_mat) {
            Some(c) => c,
            None => return None,
        };
        let mut b = inputs_y - mean.func(&inputs_x);
        train_mat.l_dirty().solve_lower_triangular_mut(&mut b);
        let alpha = match train_mat.l_dirty().ad_solve_lower_triangular(&b) {
            Some(a) => a,
            None => return None,
        };
        return Some(GaussianProcess {
            kernel: ker,
            mean: mean,
            train_mat: train_mat,
            train_x: inputs_x.clone(),
            alpha: alpha,
            beta: b,
        });
    }
    pub fn posterior(
        &self,
        x: &Vector,
    ) -> Option<(Vector, na::MatrixMN<f64, na::Dynamic, na::U2>)> {
        let prior_ker_mat = self.kernel.f_matrix(&x, &self.train_x);
        let post_mean = self.mean.func(&x) + &prior_ker_mat * &self.alpha;
        let v_mat = self
            .train_mat
            .l_dirty()
            .solve_lower_triangular(&prior_ker_mat.transpose())
            .expect("Unable to solve")
            .transpose();
        let cov = self.kernel.f_matrix(&x, &x) - &v_mat * v_mat.transpose();

        let std: Vector = cov.map_diagonal(|e| e.sqrt());
        let ci_high: Vector = &post_mean + &std * 1.95;
        let ci_low: Vector = &post_mean - &std * 1.95;

        return Some((
            post_mean,
            na::MatrixMN::<f64, na::Dynamic, na::U2>::from_columns(&[ci_high, ci_low]),
        ));
    }
    fn loglikelihood(&self) -> Option<f64> {
        use std::f64::consts::PI;
        let kernel_det = self
            .train_mat
            .l_dirty()
            .diagonal()
            .iter()
            .fold(1.0, |x, y| x * y)
            .powf(2.0);
        let n = self.train_x.nrows();
        let ll = -(self.beta.transpose() * &self.beta).x;
        let ll = ll - (2.0 * PI * kernel_det);
        let ll = ll - (n as f64) * (2.0 * PI).ln();
        return Some(ll / 2.0);
    }

    unsafe fn dloglikelihood(&self) -> na::Vector4<f64> {
        let n = self.train_x.nrows();
        let mut temp_kernel = Matrix::new_uninitialized(n, n);

        let aa = &self.alpha * self.alpha.transpose();
        build_kernel_matrix_mut(
            &self.train_x,
            &self.train_x,
            &|x1, x2| self.kernel.df_da(x1, x2),
            &mut temp_kernel,
        );

        let da = &aa * &temp_kernel;
        self.train_mat.solve_mut(&mut temp_kernel);
        let da = (da - &temp_kernel).trace() / 2.0;

        build_kernel_matrix_mut(
            &self.train_x,
            &self.train_x,
            &|x1, x2| self.kernel.df_dls(x1, x2),
            &mut temp_kernel,
        );
        let dls = &aa * &temp_kernel;
        self.train_mat.solve(&mut temp_kernel);
        let dls = (dls - &temp_kernel).trace() / 2.0;

        build_kernel_matrix_mut(
            &self.train_x,
            &self.train_x,
            &|x1, x2| self.kernel.df_dlp(x1, x2),
            &mut temp_kernel,
        );
        let dlp = &aa * &temp_kernel;
        self.train_mat.solve(&mut temp_kernel);
        let dlp = (dlp - &temp_kernel).trace() / 2.0;

        build_kernel_matrix_mut(
            &self.train_x,
            &self.train_x,
            &|x1, x2| self.kernel.df_dp(x1, x2),
            &mut temp_kernel,
        );

        let dp = aa * &temp_kernel;
        self.train_mat.solve(&mut temp_kernel);
        let dp = (dp - temp_kernel).trace() / 2.0;
        na::Vector4::new(da, dls, dlp, dp)
    }
}

struct ObjectiveFunc {
    x: Vector,
    y: Vector,
    noise: f64,
}

impl opt::Function for ObjectiveFunc {
    fn value(&self, x: &[f64]) -> f64 {
        let gp = GaussianProcess::new(
            &self.x,
            &self.y,
            HyperParameters {
                amplitude: x[0],
                length_scale_squared_exp: x[1],
                length_scale_periodic_exp: x[2],
                period: x[3],
            },
            self.noise,
        )
        .expect("Unable to create GP");
        let val = -gp
            .loglikelihood()
            .expect("Unable to calulcate loglikelihood");
        //println!("[{}, {}, {}, {}] -> {}", x[0], x[1], x[2], x[3], val);
        return val;
    }
}

impl opt::Function1 for ObjectiveFunc {
    fn gradient(&self, x: &[f64]) -> Vec<f64> {
        let gp = GaussianProcess::new(
            &self.x,
            &self.y,
            HyperParameters {
                amplitude: x[0],
                length_scale_squared_exp: x[1],
                length_scale_periodic_exp: x[2],
                period: x[3],
            },
            self.noise,
        )
        .expect("Unable to create GP");
        unsafe {
            let g: Vec<f64> = gp
                .dloglikelihood()
                .normalize()
                .iter()
                .map(|x| -*x)
                .collect();
            println!("Gradient: [{}, {}, {}, {}]", g[0], g[1], g[2], g[3]);
            return g;
        }
    }
}

fn optimize_params(x: Vector, y: Vector, noise: f64) -> (HyperParameters, f64) {
    use opt::Minimizer;
    let o = opt::GradientDescent::new()
        .max_iterations(Some(10))
        .gradient_tolerance(1e-2);
    //.line_search(opt::ArmijoLineSearch::new(0.2, 0.5, 0.8));

    let obj_fn = ObjectiveFunc {
        x: x,
        y: y,
        noise: noise,
    };

    let res = o.minimize(&obj_fn, vec![10.0, 10.0, 10.0, 10.0]);

    return (
        HyperParameters {
            amplitude: res.position[0],
            length_scale_squared_exp: res.position[1],
            length_scale_periodic_exp: res.position[2],
            period: res.position[3],
        },
        res.value,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimize() {
        let x = Vector::from_vec(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]);
        let y = Vector::from_vec(vec![1.0, 100.0, 50.0, 2.0, -5.0, 3.0, 30.0, 80.0]);

        let res = optimize_params(x, y, 1e-6f64);
        let params = res.0;
        println!(
            "{}, {}, {}, {} -> {}",
            params.amplitude,
            params.length_scale_squared_exp,
            params.length_scale_periodic_exp,
            params.period,
            res.1
        )
    }

    #[test]
    fn cholesky_det() {
        let a = na::Matrix3::<f64>::new(2.0, 1.0, 1.0, 1.0, 2.0, 1.0, 1.0, 1.0, 2.0);
        let ac = na::Cholesky::new(a.clone()).expect("Unable to compute cholesky");
        let ac_det = ac
            .l_dirty()
            .diagonal()
            .iter()
            .fold(1.0, |x, y| x * y)
            .powf(2.0);

        assert!(a.determinant() - ac_det < 1e-6f64);
    }

    #[test]
    fn gp() {
        let x = na::DVector::from_vec(vec![1.0, 2.0, 3.0, 4.0]);
        let y = na::DVector::from_vec(vec![1.0, 10.0, 20.0, 1.0]);
        let xs = na::DVector::from_vec(vec![1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0]);

        let ker = Kernel::new(1.0, 1.0, 1.0, 1.0);

        let noise_mat = Matrix::identity(x.nrows(), x.nrows()) * 1.0;
        let kmat = ker.f_matrix(&x, &x) + noise_mat;
        let chol = na::Cholesky::new(kmat.clone()).expect("Unable to compute cholesky");
        let alpha = chol.solve(&y);

        let s_ker_mat = ker.f_matrix(&xs, &x);
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
