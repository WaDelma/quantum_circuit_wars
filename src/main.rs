#![allow(warnings)]
#[macro_use]
extern crate glium;
extern crate daggy;
extern crate rusttype;
extern crate unicode_normalization;
extern crate num;
extern crate nalgebra;
extern crate image;
extern crate arrayvec;
extern crate quantum_circuit_wars;

use glium::{DisplayBuild, Program};
use glium::glutin::WindowBuilder;

use num::{Zero, One};

use graphics::RenderContext;

use quantum_circuit_wars::circuit::{Game, port};
use quantum_circuit_wars::circuit::gate::{Input, Output, Not};

use std::cell::RefCell;

type Mat = nalgebra::Matrix4<f32>;
type Vect = nalgebra::Vector2<f32>;

mod events;
mod graphics;
mod math;

pub struct Node {
    pos: Vect,
    inputs: RefCell<Vec<Vect>>,
    outputs: RefCell<Vec<Vect>>,
}

impl Node {
    fn new(pos: Vect) -> Node {
        Node {
            pos: pos,
            inputs: RefCell::new(vec![]),
            outputs: RefCell::new(vec![]),
        }
    }
}

#[derive(Clone, Copy)]
enum GameState {
    Splash, Menu, Game, End,
}

fn main() {
    use self::graphics::rendered::render_splashscreen;
    let mut gv = Game::new();
    let a = gv.add(Input::new(), Node::new(Vect::zero()));
    let b = gv.add(Not::new(), Node::new(Vect::one()));
    gv.connect(port(a, 0), port(b, 0));
    let c = gv.add(Output::new(), Node::new(2. * Vect::one()));
    gv.connect(port(b, 0), port(c, 0));
    println!("Let the quantum circuit wars begin!");
    let display = WindowBuilder::new().build_glium().unwrap();
    let mut render_context = RenderContext::new(&display);
    let mut ctx = GameContext::new();
    while ctx.running {
        use self::GameState::*;
        let dims = display.get_framebuffer_dimensions();
        render_context.cam = math::matrix(
            [[ctx.zoom / dims.0 as f32, 0., 0., 0.],
             [0., ctx.zoom / dims.1 as f32, 0., 0.],
             [0., 0., 1., 0.],
             [0., 0., 0., 1.]]
        );

        if let Some(Splash) = ctx.state {
            render_splashscreen(&mut render_context);
        }
    }
}

pub struct GameContext {
    running: bool,
    zoom: f32,
    caret: usize,
    state: Option<GameState>,
    text: String,
    node_width: f32,
    port_size: f32,
}

impl GameContext {
    fn new() -> GameContext {
        GameContext {
            running: true,
            zoom: 200.,
            caret: 0,
            state: Some(GameState::Splash),
            text: String::new(),
            node_width: 1.,
            port_size: 0.1,
        }
    }
}
