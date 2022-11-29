use std::cmp::Ordering::{Greater, Less};
use rand::Rng;
use rand_xoshiro::rand_core::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;
use rand::distributions::Uniform;
use crate::particle::Particle;
use crate::particle_type::{ConversionType, ParticleType};
use crate::vec2;
use crate::vec2::Vec2;

#[derive(Debug, Clone)]
pub struct Board {
    pub width: f64,
    pub height: f64,
    pub bounding_rects: Vec<Vec<Particle>>,
    pub br_count_x: usize,
    pub br_count_y: usize,
    pub br_width: f64,
    pub br_height: f64,
    pub particle_types: Vec<ParticleType>,
    pub touching_pushing_acc: f64,
    pub resistance: f64,
}

impl Board {
    pub fn new(
        particle_count: usize,
        type_count: usize,
        width: f64,
        height: f64,
        br_count_x: usize,
        br_count_y: usize,
        touching_pushing_acc: f64,
        resistance: f64,
        max_field_pulling_acc: f64,
        max_field_pushing_acc: f64,
        max_radius: f64,
        generate_chemistry: bool,
        color_rng_seed: u64,
        rule_rng_seed: u64,
        initial_state_rng_seed: u64,
    ) -> Self {
        let mut rule_rng = Xoshiro256PlusPlus::seed_from_u64(rule_rng_seed);

        let (converts_tos, catalysts_of_type) = if generate_chemistry {
            new_chemistry(type_count, &mut rule_rng)
        } else {
            (vec![], vec![])
        };

        let colors = get_colors(type_count, color_rng_seed);

        let mut particle_types = Vec::with_capacity(type_count);
        for i in 0..type_count {

            let ct = if generate_chemistry && converts_tos[i] != i {
                ConversionType::CONVERTS { converts_to: converts_tos[i], catalysts: catalysts_of_type[i].clone() }
            } else {
                ConversionType::INERT
            };

            let color = colors[i];

            let particle_type = ParticleType::new(color, type_count, max_field_pulling_acc, max_field_pushing_acc, max_radius, ct, &mut rule_rng);

            particle_types.push(particle_type)
        }

        let br_width = width / br_count_x as f64;
        let br_height = height / br_count_y as f64;

        let mut initial_state_rng = Xoshiro256PlusPlus::seed_from_u64(initial_state_rng_seed);

        let initial_br_capacity = (particle_count as f64 / (br_count_x * br_count_y) as f64 * 2.0).round() as usize;
        let mut bounding_rects = Vec::with_capacity(br_count_x * br_count_y);
        for _ in 0..(br_count_x * br_count_y) {
            bounding_rects.push(Vec::with_capacity(initial_br_capacity))
        }

        for _ in 0..particle_count {
            let x = initial_state_rng.gen_range(0.0..width);
            let y = initial_state_rng.gen_range(0.0..height);
            let t = initial_state_rng.gen_range(0..type_count);

            let can_convert = match &particle_types[t].conversion_type {
                ConversionType::INERT => false,
                ConversionType::CONVERTS { converts_to: _, catalysts: _ } => true,
            };

            let p = Particle::new(
                t,
                can_convert,
                Vec2::new(x % br_width as f64, y % br_height as f64),
                vec2::ZERO,
            );

            let br_col = (x / br_width).floor() as usize;
            let br_row = (y / br_height).floor() as usize;

            bounding_rects[br_row * br_count_x + br_col].push(p);
        }

        Board { width, height, bounding_rects, br_count_x, br_count_y, br_width, br_height, particle_types, touching_pushing_acc, resistance }
    }

    pub fn simulate(&mut self) {
        for br_y in 0..self.br_count_y {
            let br_y_plus = (br_y + 1).rem_euclid(self.br_count_y);

            for br_x in 0..self.br_count_x {
                let br_x_minus = (br_x as isize - 1).rem_euclid(self.br_count_x as isize) as usize;
                let br_x_plus = (br_x + 1).rem_euclid(self.br_count_x);

                let br_idxs_and_p_offsets = vec![
                    (br_y * self.br_count_x + br_x_plus, self.br_width, 0.0),
                    (br_y_plus * self.br_count_x + br_x_minus, -self.br_width, self.br_height),
                    (br_y_plus * self.br_count_x + br_x, 0.0, self.br_height),
                    (br_y_plus * self.br_count_x + br_x_plus, self.br_width, self.br_height),
                ];

                let this_br_idx = br_y * self.br_count_x + br_x;

                let mut br_idxs_and_p_offsets_before_this = vec![];
                let mut br_idxs_and_p_offsets_after_this = vec![];
                for idx_and_offset in br_idxs_and_p_offsets.iter() {
                    let (idx, _, _) = idx_and_offset;
                    if *idx < this_br_idx {
                        br_idxs_and_p_offsets_before_this.push(*idx_and_offset);
                    } else {
                        br_idxs_and_p_offsets_after_this.push(*idx_and_offset);
                    }
                }

                br_idxs_and_p_offsets_before_this.sort_by(|(idx1, _, _), (idx2, _, _)| idx1.cmp(idx2));
                br_idxs_and_p_offsets_after_this.sort_by(|(idx1, _, _), (idx2, _, _)| idx1.cmp(idx2));

                let (brs_before, brs_temp) = self.bounding_rects.split_at_mut(this_br_idx);
                let (brs_temp, brs_after) = brs_temp.split_at_mut(1);
                let this_br = &mut brs_temp[0];

                let mut brs_and_p_offsets_before_this = vec![];
                let mut brs_before_iter = brs_before.iter_mut();
                let mut last_idx = 0;
                for (i, (idx, ox, oy)) in br_idxs_and_p_offsets_before_this.iter().enumerate() {
                    let temp = last_idx;
                    last_idx = *idx;
                    let nth = *idx - temp - if i == 0 { 0 } else { 1 };
                    brs_and_p_offsets_before_this.push((brs_before_iter.nth(nth).unwrap().as_mut_slice(), *ox, *oy));
                }

                let mut brs_and_p_offsets_after_this = vec![];
                let mut brs_after_iter = brs_after.iter_mut();
                let mut last_idx = this_br_idx + 1;
                for (i, (idx, ox, oy)) in br_idxs_and_p_offsets_after_this.iter().enumerate() {
                    let temp = last_idx;
                    last_idx = *idx;
                    let nth = *idx - temp - if i == 0 { 0 } else { 1 };
                    brs_and_p_offsets_after_this.push((brs_after_iter.nth(nth).unwrap().as_mut_slice(), *ox, *oy));
                }

                for p_idx in 0..this_br.len() {
                    let mut this_br_iter = this_br.iter_mut().skip(p_idx);
                    let p = this_br_iter.next().unwrap();

                    for (ps, ox, oy) in brs_and_p_offsets_before_this.iter_mut().chain(brs_and_p_offsets_after_this.iter_mut()) {
                        for other_p in ps.iter_mut() {
                            Self::interact(p, other_p, *ox, *oy, &self.particle_types, self.touching_pushing_acc);
                        }
                    }

                    for other_p in this_br_iter {
                        Self::interact(p, other_p, 0.0, 0.0, &self.particle_types, self.touching_pushing_acc);
                    }
                }
            }
        }

        self.bounding_rects.iter_mut().for_each(|br| {
            for p in br.iter_mut() {
                let vel_mag = p.vel.x.hypot(p.vel.y);
                p.vel += Vec2::new(p.vel.x * vel_mag * -self.resistance, p.vel.y * vel_mag * -self.resistance);

                p.pos = p.pos + p.vel;

                if !p.can_convert {
                    let ct = &self.particle_types[p.type_idx].conversion_type;
                    match *ct {
                        ConversionType::CONVERTS { converts_to, catalysts: _ } => {
                            p.type_idx = converts_to;
                            p.can_convert = true;
                        },
                        ConversionType::INERT => {},
                    };
                }
            }
        });

        for by in 0..self.br_count_y {
            for bx in 0..self.br_count_x {
                let this_br_idx = by * self.br_count_x + bx;

                let mut pi = 0;
                while pi < self.bounding_rects[this_br_idx].len() {
                    let mut p = &mut self.bounding_rects[this_br_idx][pi];

                    let br_ox = if p.pos.x < 0.0 {
                        p.pos.x = p.pos.x + self.br_width;
                        -1
                    } else if p.pos.x >= self.br_width {
                        p.pos.x = p.pos.x - self.br_width;
                        1
                    } else {
                        0
                    };

                    let br_oy = if p.pos.y < 0.0 {
                        p.pos.y = p.pos.y + self.br_height;
                        -1
                    } else if p.pos.y >= self.br_height {
                        p.pos.y = p.pos.y - self.br_height;
                        1
                    } else {
                        0
                    };

                    if br_ox == 0 && br_oy == 0 {
                        pi += 1;
                        continue;
                    }

                    let new_bx = (bx as isize + br_ox).rem_euclid(self.br_count_x as isize) as usize;
                    let new_by = (by as isize + br_oy).rem_euclid(self.br_count_y as isize) as usize;

                    let p = self.bounding_rects[this_br_idx].swap_remove(pi);
                    self.bounding_rects[new_by * self.br_count_x + new_bx].push(p);
                }
            }
        }
    }

    fn interact(p1: &mut Particle, p2: &mut Particle, ox: f64, oy: f64, particle_types: &Vec<ParticleType>, pushing_acc: f64) {
        let pox = p2.pos.x + ox - p1.pos.x;
        let poy = p2.pos.y + oy - p1.pos.y;
        let dist = (pox).hypot(poy);


        if dist < 1.0 {
            // chemistry

            // p1
            if p1.can_convert {
                let ct = &particle_types[p1.type_idx].conversion_type;
                match ct {
                    ConversionType::INERT => {},
                    ConversionType::CONVERTS { converts_to: _, catalysts } => {
                        if catalysts[p2.type_idx] {
                            p1.can_convert = false;
                        }
                    }
                }
            }

            // p2
            if p2.can_convert {
                let ct = &particle_types[p2.type_idx].conversion_type;
                match ct {
                    ConversionType::INERT => {},
                    ConversionType::CONVERTS { converts_to: _, catalysts } => {
                        if catalysts[p1.type_idx] {
                            p2.can_convert = false;
                        }
                    }
                }
            }


            // acceleration due to touching particles pushing each other
            let acc_coef = 1.0 - dist;
            let this_acc = Vec2::new(pox / dist * acc_coef * -pushing_acc, poy / dist * acc_coef * -pushing_acc);
            p1.vel += this_acc;
            p2.vel -= this_acc;
        }


        // acceleration due to the field attraction/pushing between particles

        // p1
        let accelerations = &particle_types[p1.type_idx].accelerations_of_pairs[p2.type_idx];
        let radii = &particle_types[p1.type_idx].radii_of_pairs[p2.type_idx];
        let idx = radii.binary_search_by(|probe| if *probe > dist { Greater } else { Less }).unwrap_err();
        if idx < radii.len() {
            let pos_left = if idx == 0 {
                0.0
            } else {
                radii[idx - 1]
            };
            let pos_right = radii[idx];

            let inc = (dist - pos_left) / (pos_right - pos_left);

            let acc_left = accelerations[idx];
            let acc_right = *accelerations.get(idx + 1).unwrap_or(&0.0);

            let acc = acc_left * (1.0 - inc) + acc_right * inc;

            p1.vel += Vec2::new(pox / dist * acc, poy / dist * acc);
        }

        // p2
        let accelerations = &particle_types[p2.type_idx].accelerations_of_pairs[p1.type_idx];
        let radii = &particle_types[p2.type_idx].radii_of_pairs[p1.type_idx];
        let idx = radii.binary_search_by(|probe| if *probe > dist { Greater } else { Less }).unwrap_err();
        if idx < radii.len() {
            let pos_left = if idx == 0 {
                0.0
            } else {
                radii[idx - 1]
            };
            let pos_right = radii[idx];

            let inc = (dist - pos_left) / (pos_right - pos_left);

            let acc_left = accelerations[idx];
            let acc_right = *accelerations.get(idx + 1).unwrap_or(&0.0);

            let acc = acc_left * (1.0 - inc) + acc_right * inc;

            p2.vel += Vec2::new(-pox / dist * acc, -poy / dist * acc);
        }
    }
}

fn new_chemistry(type_count: usize, rule_rng: &mut impl Rng) -> (Vec<usize>, Vec<Vec<bool>>) {
    let mut converts_tos = vec![None; type_count];
    {
        let mut i = 0;
        let mut i2 = 0;
        let mut self_available = true;
        let mut blank_types_left = type_count;
        while i < type_count {
            let mut until_target = rule_rng.gen_range(if self_available {
                0..blank_types_left
            } else {
                0..(blank_types_left - 1)
            });

            //println!("{}", until_target);

            if until_target == 0 {
                converts_tos[i2] = Some(i);
                //println!("v[{}] = {}", i2, i);
                while converts_tos.get(i).is_some() && converts_tos[i].is_some() {
                    i += 1;
                }
                i2 = i;
                self_available = true;
                blank_types_left -= 1;
                continue;
            }

            until_target -= 1;

            let mut target_idx = i + 1;
            loop {
                if converts_tos[target_idx].is_none() && target_idx != i2 {
                    if until_target == 0 {
                        break;
                    } else {
                        until_target -= 1;
                        target_idx += 1;
                    }
                } else {
                    target_idx += 1;
                }
            }

            converts_tos[i2] = Some(target_idx);
            //println!("v[{}] = {}", i2, target_idx);
            i2 = target_idx;

            blank_types_left -= 1;
        }
    }
    let converts_tos = converts_tos.iter().map(|v| v.unwrap()).collect::<Vec<_>>();

    let mut catalysts_of_type = Vec::with_capacity(type_count);
    for j in 0..type_count {
        if converts_tos[j] == j {
            catalysts_of_type.push(Vec::new());
            continue;
        }

        let mut catalysts = vec![false; type_count];

        let mut has_catalyst = false;
        while !has_catalyst {
            for i in 0..type_count {
                let val = rule_rng.gen::<bool>();
                if val {
                    has_catalyst = true;
                }
                catalysts[i] = val;
            }
        }

        catalysts_of_type.push(catalysts);
    }

    (converts_tos, catalysts_of_type)
}

fn get_colors(count: usize, rng_seed: u64) -> Vec<sdl2::pixels::Color> {
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