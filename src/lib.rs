#![allow(warnings)]
extern crate nalgebra;
extern crate num;
extern crate itertools;
extern crate daggy;

use num::{Complex, One, Zero};
use nalgebra::{DMatrix, Eye};

use std::ops::{Index, Mul};

use circuit::*;

pub mod circuit;

#[derive(Clone, Debug)]
pub struct Qubit {
    q: DMatrix<Complex<f64>>
}

impl Qubit {
    pub fn new(a: Complex<f64>, b: Complex<f64>) -> Qubit {
        Qubit {
            q: DMatrix::from_column_vector(2, 1, &[a, b]),
        }
    }

    pub fn one() -> Qubit {
        Qubit {
            q: DMatrix::from_column_vector(2, 1, &[Complex::new(1., 0.), Complex::new(0., 0.)]),
        }
    }

    pub fn mat(&self) -> &DMatrix<Complex<f64>> {
        &self.q
    }
}

impl Index<usize> for Qubit {
    type Output = Complex<f64>;

    fn index(&self, n: usize) -> &Complex<f64> {
        &self.q[(0, n)]
    }
}

pub fn kronecker_product<T: Mul + Zero + One + Clone + Copy>(matrices: &[DMatrix<T>]) -> DMatrix<T> {
    matrices.iter().fold(DMatrix::new_identity(1), |a, b| {
        let h = b.nrows() * a.nrows();
        let w = b.ncols() * a.ncols();
        let mut result = DMatrix::new_ones(h, w);
        for r in 0..h {
            for c in 0..w {
                let ax = r / b.nrows();
                let ay = c / b.ncols();
                let bx = r % b.nrows();
                let by = c % b.ncols();
                result[(r, c)] = a[(ax, ay)] * b[(bx, by)];
            }
        }
        return result;
    })
}

pub fn apply_to_qubit(gate: DMatrix<Complex<f64>>, index: usize, amount: usize) -> DMatrix<Complex<f64>> {
    let mut vec: Vec<_> = ::std::iter::repeat(DMatrix::new_identity(2))
        .take(amount - 1)
        .collect();
    vec.insert(index, gate);
    kronecker_product(&vec[..])
}

pub fn not() -> DMatrix<Complex<f64>> {
    use self::num::Complex as C;
    DMatrix::from_column_vector(2, 2,
        &[C::zero(), C::one(),
          C::one(), C::zero()])
}

pub fn pauli_y() -> DMatrix<Complex<f64>> {
    use self::num::Complex as C;
    DMatrix::from_column_vector(2, 2,
        &[ C::zero(), C::new(0., 1.),
          -C::new(0., 1.), C::zero()])
}

pub fn pauli_z() -> DMatrix<Complex<f64>> {
    use self::num::Complex as C;
    DMatrix::from_column_vector(2, 2,
        &[C::one(), C::zero(),
          C::zero(), -C::one()])
}

pub fn hadamard() -> DMatrix<Complex<f64>> {
    use self::num::Complex as C;
    let y = C::new(1. / 2f64.sqrt(), 0.);
    DMatrix::from_column_vector(2, 2,
        &[y,  y,
          y, -y
        ])
}

pub fn control_not(control: usize, target: usize, qubits: usize) -> DMatrix<Complex<f64>> {
    let mut result = DMatrix::new_zeros(2usize.pow(qubits as u32), 2usize.pow(qubits as u32));
    let basis_out = basis_n(qubits)
        .map(|mut v| {
            if v[control] == 1 {
                v[target] ^= 1;
            }
            v
        });
    for (i, b) in basis_out.enumerate() {
        let mut n = 0;
        for (j, c) in b.iter().enumerate() {
            n += c * 2usize.pow((qubits - 1 - j) as u32);
        }
        result[(n, i)] = Complex::one();
    }
    result
}

#[test]
fn control_not_test() {
    let r = DMatrix::from_column_vector(8, 8, &[
        1., 0., 0., 0., 0., 0., 0., 0.,
        0., 1., 0., 0., 0., 0., 0., 0.,
        0., 0., 1., 0., 0., 0., 0., 0.,
        0., 0., 0., 1., 0., 0., 0., 0.,
        0., 0., 0., 0., 0., 0., 1., 0.,
        0., 0., 0., 0., 0., 0., 0., 1.,
        0., 0., 0., 0., 1., 0., 0., 0.,
        0., 0., 0., 0., 0., 1., 0., 0.,
        ].iter().map(|n| Complex::new(*n, 0.)).collect::<Vec<_>>()[..]);
    assert_eq!(r, control_not(0, 1, 3));
}

pub struct BasisIter {
    n: usize,
    i: usize,
}

impl Iterator for BasisIter {
    type Item = Vec<usize>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.i < 2usize.pow(self.n as u32) {
            let mut result = Vec::with_capacity(self.n);
            for x in (0..self.n).rev() {
                result.push((self.i >> x) & 1)
            }
            self.i += 1;
            Some(result)
        } else {
            None
        }
    }
}

pub fn basis_n(n: usize) -> BasisIter {
    BasisIter {
        n: n,
        i: 0,
    }
}

#[test]
fn basis_n_test() {
    let mut b = basis_n(3);
    assert_eq!(Some(vec![0, 0, 0]), b.next());
    assert_eq!(Some(vec![0, 0, 1]), b.next());
    assert_eq!(Some(vec![0, 1, 0]), b.next());
    assert_eq!(Some(vec![0, 1, 1]), b.next());
    assert_eq!(Some(vec![1, 0, 0]), b.next());
    assert_eq!(Some(vec![1, 0, 1]), b.next());
    assert_eq!(Some(vec![1, 1, 0]), b.next());
    assert_eq!(Some(vec![1, 1, 1]), b.next());
    assert_eq!(None, b.next());
    assert_eq!(None, b.next());
}

pub fn cnot_rule(x: usize, y: usize) -> (usize, usize) {
    (x, x ^ y)
}

#[test]
fn cnot_rule_test() {
    assert_eq!((0, 1), cnot_rule(0, 1));
    assert_eq!((1, 1), cnot_rule(1, 0));
    assert_eq!((1, 0), cnot_rule(1, 1));
}


#[test]
fn not_gate_test() {
    use self::num::Complex as C;
    let sqrt2 = C::new(1. / 2f64.sqrt(), 0.);
    let q1 = Qubit::new(sqrt2, sqrt2);
    let q2 = Qubit::new(C::new(0., 1.), C::zero());
    let q3 = Qubit::new(C::new(3. / 10f64.sqrt(), 0.), C::new(1. / 10f64.sqrt(), 0.));
    assert_eq!(
        *Qubit::new(sqrt2, sqrt2).mat(),
        not() * q1.mat().clone()
    );
    assert_eq!(
        *Qubit::new(C::zero(), C::new(0., 1.)).mat(),
        not() * q2.mat().clone()
    );
    assert_eq!(
        *Qubit::new(C::new(1. / 10f64.sqrt(), 0.), C::new(3. / 10f64.sqrt(), 0.)).mat(),
        not() * q3.mat().clone()
    );

    let q123 = kronecker_product(&[q1.mat().clone(), q2.mat().clone(), q3.mat().clone()]);
    println!("{:?}", q123);

    let asdads = apply_to_qubit(not(), 0, 3);
    println!("{:?}", asdads * q123.clone());

    let asdads = apply_to_qubit(not(), 1, 3);
    println!("{:?}", asdads * q123.clone());

    let sqrt5_2 = 2. * 5f64.sqrt();
    let n3 = apply_to_qubit(not(), 2, 3);
    let r = n3 * q123.clone();
    let rr = DMatrix::from_column_vector(8, 1, &[
            C::new(0., 1. / sqrt5_2), C::new(0., 3. / sqrt5_2),
            C::zero(), C::zero(),
            C::new(0., 1. / sqrt5_2), C::new(0., 3. / sqrt5_2),
            C::zero(), C::zero()]);
    assert!(r.as_vector().iter().zip(rr.as_vector()).all(|(a, b)| (a - b).norm() < 0.000001));
    // println!("{:?}", asdads * q123.clone());

    let dasd = apply_to_qubit(hadamard(), 1, 3);
    let asd = apply_to_qubit(not(), 2, 3);
    println!("{:?}", dasd * asd * q123.clone());

    let sqrt5_2 = 2. * 5f64.sqrt();
    let h2 = apply_to_qubit(hadamard(), 1, 3);
    let n3 = apply_to_qubit(not(), 2, 3);
    let r = h2.clone() * h2 * n3 * q123.clone();
    let rr = DMatrix::from_column_vector(8, 1, &[
            C::new(0., 1. / sqrt5_2), C::new(0., 3. / sqrt5_2),
            C::zero(), C::zero(),
            C::new(0., 1. / sqrt5_2), C::new(0., 3. / sqrt5_2),
            C::zero(), C::zero()]);
    assert!(r.as_vector().iter().zip(rr.as_vector()).all(|(a, b)| (a - b).norm() < 0.000001));
}

#[test]
fn apply_to_qubit_test() {
    let r = DMatrix::from_column_vector(8, 8, &[
        0., 0., 0., 0., 1., 0., 0., 0.,
        0., 0., 0., 0., 0., 1., 0., 0.,
        0., 0., 0., 0., 0., 0., 1., 0.,
        0., 0., 0., 0., 0., 0., 0., 1.,
        1., 0., 0., 0., 0., 0., 0., 0.,
        0., 1., 0., 0., 0., 0., 0., 0.,
        0., 0., 1., 0., 0., 0., 0., 0.,
        0., 0., 0., 1., 0., 0., 0., 0.,
        ].iter().map(|n| Complex::new(*n, 0.)).collect::<Vec<_>>()[..]);
    assert_eq!(r, apply_to_qubit(not(), 0, 3));
    let r = DMatrix::from_column_vector(8, 8, &[
        0., 0., 1., 0., 0., 0., 0., 0.,
        0., 0., 0., 1., 0., 0., 0., 0.,
        1., 0., 0., 0., 0., 0., 0., 0.,
        0., 1., 0., 0., 0., 0., 0., 0.,
        0., 0., 0., 0., 0., 0., 1., 0.,
        0., 0., 0., 0., 0., 0., 0., 1.,
        0., 0., 0., 0., 1., 0., 0., 0.,
        0., 0., 0., 0., 0., 1., 0., 0.,
        ].iter().map(|n| Complex::new(*n, 0.)).collect::<Vec<_>>()[..]);
    assert_eq!(r, apply_to_qubit(not(), 1, 3));
    let r = DMatrix::from_column_vector(8, 8, &[
        0., 1., 0., 0., 0., 0., 0., 0.,
        1., 0., 0., 0., 0., 0., 0., 0.,
        0., 0., 0., 1., 0., 0., 0., 0.,
        0., 0., 1., 0., 0., 0., 0., 0.,
        0., 0., 0., 0., 0., 1., 0., 0.,
        0., 0., 0., 0., 1., 0., 0., 0.,
        0., 0., 0., 0., 0., 0., 0., 1.,
        0., 0., 0., 0., 0., 0., 1., 0.,
        ].iter().map(|n| Complex::new(*n, 0.)).collect::<Vec<_>>()[..]);
    assert_eq!(r, apply_to_qubit(not(), 2, 3));
}

#[test]
fn kronecker_product_matrix_test() {
    use self::nalgebra::DMatrix as M;
    assert_eq!(
        M::<Complex<f64>>::new_identity(4),
        kronecker_product(&[M::new_identity(2), M::new_identity(2)])
    );
    assert_eq!(
        DMatrix::from_column_vector(4, 4,
            &[0., 0., 1., 0.,
              0., 0., 0., 1.,
              1., 0., 0., 0.,
              0., 1., 0., 0.]
          .iter().map(|n| Complex::new(*n, 0.)).collect::<Vec<_>>()[..]),
        kronecker_product(&[not(), M::new_identity(2)])
    );
    assert_eq!(
        DMatrix::from_column_vector(4, 4,
            &[0., 1., 0., 0.,
              1., 0., 0., 0.,
              0., 0., 0., 1.,
              0., 0., 1., 0.]
          .iter().map(|n| Complex::new(*n, 0.)).collect::<Vec<_>>()[..]),
        kronecker_product(&[M::new_identity(2), not()])
    );
}

#[test]
fn kronecker_product_qubit_test() {
    use self::Qubit as Q;
    use self::num::Complex as C;
    let r = kronecker_product(&[Q::one().mat().clone(), Q::one().mat().clone()]);
    assert_eq!(DMatrix::from_column_vector(4, 1, &[C::one(), C::zero(), C::zero(), C::zero()]), r);

    let sqrt3 = 3f64.sqrt();
    let sqrt34 = 34f64.sqrt();
    let sqrt3_34 = (3f64 / 34.).sqrt();
    let sqrt102 = 102f64.sqrt();

    let r = kronecker_product(&[
        Q::new(C::new(1., 1.) / sqrt3, C::new(1., 0.) / sqrt3).mat().clone(),
        Q::new(C::new(5., 0.) / sqrt34, C::new(3., 0.) / sqrt34).mat().clone()]
    );
    let rr = DMatrix::from_column_vector(4, 1,
        &[C::new(5., 5.) / sqrt102,
          C::new(sqrt3_34, sqrt3_34),
          C::new(5., 0.) / sqrt102,
          C::new(sqrt3_34, 0.)]
    );
    assert!(r.as_vector().iter().zip(rr.as_vector()).all(|(a, b)| (a - b).norm() < 0.000001))
}
