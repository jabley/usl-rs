use levenberg_marquardt::{LeastSquaresProblem, LevenbergMarquardt};
use nalgebra::storage::Owned;
use nalgebra::{Dim, Dynamic, MatrixMN, Vector3, VectorN, U1, U3};

#[derive(Debug, Copy, Clone)]
pub struct Measurement {
    x: f64,
    n: f64,
}

impl Measurement {
    pub fn concurrency_and_latency(n: f64, r: f64) -> Measurement {
        Measurement { x: n / r, n }
    }

    pub fn concurrency_and_throughput(n: f64, x: f64) -> Measurement {
        Measurement { x, n }
    }

    pub fn throughput_and_latency(x: f64, r: f64) -> Measurement {
        Measurement { x, n: x * r }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Model {
    pub sigma: f64,
    pub kappa: f64,
    pub lambda: f64,
}

impl Model {
    pub fn build(measurements: &[Measurement]) -> Model {
        let measurements: Vec<(f64, f64)> = measurements.iter().map(|m| (m.n, m.x)).collect();
        let problem = ModelProblem {
            params: Vector3::new(
                0.1,
                0.01,
                measurements.iter().map(|&(n, x)| x / n).fold(f64::NEG_INFINITY, f64::max),
            ),
            measurements,
        };

        let (result, report) = LevenbergMarquardt::new().minimize(problem);
        assert!(report.termination.was_successful());

        Model { sigma: result.params.x, kappa: result.params.y, lambda: result.params.z }
    }

    pub fn concurrency_to_throughput(&self, n: f64) -> f64 {
        (self.lambda * n) / (1.0 + (self.sigma * (n - 1.0)) + (self.kappa * n * (n - 1.0)))
    }
}

#[derive(Clone)]
struct ModelProblem {
    params: Vector3<f64>,
    measurements: Vec<(f64, f64)>,
}

impl LeastSquaresProblem<f64, Dynamic, U3> for ModelProblem {
    type ResidualStorage = Owned<f64, Dynamic>;
    type JacobianStorage = Owned<f64, Dynamic, U3>;
    type ParameterStorage = Owned<f64, U3>;

    fn set_params(&mut self, params: &Vector3<f64>) {
        self.params.copy_from(params)
    }

    fn params(&self) -> Vector3<f64> {
        self.params
    }

    fn residuals(&self) -> Option<VectorN<f64, Dynamic>> {
        let mut residuals = VectorN::<f64, Dynamic>::zeros_generic(
            Dynamic::from_usize(self.measurements.len()),
            U1,
        );
        let model = Model { sigma: self.params.x, kappa: self.params.y, lambda: self.params.z };
        for (mut residual, &(n, x)) in residuals.row_iter_mut().zip(self.measurements.iter()) {
            let predicted = model.concurrency_to_throughput(n);
            residual[0] = x - predicted;
        }
        Some(residuals)
    }

    fn jacobian(&self) -> Option<MatrixMN<f64, Dynamic, U3>> {
        levenberg_marquardt::differentiate_numerically(&mut self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MEASUREMENTS: [(f64, f64); 32] = [
        (1.0, 955.16),
        (2.0, 1878.91),
        (3.0, 2688.01),
        (4.0, 3548.68),
        (5.0, 4315.54),
        (6.0, 5130.43),
        (7.0, 5931.37),
        (8.0, 6531.08),
        (9.0, 7219.8),
        (10.0, 7867.61),
        (11.0, 8278.71),
        (12.0, 8646.7),
        (13.0, 9047.84),
        (14.0, 9426.55),
        (15.0, 9645.37),
        (16.0, 9897.24),
        (17.0, 10097.6),
        (18.0, 10240.5),
        (19.0, 10532.39),
        (20.0, 10798.52),
        (21.0, 11151.43),
        (22.0, 11518.63),
        (23.0, 11806.0),
        (24.0, 12089.37),
        (25.0, 12075.41),
        (26.0, 12177.29),
        (27.0, 12211.41),
        (28.0, 12158.93),
        (29.0, 12155.27),
        (30.0, 12118.04),
        (31.0, 12140.4),
        (32.0, 12074.39),
    ];

    #[test]
    fn build() {
        let measurements: Vec<Measurement> = MEASUREMENTS
            .iter()
            .map(|&(n, x)| Measurement::concurrency_and_throughput(n, x))
            .collect();
        let model = Model::build(&measurements);
        println!("{:?}", model);
        for &(n, x) in MEASUREMENTS.iter() {
            println!("{} / {} / {}", n, x, model.concurrency_to_throughput(n));
        }
    }
}
