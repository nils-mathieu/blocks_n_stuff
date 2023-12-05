// Shamelessly stolen from https://github.com/Razaekel/noise-rs/blob/d79aa83cc5bab27ccab3c82cc9265add0bbeaa46/src/core/super_simplex.rs

#![allow(clippy::excessive_precision)]

use bns_rng_derive::FromRng;

use super::Mixer;
use crate::utility::floor_i32;
use crate::Noise;

use crate as bns_rng;

const TO_REAL_CONSTANT_2D: f32 = -0.211_324_865_405_187; // (1 / sqrt(2 + 1) - 1) / 2
const TO_SIMPLEX_CONSTANT_2D: f32 = 0.366_025_403_784_439; // (sqrt(2 + 1) - 1) / 2
const TO_SIMPLEX_CONSTANT_3D: f32 = -2.0 / 3.0;

// Determined using the Mathematica code listed in the super_simplex example and find_maximum_super_simplex.nb
const NORM_CONSTANT_2D: f32 = 1.0 / 0.054_282_952_886_616_23;
const NORM_CONSTANT_3D: f32 = 1.0 / 0.086_766_400_165_536_9;

// Points taken into account for 2D:
//             (0, -1)
//                |    \
//                |      \
//                |        \
// (-1, 0) --- ( 0,  0) --- ( 1,  0)
//        \       |    \       |    \
//          \     |      \     |      \
//            \   |        \   |        \
//             ( 0,  1) --- ( 1,  1) --- ( 2,  1)
//                     \       |
//                       \     |
//                         \   |
//                          ( 1,  2)
#[rustfmt::skip]
const LATTICE_LOOKUP_2D: [([i8; 2], [f32; 2]); 4 * 8] =
    [([0, 0], [0.0, 0.0]),
     ([1, 1], [-0.577_350_269_189_626, -0.577_350_269_189_626]),
     ([-1, 0], [0.788_675_134_594_813, -0.211_324_865_405_187]),
     ([0, -1], [-0.211_324_865_405_187, 0.788_675_134_594_813]),

     ([0, 0], [0.0, 0.0]),
     ([1, 1], [-0.577_350_269_189_626, -0.577_350_269_189_626]),
     ([0, 1], [0.211_324_865_405_187, -0.788_675_134_594_813]),
     ([1, 0], [-0.788_675_134_594_813, 0.211_324_865_405_187]),

     ([0, 0], [0.0, 0.0]),
     ([1, 1], [-0.577_350_269_189_626, -0.577_350_269_189_626]),
     ([1, 0], [-0.788_675_134_594_813, 0.211_324_865_405_187]),
     ([0, -1], [-0.211_324_865_405_187, 0.788_675_134_594_813]),

     ([0, 0], [0.0, 0.0]),
     ([1, 1], [-0.577_350_269_189_626, -0.577_350_269_189_626]),
     ([2, 1], [-1.366_025_403_784_439, -0.366_025_403_784_439_04]),
     ([1, 0], [-0.788_675_134_594_813, 0.211_324_865_405_187]),

     ([0, 0], [0.0, 0.0]),
     ([1, 1], [-0.577_350_269_189_626, -0.577_350_269_189_626]),
     ([-1, 0], [0.788_675_134_594_813, -0.211_324_865_405_187]),
     ([0, 1], [0.211_324_865_405_187, -0.788_675_134_594_813]),

     ([0, 0], [0.0, 0.0]),
     ([1, 1], [-0.577_350_269_189_626, -0.577_350_269_189_626]),
     ([0, 1], [0.211_324_865_405_187, -0.788_675_134_594_813]),
     ([1, 2], [-0.366_025_403_784_439_04, -1.366_025_403_784_439]),

     ([0, 0], [0.0, 0.0]),
     ([1, 1], [-0.577_350_269_189_626, -0.577_350_269_189_626]),
     ([1, 0], [-0.788_675_134_594_813, 0.211_324_865_405_187]),
     ([0, 1], [0.211_324_865_405_187, -0.788_675_134_594_813]),

     ([0, 0], [0.0, 0.0]),
     ([1, 1], [-0.577_350_269_189_626, -0.577_350_269_189_626]),
     ([2, 1], [-1.366_025_403_784_439, -0.366_025_403_784_439_04]),
     ([1, 2], [-0.366_025_403_784_439_04, -1.366_025_403_784_439])];

#[rustfmt::skip]
const LATTICE_LOOKUP_3D: [[i8; 3]; 4 * 16] =
    [[0, 0, 0],[1, 0, 0],[0, 1, 0],[0, 0, 1],
     [1, 1, 1],[1, 0, 0],[0, 1, 0],[0, 0, 1],
     [0, 0, 0],[0, 1, 1],[0, 1, 0],[0, 0, 1],
     [1, 1, 1],[0, 1, 1],[0, 1, 0],[0, 0, 1],
     [0, 0, 0],[1, 0, 0],[1, 0, 1],[0, 0, 1],
     [1, 1, 1],[1, 0, 0],[1, 0, 1],[0, 0, 1],
     [0, 0, 0],[0, 1, 1],[1, 0, 1],[0, 0, 1],
     [1, 1, 1],[0, 1, 1],[1, 0, 1],[0, 0, 1],
     [0, 0, 0],[1, 0, 0],[0, 1, 0],[1, 1, 0],
     [1, 1, 1],[1, 0, 0],[0, 1, 0],[1, 1, 0],
     [0, 0, 0],[0, 1, 1],[0, 1, 0],[1, 1, 0],
     [1, 1, 1],[0, 1, 1],[0, 1, 0],[1, 1, 0],
     [0, 0, 0],[1, 0, 0],[1, 0, 1],[1, 1, 0],
     [1, 1, 1],[1, 0, 0],[1, 0, 1],[1, 1, 0],
     [0, 0, 0],[0, 1, 1],[1, 0, 1],[1, 1, 0],
     [1, 1, 1],[0, 1, 1],[1, 0, 1],[1, 1, 0]];

/// 2D super simplex noise implementation.
#[derive(FromRng, Clone, Debug)]
pub struct SuperSimplex2 {
    mixer: Mixer<2>,
}

impl Noise<[f32; 2]> for SuperSimplex2 {
    type Output = f32;

    fn sample(&self, point: [f32; 2]) -> Self::Output {
        // Transform point from real space to simplex space
        let to_simplex_offset = (point[0] + point[1]) * TO_SIMPLEX_CONSTANT_2D;
        let simplex_point = [point[0] + to_simplex_offset, point[1] + to_simplex_offset];

        // Get base point of simplex and barycentric coordinates in simplex space
        let simplex_base_point_i = [floor_i32(simplex_point[0]), floor_i32(simplex_point[1])];
        let simplex_base_point = [
            simplex_base_point_i[0] as f32,
            simplex_base_point_i[1] as f32,
        ];
        let simplex_rel_coords = [
            simplex_point[0] - simplex_base_point[0],
            simplex_point[1] - simplex_base_point[1],
        ];

        // Create index to lookup table from barycentric coordinates
        let region_sum = (simplex_rel_coords[0] + simplex_rel_coords[1]).floor();
        let index = ((region_sum >= 1.0) as usize) << 2
            | ((simplex_rel_coords[0] - simplex_rel_coords[1] * 0.5 + 1.0 - region_sum * 0.5 >= 1.0)
                as usize)
                << 3
            | ((simplex_rel_coords[1] - simplex_rel_coords[0] * 0.5 + 1.0 - region_sum * 0.5 >= 1.0)
                as usize)
                << 4;

        // Transform barycentric coordinates to real space
        let to_real_offset = (simplex_rel_coords[0] + simplex_rel_coords[1]) * TO_REAL_CONSTANT_2D;
        let real_rel_coords = simplex_rel_coords.map(|v| v + to_real_offset);

        let mut value = 0.0;

        for lattice_lookup in &LATTICE_LOOKUP_2D[index..index + 4] {
            let dpos = [
                real_rel_coords[0] + lattice_lookup.1[0],
                real_rel_coords[1] + lattice_lookup.1[1],
            ];
            let attn = (2.0 / 3.0) - (dpos[0] * dpos[0] + dpos[1] * dpos[1]);
            if attn > 0.0 {
                let lattice_point = [
                    simplex_base_point_i[0] + lattice_lookup.0[0] as i32,
                    simplex_base_point_i[1] + lattice_lookup.0[1] as i32,
                ];
                let gradient = grad2(
                    self.mixer
                        .sample([lattice_point[0] as u64, lattice_point[1] as u64])
                        as usize,
                );
                value += attn.powi(4) * (gradient[0] * dpos[0] + gradient[1] * dpos[1]);
            }
        }

        value * NORM_CONSTANT_2D
    }
}

#[inline(always)]
fn grad2(index: usize) -> [f32; 2] {
    const DIAG: f32 = std::f32::consts::FRAC_1_SQRT_2;

    const VALUES: [[f32; 2]; 8] = [
        [1.0, 0.0],
        [-1.0, 0.0],
        [0.0, 1.0],
        [0.0, -1.0],
        [DIAG, DIAG],
        [-DIAG, DIAG],
        [DIAG, -DIAG],
        [-DIAG, -DIAG],
    ];

    unsafe { *VALUES.get_unchecked(index % 8) }
}

/// 3D super simplex noise implementation.
#[derive(FromRng, Clone, Debug)]
pub struct SuperSimplex3 {
    mixer: Mixer<3>,
}

impl Noise<[f32; 3]> for SuperSimplex3 {
    type Output = f32;

    fn sample(&self, point: [f32; 3]) -> Self::Output {
        // Transform point from real space to simplex space
        let to_simplex_offset = (point[0] + point[1] + point[2]) * TO_SIMPLEX_CONSTANT_3D;
        let simplex_point = point.map(|v| -(v + to_simplex_offset));
        let second_simplex_point = [
            simplex_point[0] + 512.5,
            simplex_point[1] + 512.5,
            simplex_point[2] + 512.5,
        ];

        // Get base point of simplex and barycentric coordinates in simplex space
        let simplex_base_point_i = [
            floor_i32(simplex_point[0]),
            floor_i32(simplex_point[1]),
            floor_i32(simplex_point[2]),
        ];
        let simplex_base_point = [
            simplex_base_point_i[0] as f32,
            simplex_base_point_i[1] as f32,
            simplex_base_point_i[2] as f32,
        ];
        let simplex_rel_coords = [
            simplex_point[0] - simplex_base_point[0],
            simplex_point[1] - simplex_base_point[1],
            simplex_point[2] - simplex_base_point[2],
        ];
        let second_simplex_base_point_i = [
            floor_i32(second_simplex_point[0]),
            floor_i32(second_simplex_point[1]),
            floor_i32(second_simplex_point[2]),
        ];
        let second_simplex_base_point = [
            second_simplex_base_point_i[0] as f32,
            second_simplex_base_point_i[1] as f32,
            second_simplex_base_point_i[2] as f32,
        ];
        let second_simplex_rel_coords = [
            second_simplex_point[0] - second_simplex_base_point[0],
            second_simplex_point[1] - second_simplex_base_point[1],
            second_simplex_point[2] - second_simplex_base_point[2],
        ];

        // Create indices to lookup table from barycentric coordinates
        let index = ((simplex_rel_coords[0] + simplex_rel_coords[1] + simplex_rel_coords[2] >= 1.5)
            as usize)
            << 2
            | ((-simplex_rel_coords[0] + simplex_rel_coords[1] + simplex_rel_coords[2] >= 0.5)
                as usize)
                << 3
            | ((simplex_rel_coords[0] - simplex_rel_coords[1] + simplex_rel_coords[2] >= 0.5)
                as usize)
                << 4
            | ((simplex_rel_coords[0] + simplex_rel_coords[1] - simplex_rel_coords[2] >= 0.5)
                as usize)
                << 5;
        let second_index = ((second_simplex_rel_coords[0]
            + second_simplex_rel_coords[1]
            + second_simplex_rel_coords[2]
            >= 1.5) as usize)
            << 2
            | ((-second_simplex_rel_coords[0]
                + second_simplex_rel_coords[1]
                + second_simplex_rel_coords[2]
                >= 0.5) as usize)
                << 3
            | ((second_simplex_rel_coords[0] - second_simplex_rel_coords[1]
                + second_simplex_rel_coords[2]
                >= 0.5) as usize)
                << 4
            | ((second_simplex_rel_coords[0] + second_simplex_rel_coords[1]
                - second_simplex_rel_coords[2]
                >= 0.5) as usize)
                << 5;

        let mut value = 0.0;

        // Sum contributions from first lattice
        for &lattice_lookup in &LATTICE_LOOKUP_3D[index..index + 4] {
            let dpos = [
                simplex_rel_coords[0] - lattice_lookup[0] as f32,
                simplex_rel_coords[1] - lattice_lookup[1] as f32,
                simplex_rel_coords[2] - lattice_lookup[2] as f32,
            ];
            let attn = 0.75 - (dpos[0] * dpos[0] + dpos[1] * dpos[1] + dpos[2] * dpos[2]);
            if attn > 0.0 {
                let lattice_point = [
                    (simplex_base_point_i[0] + lattice_lookup[0] as i32) as u64,
                    (simplex_base_point_i[1] + lattice_lookup[1] as i32) as u64,
                    (simplex_base_point_i[2] + lattice_lookup[2] as i32) as u64,
                ];
                let gradient = grad3(self.mixer.sample(lattice_point) as usize);
                value += attn.powi(4)
                    * (gradient[0] * dpos[0] + gradient[1] * dpos[1] + gradient[2] * dpos[2]);
            }
        }

        // Sum contributions from second lattice
        for &lattice_lookup in &LATTICE_LOOKUP_3D[second_index..second_index + 4] {
            let dpos = [
                second_simplex_rel_coords[0] - lattice_lookup[0] as f32,
                second_simplex_rel_coords[1] - lattice_lookup[1] as f32,
                second_simplex_rel_coords[2] - lattice_lookup[2] as f32,
            ];

            let attn = 0.75 - (dpos[0] * dpos[0] + dpos[1] * dpos[1] + dpos[2] * dpos[2]);
            if attn > 0.0 {
                let lattice_point = [
                    (second_simplex_base_point_i[0] + lattice_lookup[0] as i32) as u64,
                    (second_simplex_base_point_i[1] + lattice_lookup[1] as i32) as u64,
                    (second_simplex_base_point_i[2] + lattice_lookup[2] as i32) as u64,
                ];
                let gradient = grad3(self.mixer.sample(lattice_point) as usize);
                value += attn.powi(4)
                    * (gradient[0] * dpos[0] + gradient[1] * dpos[1] + gradient[2] * dpos[2]);
            }
        }

        value * NORM_CONSTANT_3D
    }
}

#[inline(always)]
fn grad3(mut index: usize) -> [f32; 3] {
    const DIAG: f32 = core::f32::consts::FRAC_1_SQRT_2;
    const DIAG2: f32 = 0.577_350_269_189_625_8;

    const VALUES: [[f32; 3]; 20] = [
        // 12 edges repeated twice
        // indices 0-11 and 12-23
        [DIAG, DIAG, 0.0],
        [-DIAG, DIAG, 0.0],
        [DIAG, -DIAG, 0.0],
        [-DIAG, -DIAG, 0.0],
        [DIAG, 0.0, DIAG],
        [-DIAG, 0.0, DIAG],
        [DIAG, 0.0, -DIAG],
        [-DIAG, 0.0, -DIAG],
        [0.0, DIAG, DIAG],
        [0.0, -DIAG, DIAG],
        [0.0, DIAG, -DIAG],
        [0.0, -DIAG, -DIAG],
        // Corners
        // indices 24-31
        [DIAG2, DIAG2, DIAG2],
        [-DIAG2, DIAG2, DIAG2],
        [DIAG2, -DIAG2, DIAG2],
        [-DIAG2, -DIAG2, DIAG2],
        [DIAG2, DIAG2, -DIAG2],
        [-DIAG2, DIAG2, -DIAG2],
        [DIAG2, -DIAG2, -DIAG2],
        [-DIAG2, -DIAG2, -DIAG2],
    ];

    index %= 32;

    if index < 12 {
        unsafe { *VALUES.get_unchecked(index) }
    } else if index < 24 {
        unsafe { *VALUES.get_unchecked(index - 12) }
    } else {
        unsafe { *VALUES.get_unchecked(index - 24 + 12) }
    }
}
