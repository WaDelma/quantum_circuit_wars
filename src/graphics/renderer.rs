use glium::{Frame, VertexBuffer, Blend, Surface};
use glium::Display;
use glium::index::{IndexBuffer, PrimitiveType};
use glium::draw_parameters::DrawParameters;
use glium::draw_parameters::LinearBlendingFactor::*;
use glium::draw_parameters::BlendingFunction::*;
use glium::uniforms::{Uniforms, UniformsStorage, AsUniformValue, MagnifySamplerFilter, MinifySamplerFilter};

use nalgebra::Norm;
use std::convert::AsRef;

use {GameContext, Node, Vect};
use super::{RenderContext, Vertex, vert};
use math::*;
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

pub fn render(display: &Display, rctx: &mut RenderContext, world: GameView<Node>, ctx: &GameContext) {
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
    for (_, data) in world.iter() {
        let pos = flip_y(data.pos);
        let corner_pos = pos - Vect::new(ctx.node_width, ctx.node_width) * 0.5;
        let matrix = rctx.cam * translation(corner_pos.x, corner_pos.y);
        let uniforms = uniform! {
            matrix: *matrix.as_ref(),
        };
        draw(&mut target, &rctx, "back", "plain", &uniforms, &draw_params);
        let matrix = rctx.cam * translation(corner_pos.x + 0.05, corner_pos.y + 0.05) * scale(0.9, 0.9);
        let program = rctx.programs.get("plain");
        let program = program.as_ref().expect("Node didn't have shader.");
        let uniforms = uniform! {
            matrix: *matrix.as_ref(),
        };
        let model = rctx.models.get("node").unwrap();
        target.draw(&model.vertices, &model.indices, &program, &uniforms, &draw_params).expect("Drawing node failed.");

        let mut draw = |things: &[_]| {
            for p in things {
                let p = pos + flip_y(*p) - Vect::new(ctx.port_size, ctx.port_size) * 0.5;
                let matrix = rctx.cam * translation(p.x, p.y) * scale(ctx.port_size, ctx.port_size);
                let uniforms = uniform! {
                    matrix: *matrix.as_ref(),
                };
                draw(&mut target, &rctx, "node", "plain", &uniforms, &draw_params);
            }
        };
        draw(&data.outputs.borrow());
        draw(&data.inputs.borrow());
    }
    let mut lines = Vec::with_capacity(world.connections());
    for (src, trg) in world.iter_connections() {
        let src = output_pos(&world, src, ctx.port_size);
        let trg = input_pos(&world, trg, ctx.port_size);
        let trg = Vect::new(trg[0], trg[1] + ctx.port_size);
        add_arrow(&mut lines, src, trg, 0.1, 0.1 * TAU);
    }
    let vertices = VertexBuffer::new(display, &lines).unwrap();
    let indices = (0..lines.len() as u32).collect::<Vec<_>>();
    let indices = IndexBuffer::new(display, PrimitiveType::LinesList, &indices).unwrap();
    let matrix = rctx.cam * translation(0., 0.);
    let uniforms = uniform! {
        matrix: *matrix.as_ref(),
    };
    let program = rctx.programs.get("plain").unwrap();
    target.draw(&vertices, &indices, program, &uniforms, &draw_params).unwrap();
    target.finish().unwrap();
}

pub fn render_splashscreen(display: &Display, render_context: &mut RenderContext) {
    use math::translation;
    let mut target = display.draw();
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
    let splash_matrix = render_context.cam * translation(0., 0.) * scale(5., 5.);
    let uniforms = uniform! {
        matrix: *splash_matrix.as_ref(),
        tex: texture.sampled()
            .magnify_filter(MagnifySamplerFilter::Nearest)
            .minify_filter(MinifySamplerFilter::Nearest),

    };
    target.clear_color(0.,0.,0.,1.);
    target.draw(&model.vertices, &model.indices, program, &uniforms, &draw_params);
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
