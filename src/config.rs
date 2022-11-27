use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub image_width: usize,
    pub image_height: usize,

    pub color_rng_seed: Option<u64>,

    pub save_frames_to_path: Option<String>,

    pub iterations_per_frame: usize,

    pub rule_rng_seed: Option<u64>,
    pub generate_chemistry: bool,

    pub initial_state_rng_seed: Option<u64>,

    pub particle_count: usize,
    pub type_count: usize,

    pub board_width: f64,
    pub board_height: f64,

    pub bounding_rect_cols: usize,
    pub bounding_rect_rows: usize,

    pub touching_pushing_acc: f64,
    pub resistance: f64,
    pub max_field_pulling_acc: f64,
    pub max_field_pushing_acc: f64,
    pub max_radius: f64,
}