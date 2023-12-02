use super::Mixer;
use crate as bns_rng;
use crate::{FromRng, Noise};

/// Implementation of voronoi noise.
#[derive(Debug, Clone, FromRng)]
pub struct Voronoi2 {
    x: Mixer<2>,
    y: Mixer<2>,
}

impl Voronoi2 {
    /// Computes the position of the point in the provided voronoi cell.
    pub fn voronoi_point(&self, [x, y]: [i32; 2]) -> [f32; 2] {
        [
            crate::utility::f32_from_u32_01(self.x.sample([x as u64, y as u64]) as u32),
            crate::utility::f32_from_u32_01(self.y.sample([x as u64, y as u64]) as u32),
        ]
    }
}

impl Noise<[f32; 2]> for Voronoi2 {
    type Output = [i32; 2];

    fn sample(&self, coords: [f32; 2]) -> Self::Output {
        let xi = crate::utility::floor_i32(coords[0]);
        let yi = crate::utility::floor_i32(coords[1]);
        let xf = coords[0] - xi as f32;
        let yf = coords[1] - yi as f32;

        let mut best_dist = f32::INFINITY;
        let mut best_point = [0, 0];

        for dy in -1..=1 {
            for dx in -1..=1 {
                let pt = self.voronoi_point([xi + dx, yi + dy]);
                let diff = [dx as f32 + pt[0] - xf, dy as f32 + pt[1] - yf];
                let sq_dist = diff[0] * diff[0] + diff[1] * diff[1];
                if sq_dist < best_dist {
                    best_dist = sq_dist;
                    best_point = [xi + dx, yi + dy];
                }
            }
        }

        best_point
    }
}
