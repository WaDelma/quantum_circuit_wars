#[macro_use]
extern crate glium;
extern crate daggy;
extern crate rusttype;
extern crate unicode_normalization;
extern crate nalgebra;
extern crate image;
extern crate arrayvec;

use glium::{DisplayBuild, Program};
use glium::glutin::WindowBuilder;

use graphics::RenderContext;

type Mat = nalgebra::Matrix4<f32>;
type Vect = nalgebra::Vector2<f32>;

mod events;
mod graphics;
mod math;

fn main() {
    println!("Let the quantum circuit wars begin!");
    let display = WindowBuilder::new().build_glium().unwrap();
    let mut render_context = RenderContext::new(&display);
}
