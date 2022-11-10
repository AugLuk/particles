use std::cmp::Ordering::{Greater, Less};
use rand::Rng;
use rand_xoshiro::rand_core::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;
use crate::particle::Particle;
use crate::particle_type::{ConversionType, ParticleType};
use crate::{color, vec2};
use crate::vec2::Vec2;

const COLORS: [sdl2::pixels::Color; 32] = [
    sdl2::pixels::Color::RGB(255, 0, 0),
    sdl2::pixels::Color::RGB(255, 128, 0),
    sdl2::pixels::Color::RGB(255, 255, 0),
    sdl2::pixels::Color::RGB(0, 255, 0),
    sdl2::pixels::Color::RGB(0, 255, 255),
    sdl2::pixels::Color::RGB(0, 0, 255),
    sdl2::pixels::Color::RGB(128, 0, 255),
    sdl2::pixels::Color::RGB(255, 0, 255),

    sdl2::pixels::Color::RGB(128, 0, 0),
    sdl2::pixels::Color::RGB(128, 64, 0),
    sdl2::pixels::Color::RGB(128, 128, 0),
    sdl2::pixels::Color::RGB(0, 128, 0),
    sdl2::pixels::Color::RGB(0, 128, 128),
    sdl2::pixels::Color::RGB(0, 0, 128),
    sdl2::pixels::Color::RGB(64, 0, 128),
    sdl2::pixels::Color::RGB(128, 0, 128),

    sdl2::pixels::Color::RGB(255, 128, 128),
    sdl2::pixels::Color::RGB(255, 191, 128),
    sdl2::pixels::Color::RGB(255, 255, 128),
    sdl2::pixels::Color::RGB(128, 255, 128),
    sdl2::pixels::Color::RGB(128, 255, 255),
    sdl2::pixels::Color::RGB(128, 128, 255),
    sdl2::pixels::Color::RGB(191, 128, 255),
    sdl2::pixels::Color::RGB(255, 128, 255),

    sdl2::pixels::Color::RGB(128, 64, 64),
    sdl2::pixels::Color::RGB(128, 96, 64),
    sdl2::pixels::Color::RGB(128, 128, 64),
    sdl2::pixels::Color::RGB(64, 128, 64),
    sdl2::pixels::Color::RGB(64, 128, 128),
    sdl2::pixels::Color::RGB(64, 64, 128),
    sdl2::pixels::Color::RGB(96, 64, 128),
    sdl2::pixels::Color::RGB(128, 64, 128),
];

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
    pub particle_count: usize,
    pub touching_pushing_acc: f64,
    pub resistance: f64,
}

impl Board {
    pub fn new(particle_count: usize, type_count: usize, width: f64, height: f64, br_count_x: usize, br_count_y: usize, touching_pushing_acc: f64, resistance: f64, max_field_pulling_acc: f64, max_field_pushing_acc: f64, max_radius: f64, use_procedural_colors: bool, use_chemistry: bool, color_rng_seed: u64, rule_rng_seed: u64, initial_state_rng_seed: u64) -> Self {
        let br_width = width / br_count_x as f64;
        let br_height = height / br_count_y as f64;

        let mut rule_rng = Xoshiro256PlusPlus::seed_from_u64(rule_rng_seed);
        let mut initial_state_rng = Xoshiro256PlusPlus::seed_from_u64(initial_state_rng_seed);


        let initial_br_capacity = (particle_count as f64 / (br_count_x * br_count_y) as f64 * 2.0).round() as usize;
        let mut bounding_rects = Vec::with_capacity(br_count_x * br_count_y);
        for _ in 0..(br_count_x * br_count_y) {
            bounding_rects.push(Vec::with_capacity(initial_br_capacity))
        }

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

        // println!("Chemistry:");
        // println!("\tConverts_tos:\n\t\t{:?}", converts_tos);
        // println!("\tCatalysts:");
        // catalysts_of_type.iter().enumerate().for_each(|(i, v)| println!("\t\t{}: {:?}", i, v));

        let procedural_colors = if use_procedural_colors {
            color::get_procedural_colors(type_count, color_rng_seed)
        } else {
            vec![]
        };

        let mut particle_types = Vec::with_capacity(type_count);
        for i in 0..type_count {
            let ct = if converts_tos[i] == i || !use_chemistry {
                ConversionType::INERT
            } else {
                ConversionType::CONVERTS { converts_to: converts_tos[i], catalysts: catalysts_of_type[i].clone() }
            };

            let color = if use_procedural_colors {
                procedural_colors[i]
            } else {
                COLORS[i % 32]
            };

            particle_types.push(ParticleType::from_random(color, type_count, max_field_pulling_acc, max_field_pushing_acc, max_radius, ct, &mut rule_rng))
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

        Board { width, height, bounding_rects, br_count_x, br_count_y, br_width, br_height, particle_types, particle_count, touching_pushing_acc, resistance }
    }

    pub fn simulate(&mut self) {
        for by in 0..self.br_count_y {
            let by_plus = (by + 1).rem_euclid(self.br_count_y);

            for bx in 0..self.br_count_x {
                let bx_minus = (bx as isize - 1).rem_euclid(self.br_count_x as isize) as usize;
                let bx_plus = (bx + 1).rem_euclid(self.br_count_x);

                let foo = vec![
                    (by * self.br_count_x + bx_plus, self.br_width, 0.0),
                    (by_plus * self.br_count_x + bx_minus, -self.br_width, self.br_height),
                    (by_plus * self.br_count_x + bx, 0.0, self.br_height),
                    (by_plus * self.br_count_x + bx_plus, self.br_width, self.br_height),
                ];

                let this_br_idx = by * self.br_count_x + bx;

                let mut foo_before = vec![];
                let mut foo_after = vec![];
                for foo_e in foo.iter() {
                    let (idx, _, _) = foo_e;
                    if *idx < this_br_idx {
                        foo_before.push(*foo_e);
                    } else {
                        foo_after.push(*foo_e);
                    }
                }

                foo_before.sort_by(|(idx1, _, _), (idx2, _, _)| idx1.cmp(idx2));
                foo_after.sort_by(|(idx1, _, _), (idx2, _, _)| idx1.cmp(idx2));

                let mut last_idx = 0;
                for (i, foo_e) in foo_before.iter_mut().enumerate() {
                    let (idx, ox, oy) = foo_e;
                    let temp = last_idx;
                    last_idx = *idx;
                    *foo_e = (*idx - temp - if i == 0 { 0 } else { 1 }, *ox, *oy);
                }
                let mut last_idx = this_br_idx + 1;
                for (i, foo_e) in foo_after.iter_mut().enumerate() {
                    let (idx, ox, oy) = foo_e;
                    let temp = last_idx;
                    last_idx = *idx;
                    *foo_e = (*idx - temp - if i == 0 { 0 } else { 1 }, *ox, *oy);
                }

                let (brs_before, brs_temp) = self.bounding_rects.split_at_mut(this_br_idx);
                let (brs_temp, brs_after) = brs_temp.split_at_mut(1);
                let this_br = &mut brs_temp[0];

                let mut brs_before_iter = brs_before.iter_mut();
                let mut bar_before = vec![];
                for (idx, ox, oy) in foo_before.iter() {
                    bar_before.push((brs_before_iter.nth(*idx).unwrap().as_mut_slice(), *ox, *oy));
                }
                let mut brs_after_iter = brs_after.iter_mut();
                let mut bar_after = vec![];
                for (idx, ox, oy) in foo_after.iter() {
                    bar_after.push((brs_after_iter.nth(*idx).unwrap().as_mut_slice(), *ox, *oy));
                }


                let len = this_br.len();
                for pi in 0..len {
                    let mut this_br_iter = this_br.iter_mut().skip(pi);
                    let p = this_br_iter.next().unwrap();

                    for (ps, ox, oy) in bar_before.iter_mut().chain(bar_after.iter_mut()) {
                        for other_p in ps.iter_mut() {
                            Self::interact(p, other_p, *ox, *oy, &self.particle_types, self.touching_pushing_acc);
                        }
                    }

                    // bar_before.par_iter_mut().for_each(|(ps, ox, oy)| {
                    //     for other_p in ps.iter_mut() {
                    //         Self::calc_particle_acc(p, other_p, *ox, *oy, &self.particle_types, self.pushing_acc);
                    //     }
                    // });
                    //
                    // bar_after.par_iter_mut().for_each(|(ps, ox, oy)| {
                    //     for other_p in ps.iter_mut() {
                    //         Self::calc_particle_acc(p, other_p, *ox, *oy, &self.particle_types, self.pushing_acc);
                    //     }
                    // });

                    for other_p in this_br_iter {
                        Self::interact(p, other_p, 0.0, 0.0, &self.particle_types, self.touching_pushing_acc);
                    }

                    let vel_mag = p.vel.x.hypot(p.vel.y);
                    p.vel += Vec2::new(p.vel.x * vel_mag * -self.resistance, p.vel.y * vel_mag * -self.resistance);
                }
            }
        }

        for by in 0..self.br_count_y {
            for bx in 0..self.br_count_x {
                let this_br_idx = by * self.br_count_x + bx;

                let mut pi = 0;
                while pi < self.bounding_rects[this_br_idx].len() {
                    let mut p = &mut self.bounding_rects[this_br_idx][pi];


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


                    p.pos = p.pos + p.vel;

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