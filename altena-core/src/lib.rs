#![feature(conservative_impl_trait, universal_impl_trait)]
#![feature(iterator_try_fold)]
#![feature(dyn_trait)]
#![feature(fnbox)]

extern crate ansi_term;
extern crate euclid;
extern crate image;
extern crate num_traits;
extern crate rect_iter;
mod screen;
mod world;
mod errors;
mod testutils;

/// Wrapper of SDL or HTML5 Canvas
pub trait Context {}

/// GameData
pub struct GameData {}

// Script Engine
