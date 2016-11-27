use glium::{Frame, VertexBuffer, Blend, Surface};
use glium::Display;
use glium::index::{IndexBuffer, PrimitiveType};
use glium::draw_parameters::DrawParameters;
use glium::uniforms::{Uniforms, UniformsStorage, AsUniformValue, MagnifySamplerFilter, MinifySamplerFilter};

use nalgebra::{Vector4, Dot, Norm, Iterable};
use num::{Complex, Zero};
use std::convert::AsRef;
use baal;

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

fn upper(typ: super::Type) -> super::ABox {
    super::ABox {
        pos: Vect::new(-0.15, 0.3),
        typ: typ,
    }
}

fn lower(typ: super::Type) -> super::ABox {
    super::ABox {
        pos: Vect::new(-0.15, -0.5),
        typ: typ,
    }
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
            KeyboardInput(Pressed, _, Some(Key::Q)) => {
                rctx.boxes.push(upper(super::Type::Not));
            },
            KeyboardInput(Pressed, _, Some(Key::A)) => {
                rctx.boxes.push(upper(super::Type::H));
            },
            KeyboardInput(Pressed, _, Some(Key::W)) => {
                rctx.boxes.push(upper(super::Type::Y));
            },
            KeyboardInput(Pressed, _, Some(Key::S)) => {
                rctx.boxes.push(upper(super::Type::Z));
            },
            KeyboardInput(Pressed, _, Some(Key::I)) => {
                rctx.boxes.push(lower(super::Type::Not));
            },
            KeyboardInput(Pressed, _, Some(Key::K)) => {
                rctx.boxes.push(lower(super::Type::H));
            },
            KeyboardInput(Pressed, _, Some(Key::O)) => {
                rctx.boxes.push(lower(super::Type::Y));
            },
            KeyboardInput(Pressed, _, Some(Key::L)) => {
                rctx.boxes.push(lower(super::Type::Z));
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
    let texture = rctx.textures.get("bg").unwrap();
    let model = rctx.models.get("node").unwrap();
    let program = rctx.programs.get("texture").unwrap();
    let splash_matrix =
        translation(-1., -1.) *
        scale(2., 2.);
    let uniforms = uniform! {
        matrix: *splash_matrix.as_ref(),
        tex: texture.sampled(),
        frame: (rctx.frame as i32 / 100) % 8,
    };
    target.draw(&model.vertices, &model.indices, program, &uniforms, &draw_params);

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
        if b.typ.tex().is_empty() {
            let program = rctx.programs.get("plain");
            let program = program.as_ref().expect("Node didn't have shader.");
            let uniforms = uniform! {
                matrix: *matrix.as_ref(),
            };
            let model = rctx.models.get("node").unwrap();
            target.draw(&model.vertices, &model.indices, &program, &uniforms, &draw_params).expect("Drawing node failed.");
        } else {
            let program = rctx.programs.get("texture");
            let program = program.as_ref().expect("Node didn't have shader.");
            let texture = rctx.textures.get(b.typ.tex()).unwrap();
            let uniforms = uniform! {
                matrix: *matrix.as_ref(),
                tex: texture.sampled(),
                frame: 0,
            };
            let model = rctx.models.get("node").unwrap();
            target.draw(&model.vertices, &model.indices, &program, &uniforms, &draw_params).expect("Drawing node failed.");
        }
    }
    let state = Vector4::new(rctx.state[(0, 0)], rctx.state[(1, 0)], rctx.state[(2, 0)], rctx.state[(3, 0)]);
    rctx.score_a = calc_score(state.clone(), rctx.goal_a.clone());
    rctx.score_b = calc_score(state, rctx.goal_b.clone());

    let string = format!("Alice: {}", round(rctx.score_a, 4));
    rctx.fonts.draw_text(display, &mut target, "press_start_2p", 20., [1., 0., 0., 1.], Vect::new(0.45, -1.8), &string);

    let goal_a = rctx.goal_a.clone();
    let string = format!("{} {}i", round(goal_a[0].re, 2), round(goal_a[0].im, 2));
    rctx.fonts.draw_text(display, &mut target, "press_start_2p", 20., [1., 0., 0., 1.], Vect::new(1.7, -0.4), &string);
    let string = format!("{} {}i", round(goal_a[1].re, 2), round(goal_a[1].im, 2));
    rctx.fonts.draw_text(display, &mut target, "press_start_2p", 20., [1., 0., 0., 1.], Vect::new(1.7, -0.5), &string);
    let string = format!("{} {}i", round(goal_a[2].re, 2), round(goal_a[2].im, 2));
    rctx.fonts.draw_text(display, &mut target, "press_start_2p", 20., [1., 0., 0., 1.], Vect::new(1.7, -0.6), &string);
    let string = format!("{} {}i", round(goal_a[3].re, 2), round(goal_a[3].im, 2));
    rctx.fonts.draw_text(display, &mut target, "press_start_2p", 20., [1., 0., 0., 1.], Vect::new(1.7, -0.7), &string);

    let string = format!("Bob: {}", round(rctx.score_b, 4));
    rctx.fonts.draw_text(display, &mut target, "press_start_2p", 20., [1., 0., 0., 1.], Vect::new(0.45, 0.), &string);

    let goal_b = rctx.goal_b.clone();
    let string = format!("{} {}i", round(goal_b[0].re, 2), round(goal_b[0].im, 2));
    rctx.fonts.draw_text(display, &mut target, "press_start_2p", 20., [1., 0., 0., 1.], Vect::new(1.7, -1.2), &string);
    let string = format!("{} {}i", round(goal_b[1].re, 2), round(goal_b[1].im, 2));
    rctx.fonts.draw_text(display, &mut target, "press_start_2p", 20., [1., 0., 0., 1.], Vect::new(1.7, -1.3), &string);
    let string = format!("{} {}i", round(goal_b[2].re, 2), round(goal_b[2].im, 2));
    rctx.fonts.draw_text(display, &mut target, "press_start_2p", 20., [1., 0., 0., 1.], Vect::new(1.7, -1.4), &string);
    let string = format!("{} {}i", round(goal_b[3].re, 2), round(goal_b[3].im, 2));
    rctx.fonts.draw_text(display, &mut target, "press_start_2p", 20., [1., 0., 0., 1.], Vect::new(1.7, -1.5), &string);

    let string = format!("{} {}i", round(state[0].re, 2), round(state[0].im, 2));
    rctx.fonts.draw_text(display, &mut target, "press_start_2p", 20., [1., 0., 0., 1.], Vect::new(0., -0.8), &string);
    let string = format!("{} {}i", round(state[1].re, 2), round(state[1].im, 2));
    rctx.fonts.draw_text(display, &mut target, "press_start_2p", 20., [1., 0., 0., 1.], Vect::new(0., -0.9), &string);
    let string = format!("{} {}i", round(state[2].re, 2), round(state[2].im, 2));
    rctx.fonts.draw_text(display, &mut target, "press_start_2p", 20., [1., 0., 0., 1.], Vect::new(0., -1.), &string);
    let string = format!("{} {}i", round(state[3].re, 2), round(state[3].im, 2));
    rctx.fonts.draw_text(display, &mut target, "press_start_2p", 20., [1., 0., 0., 1.], Vect::new(0., -1.1), &string);

    target.finish().unwrap();
    for i in to_be_removed.into_iter().rev() {
        if rctx.boxes[i].typ.is_big() {
            rctx.state = rctx.boxes[i].typ.mat() * rctx.state.clone();
        } else {
            if rctx.boxes[i].pos.y > 0. {
                rctx.state = apply_to_qubit(rctx.boxes[i].typ.mat(), 0, 2) * rctx.state.clone();
            } else {
                rctx.state = apply_to_qubit(rctx.boxes[i].typ.mat(), 1, 2) * rctx.state.clone();
            }
        }
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
            baal::effect::short::play_on_listener(1);
            ctx.state = Some(::GameState::End);
        }
    }
}

fn round(x: f64, digits: i32) -> f64 {
    let digits = 10f64.powi(digits);
    (x * digits).round() / digits
}

#[test]
fn round_test() {
    assert_eq!(6.28, round(6.2831, 2));
    assert_eq!(0.12346, round(0.123456789, 5));
}

fn calc_score(state: Vector4<Complex<f64>>, goal: Vector4<Complex<f64>>) -> f64 {
    state.iter()
        .zip(goal.iter())
        .map(|(a, b)| a * b)
        .fold(Complex::zero(), |a, b| a + b).norm_sqr()
}

pub fn render_splashscreen(display: &Display, render_context: &mut RenderContext, ctx: &mut GameContext) {
    use glium::glutin::Event::*;
    use glium::glutin::ElementState::*;
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
    };
    target.draw(&model.vertices, &model.indices, program, &uniforms, &draw_params);

    let string = "Press ANY-key to continue!";
    render_context.fonts.draw_text(display, &mut target, "press_start_2p", 20., [1., 0., 0., 1.], Vect::new(0.45, -1.8), string);
    target.finish().unwrap();

    for event in display.poll_events() {
        match event {
            Closed => ctx.running = false,
            KeyboardInput(Pressed, _, _) => {
                ctx.timer = 0;
                ctx.state = Some(::GameState::Lore)
            },
            _ => {}
        }
    }
}

pub fn render_lorescreen(display: &Display, render_context: &mut RenderContext, ctx: &mut GameContext) {
    use glium::glutin::Event::*;
    use glium::glutin::ElementState::*;
    let mut target = display.draw();
    target.clear_color(0., 0., 0., 1.);
    render_context.fonts.draw_text(display, &mut target, "press_start_2p", 18., [1., 0., 0., 1.], Vect::new(0.025, -0.5),
        "     After years of playing quantum games together      ");
    render_context.fonts.draw_text(display, &mut target, "press_start_2p", 18., [1., 0., 0., 1.], Vect::new(0.025, -0.7),
        "Alice and Bob have finally grown fed up with each other.");
    render_context.fonts.draw_text(display, &mut target, "press_start_2p", 18., [1., 0., 0., 1.], Vect::new(0.025, -0.9),
        "               To resolve their conflict                ");
    render_context.fonts.draw_text(display, &mut target, "press_start_2p", 18., [1., 0., 0., 1.], Vect::new(0.025, -1.1),
        "        they have agreed to play one final game,        ");
    render_context.fonts.draw_text(display, &mut target, "press_start_2p", 18., [1., 0., 0., 1.], Vect::new(0.025, -1.3),
        "               betting their very lives.                ");
    render_context.fonts.draw_text(display, &mut target, "press_start_2p", 18., [1., 0., 0., 1.], Vect::new(0.025, -1.5),
        "                There can be only one.                  ");
    render_context.fonts.draw_text(display, &mut target, "press_start_2p", 18., [1., 0., 0., 1.], Vect::new(0.025, -1.7),
        "         Let the quantum circuit wars begin..!          ");
    target.finish().unwrap();
    for event in display.poll_events() {
        match event {
            Closed => ctx.running = false,
            KeyboardInput(Pressed, _, _) => {
                ctx.timer = 0;
                ctx.state = Some(::GameState::Menu)
            },
            _ => {}
        }
    }
}

pub fn render_introscreen(display: &Display, render_context: &mut RenderContext, ctx: &mut GameContext) {
    use glium::glutin::Event::*;
    use glium::glutin::ElementState::*;
    let mut target = display.draw();
    target.clear_color(0., 0., 0., 1.);
    render_context.fonts.draw_text(display, &mut target, "press_start_2p", 18., [1., 0., 0., 1.], Vect::new(0.025, -0.5),
        "Alice can use gates X, Y, Z and H with keys Q, W, S, A.");
    render_context.fonts.draw_text(display, &mut target, "press_start_2p", 18., [1., 0., 0., 1.], Vect::new(0.025, -0.7),
        "Bob can use gates X, Y, Z and H with keys I, O, L, K.");
    render_context.fonts.draw_text(display, &mut target, "press_start_2p", 18., [1., 0., 0., 1.], Vect::new(0.025, -0.9),
        "Alice target is ({0}, {1}, {1}, {0})*sqrt(2)");
    render_context.fonts.draw_text(display, &mut target, "press_start_2p", 18., [1., 0., 0., 1.], Vect::new(0.025, -1.1),
        "Bob target is ({0}, {1}, {-1}, {0})*sqrt(2)");
    render_context.fonts.draw_text(display, &mut target, "press_start_2p", 18., [1., 0., 0., 1.], Vect::new(0.025, -1.3),
        "Starting state is ({1}, {0}, {0}, {1})*sqrt(2)");
    render_context.fonts.draw_text(display, &mut target, "press_start_2p", 18., [1., 0., 0., 1.], Vect::new(0.025, -1.5),
        "Alice controls the top qubit and Bob controls the bottom one.");
    target.finish().unwrap();
    for event in display.poll_events() {
        match event {
            Closed => ctx.running = false,
            KeyboardInput(Pressed, _, _) => {
                ctx.timer = 0;
                ctx.state = Some(::GameState::Game)
            },
            _ => {}
        }
    }
}

pub fn render_endscreen(display: &Display, render_context: &mut RenderContext, ctx: &mut GameContext) {
    use glium::glutin::Event::*;
    use glium::glutin::ElementState::*;
    let mut target = display.draw();
    target.clear_color(0., 0., 0., 1.);
    if render_context.score_a != render_context.score_b {
        if render_context.score_a > render_context.score_b {
            render_context.fonts.draw_text(display, &mut target, "press_start_2p", 30., [1., 0., 0., 1.], Vect::new(0.025, -0.9),
                "ALICE");
        } else {
            render_context.fonts.draw_text(display, &mut target, "press_start_2p", 30., [1., 0., 0., 1.], Vect::new(0.025, -0.9),
                "BOB");
        }
        render_context.fonts.draw_text(display, &mut target, "press_start_2p", 30., [1., 0., 0., 1.], Vect::new(0.025, -1.2),
            "IS THE WINNER!");

        if render_context.score_a > render_context.score_b {
            render_context.fonts.draw_text(display, &mut target, "press_start_2p", 30., [1., 0., 0., 1.], Vect::new(0.025, -1.5),
                "BOB");
        } else {
            render_context.fonts.draw_text(display, &mut target, "press_start_2p", 30., [1., 0., 0., 1.], Vect::new(0.025, -1.5),
                "ALICE");
        }
        render_context.fonts.draw_text(display, &mut target, "press_start_2p", 30., [1., 0., 0., 1.], Vect::new(0.025, -1.8),
            "IS NO MORE!");
    } else {
        render_context.fonts.draw_text(display, &mut target, "press_start_2p", 30., [1., 0., 0., 1.], Vect::new(0.025, -1.5),
            "NEITHER ALICE NOR BOB");
        render_context.fonts.draw_text(display, &mut target, "press_start_2p", 30., [1., 0., 0., 1.], Vect::new(0.025, -1.8),
            "MADE IT!");
    }

    target.finish().unwrap();
    for event in display.poll_events() {
        match event {
            Closed => ctx.running = false,
            KeyboardInput(Pressed, _, _) => {
                ctx.timer = 0;
                ctx.state = Some(::GameState::Game)
            },
            _ => {}
        }
    }
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
