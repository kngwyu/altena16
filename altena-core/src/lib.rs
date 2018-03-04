#![feature(conservative_impl_trait, dyn_trait, iterator_try_fold, match_default_bindings, nll,
           universal_impl_trait)]

extern crate ansi_term;
extern crate euclid;
extern crate image;
extern crate num_traits;
extern crate rect_iter;
mod screen;
mod world;
mod errors;
#[cfg(test)]
mod testutils;

/// Wrapper of SDL or HTML5 Canvas
pub trait Context {}

/// GameData
pub struct GameData {}

// Script Engine
