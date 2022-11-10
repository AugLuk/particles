mod vec2;
mod particle;
mod particle_type;
mod board;
mod color;

use std::{fs, path};
use std::fs::OpenOptions;
use std::io::BufWriter;
use configparser::ini::Ini;
use rand::Rng;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::rect::Rect;
use crate::board::Board;

fn main() -> Result<(), String> {
    // println!("Running setup...");

    // ---------------------------------------------------------------------------------------------
    // Read the config file
    // ---------------------------------------------------------------------------------------------

    let config_str = fs::read_to_string("config.ini").expect("Error while reading the configuration file.");
    let mut config = Ini::new();
    let _ = config.read(config_str);

    let image_width = config.getuint("default", "image_width").unwrap().unwrap() as usize;
    let image_height = config.getuint("default", "image_height").unwrap().unwrap() as usize;

    let use_procedural_colors = config.getbool("default", "use_procedural_colors").unwrap().unwrap();
    let color_rng_seed = config.getuint("default", "color_rng_seed").unwrap().unwrap();

    let save_frames = config.getbool("default", "save_frames").unwrap().unwrap();

    let iterations_per_frame = config.getuint("default", "iterations_per_frame").unwrap().unwrap() as usize;

    let use_random_rule_rng_seed = config.getbool("default", "use_random_rule_rng_seed").unwrap().unwrap();
    let rule_rng_seed = config.getuint("default", "rule_rng_seed").unwrap().unwrap();
    let initial_state_rng_seed = config.getuint("default", "initial_state_rng_seed").unwrap().unwrap();

    let particle_count = config.getuint("default", "particle_count").unwrap().unwrap() as usize;
    let type_count = config.getuint("default", "type_count").unwrap().unwrap() as usize;

    let board_width = config.getfloat("default", "board_width").unwrap().unwrap();
    let board_height = config.getfloat("default", "board_height").unwrap().unwrap();

    let bounding_rect_cols = config.getuint("default", "bounding_rect_cols").unwrap().unwrap() as usize;
    let bounding_rect_rows = config.getuint("default", "bounding_rect_rows").unwrap().unwrap() as usize;

    let touching_pushing_acc = config.getfloat("default", "touching_pushing_acc").unwrap().unwrap();
    let resistance = config.getfloat("default", "resistance").unwrap().unwrap();
    let max_field_pulling_acc = config.getfloat("default", "max_field_pulling_acc").unwrap().unwrap();
    let max_field_pushing_acc = config.getfloat("default", "max_field_pushing_acc").unwrap().unwrap();
    let max_radius = config.getfloat("default", "max_radius").unwrap().unwrap();

    let use_chemistry = config.getbool("default", "use_chemistry").unwrap().unwrap();


    // ---------------------------------------------------------------------------------------------
    // Initialize the simulation state
    // ---------------------------------------------------------------------------------------------

    let rule_rng_seed = if use_random_rule_rng_seed {
        let val = rand::thread_rng().gen();
        println!("rule_rng_seed = {}", val);
        val
    } else {
        rule_rng_seed
    };

    let mut board = Board::new(particle_count, type_count, board_width, board_height, bounding_rect_cols, bounding_rect_rows, touching_pushing_acc, resistance, max_field_pulling_acc, max_field_pushing_acc, max_radius, use_procedural_colors, use_chemistry, color_rng_seed, rule_rng_seed, initial_state_rng_seed);

    // ---------------------------------------------------------------------------------------------
    // SDL2 setup
    // ---------------------------------------------------------------------------------------------

    let sdl_context = sdl2::init()?;
    let video_subsys = sdl_context.video()?;

    let window = video_subsys
        .window(
            "Simulated Annealing Matrix",
            (image_width) as u32,
            (image_height) as u32,
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

    // println!("Setup complete.");

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

            for _ in 0..iterations_per_frame {
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
                        let px = ((p.pos.x + ox) * (image_width as f64 / board.width)).round() as i16;
                        let py = ((p.pos.y + oy) * (image_height as f64 / board.height)).round() as i16;
                        let r = (image_height as f64 / board.height / 2.0).round() as i16;
                        let color = board.particle_types[p.type_idx].color;

                        let _ = canvas.filled_circle(px, py, r, color);
                    }
                }
            }

            if save_frames {
                let img_data = canvas.read_pixels(Rect::new(0, 0, image_width as u32, image_height as u32), sdl2::pixels::PixelFormatEnum::RGB888).unwrap();

                let path_string = format!("output/frame_{:0>4}.png", frame_idx);
                let path = path::Path::new(&path_string);
                let prefix = path.parent().unwrap();
                fs::create_dir_all(prefix).unwrap();
                let file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(path)
                    .unwrap();
                let ref mut w = BufWriter::new(file);

                let mut encoder = png::Encoder::new(w, image_width as u32, image_height as u32);
                encoder.set_color(png::ColorType::Rgb);
                encoder.set_depth(png::BitDepth::Eight);
                let mut writer = encoder.write_header().unwrap();

                let img_data= img_data.chunks(4).flat_map(|chunk| [chunk[2], chunk[1], chunk[0]]).collect::<Vec<_>>();

                writer.write_image_data(&img_data).unwrap();
            }

            frame_idx += 1;
        }

        canvas.present();
    }

    // ---------------------------------------------------------------------------------------------
    // Ending tasks
    // ---------------------------------------------------------------------------------------------

    Ok(())
}