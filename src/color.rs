use rand::{Rng, SeedableRng};
use rand::distributions::Uniform;
use rand_xoshiro::Xoshiro256PlusPlus;

pub fn get_procedural_colors(count: usize, rng_seed: u64) -> Vec<sdl2::pixels::Color> {
    let min_delta = 0.8 / (count as f64).sqrt();

    let uv_dist = Uniform::new_inclusive(-1.0, 1.0);
    let y_dist = Uniform::new_inclusive(0.4, 1.0);

    let mut rng = Xoshiro256PlusPlus::seed_from_u64(rng_seed);

    let mut colors = Vec::with_capacity(count);
    'outer:
    loop {
        let mut yuv = Vec::with_capacity(count);

        for _ in 0..count {
            let ru: f64 = rng.sample(uv_dist);
            let rv: f64 = rng.sample(&uv_dist);

            let u = smoothstep_inverse(ru.abs().powf(2.2)) * ru.signum() * 0.436;
            let v = smoothstep_inverse(rv.abs().powf(2.2)) * rv.signum() * 0.615;

            let y: f64 = rng.sample(&y_dist);

            yuv.push((y, u, v));
        }

        for i in 0..count {
            let (y, u, v) = yuv[i];

            for j in (i + 1)..count {
                let (y2, u2, v2) = yuv[j];

                if (y - y2).powi(2) + (u - u2).powi(2) + (v - v2).powi(2) < min_delta.powi(2) {
                    continue 'outer;
                }
            }
        }

        for &(y, u, v) in yuv.iter() {
            let r = y + 1.28033 * v;
            let g = y + -0.21482 * u + -0.38059 * v;
            let b = y + 2.12798 * u;

            let r = (r.clamp(0.0, 1.0) * 255.0).round() as u8;
            let g = (g.clamp(0.0, 1.0) * 255.0).round() as u8;
            let b = (b.clamp(0.0, 1.0) * 255.0).round() as u8;

            colors.push(sdl2::pixels::Color::RGB(r, g, b));
        }

        break;
    }

    colors
}

fn smoothstep_inverse(x: f64) -> f64 {
    0.5 - ((1.0 - 2.0 * x).asin() / 3.0).sin()
}