#![feature(conservative_impl_trait, universal_impl_trait)]
#![feature(dyn_trait)]
#![feature(catch_expr)]
#![feature(fnbox)]
extern crate euclid;
extern crate image;
extern crate num_traits;
extern crate rect_iter;
mod screen;
mod world;
mod errors;

/// Wrapper of SDL or HTML5 Canvas
pub trait Context {}

/// GameData
pub struct GameData {}

// Script Engine
