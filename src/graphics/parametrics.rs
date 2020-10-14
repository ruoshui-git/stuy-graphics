#![allow(dead_code)]

use std::option::Option;

/// A type that represents a parametric equation from 0.0 to 1.0
pub struct Parametric<F1, F2>
where
    F1: Fn(f64) -> f64,
    F2: Fn(f64) -> f64,
{
    x: F1,
    y: F2,
}

impl<'a, F1, F2> Parametric<F1, F2>
where
    F1: Fn(f64) -> f64,
    F2: Fn(f64) -> f64,
{
    /// Make a new Parametric equation instance with functions that give x and y based on t
    pub fn new(x: F1, y: F2) -> Self {
        Parametric { x, y }
    }

    /// Return point (x, y) at input t
    pub fn point_at(&self, t: f64) -> (f64, f64) {
        ((self.x)(t), (self.y)(t))
    }

    /// Return an iterator through all the points generated by this Parametric
    pub fn points_iter(&'a self, step: f64) -> impl Iterator<Item = (f64, f64)> + 'a {
        ParametricIter::new(self, step)
    }
}

pub struct ParametricIter<'a, F1, F2>
where
    F1: Fn(f64) -> f64,
    F2: Fn(f64) -> f64,
{
    parametric: &'a Parametric<F1, F2>,
    t: f64,
    step: f64,
}

impl<F1, F2> Iterator for ParametricIter<'_, F1, F2>
where
    F1: Fn(f64) -> f64,
    F2: Fn(f64) -> f64,
{
    type Item = (f64, f64);

    fn next(&mut self) -> Option<Self::Item> {
        if self.t > 1.0 {
            None
        } else {
            let old_t = self.t;
            self.t += self.step;
            Some(self.parametric.point_at(old_t))
        }
    }
}

impl<'a, F1, F2> ParametricIter<'a, F1, F2>
where
    F1: Fn(f64) -> f64,
    F2: Fn(f64) -> f64,
{
    fn new(parametric: &'a Parametric<F1, F2>, step: f64) -> Self {
        assert!(step > 0.0);
        ParametricIter {
            parametric,
            t: 0.0,
            step,
        }
    }
}
