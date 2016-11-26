use nalgebra::DMatrix;
use num::Complex;

use std::f64;

pub trait Gate {
    fn max_in(&self) -> u32;
    fn max_out(&self) -> u32;
    fn process(&self, &[DMatrix<Complex<f64>>]) -> Vec<DMatrix<Complex<f64>>>;
}

#[derive(Clone)]
pub struct Input {
    theta: f64,
    phi: f64,
}

impl Input {
    pub fn new() -> Box<Gate> {
        Box::new(Input{
            theta: 0.,
            phi: 0.,
        })
    }
}

impl Gate for Input {
    fn max_in(&self) -> u32 {
        0
    }
    fn max_out(&self) -> u32 {
        1
    }
    fn process(&self, _: &[DMatrix<Complex<f64>>]) -> Vec<DMatrix<Complex<f64>>> {
        use ::num::Complex as C;
        vec![DMatrix::from_column_vector(2, 1, &[
            C::new((self.theta / 2.).cos(), 0.),
            C::new(f64::consts::E, 0.).powc(C::new(0., self.phi)) * (self.theta / 2.).sin()])]
    }
}

#[derive(Clone)]
pub struct Output;

impl Output {
    pub fn new() -> Box<Gate> {
        Box::new(Output)
    }
}

impl Gate for Output {
    fn max_in(&self) -> u32 {
        1
    }
    fn max_out(&self) -> u32 {
        0
    }
    fn process(&self, _: &[DMatrix<Complex<f64>>]) -> Vec<DMatrix<Complex<f64>>> {
        vec![]
    }
}

#[derive(Clone)]
pub struct Not;

impl Not {
    pub fn new() -> Box<Gate> {
        Box::new(Not)
    }
}

impl Gate for Not {
    fn max_in(&self) -> u32 {
        1
    }
    fn max_out(&self) -> u32 {
        1
    }
    fn process(&self, input: &[DMatrix<Complex<f64>>]) -> Vec<DMatrix<Complex<f64>>> {
        if input.len() == 1 {
            vec![::not() * input[0].clone()]
        } else {
            vec![]
        }
    }
}
