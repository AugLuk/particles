mod config;
mod vec2;
mod particle;
mod particle_type;
mod board;

use std::{fs, path};
use std::fs::OpenOptions;
use std::io::BufWriter;
use rand::{Rng, thread_rng};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::rect::Rect;
use crate::board::Board;
use crate::config::Config;

fn main() -> Result<(), String> {
    // ---------------------------------------------------------------------------------------------
    // Read the config file
    // ---------------------------------------------------------------------------------------------

    let config_str = fs::read_to_string("config.ron").expect("Error while reading the configuration file.");
    let config: Config = ron::from_str(&config_str).expect("Error while reading the configuration file.");

    // ---------------------------------------------------------------------------------------------
    // Initialize the simulation state
    // ---------------------------------------------------------------------------------------------

    let procedural_color_rng_seed = match config.color_rng_seed {
        Some(v) => v,
        None => {
            let v = thread_rng().gen();
            println!("Color rng seed = {}", v);
            v
        },
    };

    let rule_rng_seed = match config.rule_rng_seed {
        Some(v) => v,
        None => {
            let v = thread_rng().gen();
            println!("Rule rng seed = {}", v);
            v
        },
    };

    let initial_state_rng_seed = match config.initial_state_rng_seed {
        Some(v) => v,
        None => {
            let v = thread_rng().gen();
            println!("Initial state rng seed = {}", v);
            v
        },
    };

    let mut board = Board::new(
        config.particle_count,
        config.type_count,
        config.board_width,
        config.board_height,
        config.bounding_rect_cols,
        config.bounding_rect_rows,
        config.touching_pushing_acc,
        config.resistance,
        config.max_field_pulling_acc,
        config.max_field_pushing_acc,
        config.max_radius,
        config.generate_chemistry,
        procedural_color_rng_seed,
        rule_rng_seed,
        initial_state_rng_seed,
    );

    // ---------------------------------------------------------------------------------------------
    // SDL2 setup
    // ---------------------------------------------------------------------------------------------

    let sdl_context = sdl2::init()?;
    let video_subsys = sdl_context.video()?;

    let window = video_subsys
        .window(
            "Simulated Annealing Matrix",
            (config.image_width) as u32,
            (config.image_height) as u32,
        )
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;

    let mut event_pump = sdl_context.event_pump()?;

    canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
    canvas.clear();

    // ---------------------------------------------------------------------------------------------
    // Main loop
    // ---------------------------------------------------------------------------------------------

    let mut draw_continuously = true;
    let mut draw_once = false;
    let mut simulate_continuously = true;
    let mut simulate_once = false;
    let mut running = true;
    let mut frame_idx: usize = 0;
    while running {
        for event in event_pump.poll_iter() {
            match event {
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    draw_continuously = !draw_continuously;
                    simulate_continuously = !simulate_continuously;
                },
                Event::KeyDown {
                    keycode: Some(Keycode::F),
                    ..
                } => {
                    draw_once = true;
                    simulate_once = true;
                },
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    running = false;
                }
                _ => {}
            }
        }

        if simulate_continuously || simulate_once {
            simulate_once = false;

            for _ in 0..config.iterations_per_frame {
                board.simulate();
            }
        }

        if draw_continuously || draw_once {
            draw_once = false;

            canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
            canvas.clear();

            for by in 0..board.br_count_y {
                let oy = by as f64 * board.br_height;

                for bx in 0..board.br_count_x {
                    let ox = bx as f64 * board.br_width;

                    let br = &board.bounding_rects[by * board.br_count_x + bx];
                    for p in br.iter() {
                        let px = ((p.pos.x + ox) * (config.image_width as f64 / board.width)).round() as i16;
                        let py = ((p.pos.y + oy) * (config.image_height as f64 / board.height)).round() as i16;
                        let r = (config.image_height as f64 / board.height / 2.0).round() as i16;
                        let color = board.particle_types[p.type_idx].color;

                        let _ = canvas.filled_circle(px, py, r, color);
                    }
                }
            }

            match &config.save_frames_to_path {
                Some(path) => {
                    let img_data = canvas.read_pixels(Rect::new(0, 0, config.image_width as u32, config.image_height as u32), sdl2::pixels::PixelFormatEnum::RGB888).unwrap();

                    let path_string = format!("{}/frame_{:0>4}.png", path, frame_idx);
                    let path = path::Path::new(&path_string);
                    let prefix = path.parent().unwrap();
                    fs::create_dir_all(prefix).unwrap();
                    let file = OpenOptions::new()
                        .write(true)
                        .create(true)
                        .open(path)
                        .unwrap();
                    let ref mut w = BufWriter::new(file);

                    let mut encoder = png::Encoder::new(w, config.image_width as u32, config.image_height as u32);
                    encoder.set_color(png::ColorType::Rgb);
                    encoder.set_depth(png::BitDepth::Eight);
                    let mut writer = encoder.write_header().unwrap();

                    let img_data = img_data.chunks(4).flat_map(|chunk| [chunk[2], chunk[1], chunk[0]]).collect::<Vec<_>>();

                    writer.write_image_data(&img_data).unwrap();
                },
                None => {},
            }

            frame_idx += 1;
        }

        canvas.present();
    }

    Ok(())
}