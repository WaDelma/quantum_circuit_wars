use glium::{Frame, VertexBuffer, Blend, Surface};
use glium::Display;
use glium::index::{IndexBuffer, PrimitiveType};
use glium::draw_parameters::DrawParameters;
use glium::uniforms::{Uniforms, UniformsStorage, AsUniformValue, MagnifySamplerFilter, MinifySamplerFilter};

use nalgebra::{Vector4, Dot, Norm, Iterable};
use num::{Complex, Zero};
use std::convert::AsRef;

use {GameContext, Node, Vect};
use super::{RenderContext, Vertex, vert};
use math::*;
use quantum_circuit_wars::*;
use quantum_circuit_wars::circuit::{GameView, Game, Port};

fn input_pos(gen: &Game<Node>, input: Port<u32>, _size: f32) -> Vect {
    let node = gen.get(input.node).unwrap();
    let pos = node.1.pos;
    let percent = (input.port + 1) as f32 / (node.0.max_in() + 1) as f32;
    Vect::new(pos[0] - 0.5 + percent, -(pos[1] - 0.5))
}

fn output_pos(gen: &Game<Node>, output: Port<u32>, size: f32) -> Vect {
    let node = gen.get(output.node).unwrap();
    let pos = node.1.pos;
    let percent = (output.port + 1) as f32 / (node.0.max_out() + 1) as f32;
    Vect::new(pos[0] - 0.5 + percent, -(pos[1] + 0.5 + size))
}

pub fn render(display: &Display, rctx: &mut RenderContext, world: GameView<Node>, ctx: &mut GameContext) {
    use glium::draw_parameters::LinearBlendingFactor::*;
    use glium::draw_parameters::BlendingFunction::*;
    use glium::glutin::Event::*;
    use glium::glutin::ElementState::*;
    use glium::glutin::MouseButton as Mouse;
    use glium::glutin::VirtualKeyCode as Key;
    use glium::glutin::MouseScrollDelta;
    for event in display.poll_events() {
        match event {
            Closed => ctx.running = false,
            KeyboardInput(Pressed, _, Some(Key::I)) => {
                rctx.boxes.push(super::ABox {
                    pos: Vect::new(-0.15, -0.5),
                    typ: super::Type::Not,
                });
            },
            KeyboardInput(Pressed, _, Some(Key::Q)) => {
                rctx.boxes.push(super::ABox {
                    pos: Vect::new(-0.15, 0.3),
                    typ: super::Type::Not,
                });
            },
            _ => {}
        }
    }
    let mut target = display.draw();
    target.clear_color(0.0157, 0.0173, 0.0204, 1.);
    let draw_params = DrawParameters {
        blend: Blend {
            color: Addition {
                source: SourceAlpha,
                destination: OneMinusSourceAlpha,
            },
            alpha: Addition {
                source: SourceAlpha,
                destination: OneMinusSourceAlpha,
            },
            constant_value: (0f32, 0f32, 0f32, 1f32),
        },
        smooth: None,
        ..Default::default()
    };
    let dims = display.get_framebuffer_dimensions();
    let mut lines = Vec::with_capacity(2);
    lines.push(vert(-0.5, -0.4));
    lines.push(vert(rctx.line_end, -0.4));
    lines.push(vert(-0.5, 0.4));
    lines.push(vert(rctx.line_end, 0.4));
    let vertices = VertexBuffer::new(display, &lines).unwrap();
    let indices = (0..lines.len() as u32).collect::<Vec<_>>();
    let indices = IndexBuffer::new(display, PrimitiveType::LinesList, &indices).unwrap();
    let matrix = translation(0., 0.);
    let uniforms = uniform! {
        matrix: *matrix.as_ref(),
    };
    let program = rctx.programs.get("plain").unwrap();
    target.draw(&vertices, &indices, program, &uniforms, &draw_params).unwrap();

    let matrix = translation(-0.725, -0.55) *
        scale(0.33, 0.33);
    let program = rctx.programs.get("texture");
    let program = program.as_ref().expect("Node didn't have shader.");
    let texture = rctx.textures.get("alice").unwrap();
    let uniforms = uniform! {
        matrix: *matrix.as_ref(),
        frame: rctx.alice_frame / 100,
        tex: texture.sampled()
            .magnify_filter(MagnifySamplerFilter::Nearest)
            .minify_filter(MinifySamplerFilter::Nearest),
    };
    let model = rctx.models.get("frame").unwrap();
    target.draw(&model.vertices, &model.indices, &program, &uniforms, &draw_params).expect("Drawing node failed.");
    rctx.alice_frame += 1;
    rctx.alice_frame %= 800;

    let matrix = translation(-0.725, 0.3) *
        scale(0.33, 0.33);
    let program = rctx.programs.get("texture");
    let program = program.as_ref().expect("Node didn't have shader.");
    let texture = rctx.textures.get("bob").unwrap();
    let uniforms = uniform! {
        matrix: *matrix.as_ref(),
        frame: rctx.bob_frame / 100,
        tex: texture.sampled()
            .magnify_filter(MagnifySamplerFilter::Nearest)
            .minify_filter(MinifySamplerFilter::Nearest),
    };
    let model = rctx.models.get("frame").unwrap();
    target.draw(&model.vertices, &model.indices, &program, &uniforms, &draw_params).expect("Drawing node failed.");
    rctx.bob_frame += 1;
    rctx.bob_frame %= 800;

    let mut to_be_removed = vec![];
    let mut is_end = rctx.level.is_empty();
    for (i, b) in rctx.boxes.iter_mut().enumerate() {
        b.pos.x -= 0.001;
        if b.pos.x < -0.70 {
            to_be_removed.push(i);
        }
        let mut matrix = translation(b.pos.x, b.pos.y) * if b.typ.is_big() {
            is_end = false;
            scale(0.15, 1.)
        } else {
            scale(0.15, 0.2)
        };
        let program = rctx.programs.get("plain");
        let program = program.as_ref().expect("Node didn't have shader.");
        let uniforms = uniform! {
            matrix: *matrix.as_ref(),
        };
        let model = rctx.models.get("node").unwrap();
        target.draw(&model.vertices, &model.indices, &program, &uniforms, &draw_params).expect("Drawing node failed.");
    }
    target.finish().unwrap();
    for i in to_be_removed.into_iter().rev() {
        if rctx.boxes[i].typ.is_big() {
            rctx.state = rctx.boxes[i].typ.mat() * rctx.state.clone();
            print!("{:?}: (", rctx.boxes[i].typ);
            for c in rctx.state.as_vector() {
                print!("{}, ", c);
            }
            println!(")");
        } else {
            if rctx.boxes[i].pos.y > 0. {
                rctx.state = apply_to_qubit(rctx.boxes[i].typ.mat(), 0, 2) * rctx.state.clone();
                print!("A: {:?}: (", rctx.boxes[i].typ);
                for c in rctx.state.as_vector() {
                    print!("{}, ", c);
                }
                println!(")");
            } else {
                rctx.state = apply_to_qubit(rctx.boxes[i].typ.mat(), 1, 2) * rctx.state.clone();
                print!("B: {:?}: (", rctx.boxes[i].typ);
                for c in rctx.state.as_vector() {
                    print!("{}, ", c);
                }
                println!(")");
            }
        }
        let state = Vector4::new(rctx.state[(0, 0)], rctx.state[(1, 0)], rctx.state[(2, 0)], rctx.state[(3, 0)]);
        println!("A score: {}", calc_score(state.clone(), rctx.goal_a.clone()));
        println!("B score: {}", calc_score(state, rctx.goal_b.clone()));
        rctx.boxes.remove(i);
    }
    rctx.frame += 1;

    if rctx.frame > rctx.big_block_target {
        if let Some(b) = rctx.level.pop() {
            rctx.boxes.push(b.1);
            if let Some(b) = rctx.level.last() {
                rctx.big_block_target = rctx.frame + b.0;
            }
        }
    }

    if is_end {
        rctx.line_end -= 0.001;
        if rctx.line_end < -0.5 {
            panic!();
        }
    }
}

fn calc_score(state: Vector4<Complex<f64>>, goal: Vector4<Complex<f64>>) -> f64 {
    state.iter()
        .zip(goal.iter())
        .map(|(a, b)| a * b)
        .fold(Complex::zero(), |a, b| a + b).norm_sqr()
}

pub fn render_splashscreen(display: &Display, render_context: &mut RenderContext) {
    use glium::draw_parameters::LinearBlendingFactor::*;
    use glium::draw_parameters::BlendingFunction::*;
    use math::translation;
    let mut target = display.draw();
    target.clear_color(0., 0., 0., 1.);
    let draw_params = DrawParameters {
        blend: Blend {
            color: Addition {
                source: SourceAlpha,
                destination: OneMinusSourceAlpha,
            },
            alpha: Addition {
                source: SourceAlpha,
                destination: OneMinusSourceAlpha,
            },
            constant_value: (0f32, 0f32, 0f32, 1f32),
        },
        smooth: None,
        ..Default::default()
    };
    let texture = render_context.textures.get("splash").unwrap();
    let model = render_context.models.get("node").unwrap();
    let program = render_context.programs.get("texture").unwrap();
    let splash_matrix =
        translation(-1., -1.) *
        scale(2., 2.);
    let uniforms = uniform! {
        matrix: *splash_matrix.as_ref(),
        tex: texture.sampled()
            .magnify_filter(MagnifySamplerFilter::Nearest)
            .minify_filter(MinifySamplerFilter::Nearest),

    };
    target.draw(&model.vertices, &model.indices, program, &uniforms, &draw_params);

    let string = "Press ANY-key to continue!";
    render_context.fonts.draw_text(display, &mut target, "press_start_2p", 20., [1., 0., 0., 1.], Vect::new(0.45, -1.8), string);
    target.finish().unwrap();
}

fn draw<A, B>(target: &mut Frame, rctx: &RenderContext, model: &str, program: &str, uniforms: &UniformsStorage<A, B>, draw_params: &DrawParameters)
    where A: AsUniformValue,
          B: Uniforms,
{
    let model = rctx.models.get(model).unwrap();
    let program = rctx.programs.get(program).unwrap();
    target.draw(&model.vertices, &model.indices, program, uniforms, draw_params).unwrap();
}

fn add_arrow(lines: &mut Vec<Vertex>, src: Vect, trg: Vect, len: f32, theta: f32) {
    lines.push(vert(src.x, src.y));
    lines.push(vert(trg.x, trg.y));
    let vec = (src - trg).normalize();
    let mut add_part = |theta: f32| {
        let (sin, cos) = theta.sin_cos();
        let arrow = Vect::new(
            vec.x * cos - vec.y * sin,
            vec.x * sin + vec.y * cos);
        lines.push(vert(trg.x, trg.y));
        let v = trg + (arrow * len);
        lines.push(vert(v.x, v.y));
    };
    add_part(theta);
    add_part(-theta);
}
