extern crate glium;
extern crate daggy;
extern crate rusttype;
extern crate unicode_normalization;
extern crate nalgebra;
extern crate image;

mod render;

fn main() {
    println!("Hello, world!");

    let display = WindowBuilder::new().build_glium().unwrap();
}
