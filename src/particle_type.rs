#[derive(Debug, Clone)]
pub struct ParticleType {
    pub color: sdl2::pixels::Color,
    pub accelerations_of_pairs: Vec<Vec<f64>>,
    pub radii_of_pairs: Vec<Vec<f64>>,
    pub conversion_type: ConversionType,
}

#[derive(Debug, Clone)]
pub enum ConversionType {
    CONVERTS { converts_to: usize, catalysts: Vec<bool>},
    INERT,
}

impl ParticleType {
    pub fn new(color: sdl2::pixels::Color, type_count: usize, max_pulling_acc: f64, max_pushing_acc: f64, max_radius: f64, conversion_type: ConversionType, rng: &mut impl rand::Rng) -> Self {
        let mut accelerations_of_pairs = Vec::with_capacity(type_count);
        let mut radii_of_pairs = Vec::with_capacity(type_count);


        for _ in 0..type_count {
            let node_count = rng.gen_range(2..=5);

            let mut accelerations = Vec::with_capacity(node_count);
            accelerations.push(rng.gen_range(-max_pushing_acc..0.0));
            accelerations.push(rng.gen_range(-max_pushing_acc..0.0));
            for _ in 0..(node_count - 2) {
                accelerations.push(rng.gen_range(-max_pushing_acc..max_pulling_acc));
            }

            let mut radii = Vec::with_capacity(node_count);
            for _ in 0..node_count {
                radii.push(rng.gen_range(0.0..max_radius));
            }
            radii.sort_unstable_by(|a  , b| a.partial_cmp(b).unwrap());

            accelerations_of_pairs.push(accelerations);
            radii_of_pairs.push(radii);
        }

        ParticleType { color, accelerations_of_pairs, radii_of_pairs, conversion_type, }
    }
}