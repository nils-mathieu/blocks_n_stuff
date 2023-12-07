use bitflags::bitflags;
use bns_core::BlockFlags;
use glam::{IVec3, Vec3, Vec3A};

use crate::world::World;

/// A hit result for a swept AABB collision.
///
/// See [`Aabb::sweep`].
#[derive(Debug)]
struct SweepHit {
    /// The time of entry.
    entry_time: f32,
    /// The direction in which the sweep hit the collider.
    direction: Hit,
}

/// An axis-aligned bounded box.
struct Aabb {
    /// The minimum point of the box.
    min: Vec3A,
    /// The maximum point of the box.
    max: Vec3A,
}

impl Aabb {
    /// Returns the broadphase of this AABB with the given delta.
    pub fn broadphase(&self, delta: Vec3A) -> Self {
        Self {
            min: self.min.min(self.min + delta),
            max: self.max.max(self.max + delta),
        }
    }

    /// Sweeps `self` against `other` with the given delta.
    #[rustfmt::skip]
    pub fn sweep(&self, delta: Vec3A, other: &Self) -> Option<SweepHit> {
        // Find the distance between the near/far planes of the two boxes.

        let max_minus_min = other.max - self.min;
        let min_minus_max = other.min - self.max;

        // Select between the two vectors based on the sign of the delta, per axis.
        // NOTE: this could be optimized using SIMD, but I'm not sure how portable this is.
        // I kinda hope that the compiler will do it for me, but it's a bit unlikely.
        let inv_entry = Vec3A::new(
            if delta.x > 0.0 { min_minus_max.x } else { max_minus_min.x },
            if delta.y > 0.0 { min_minus_max.y } else { max_minus_min.y },
            if delta.z > 0.0 { min_minus_max.z } else { max_minus_min.z },
        );
        let inv_exit = Vec3A::new(
            if delta.x > 0.0 { max_minus_min.x } else { min_minus_max.x },
            if delta.y > 0.0 { max_minus_min.y } else { min_minus_max.y },
            if delta.z > 0.0 { max_minus_min.z } else { min_minus_max.z },
        );

        // Find the time of entry and exit for each axis.
        // Same this as above, I hope it gets vectorized.
        let mut entry = Vec3A::new(
            if delta.x == 0.0 { -f32::INFINITY } else { inv_entry.x / delta.x },
            if delta.y == 0.0 { -f32::INFINITY } else { inv_entry.y / delta.y },
            if delta.z == 0.0 { -f32::INFINITY } else { inv_entry.z / delta.z },
        );
        let exit = Vec3A::new(
            if delta.x == 0.0 { f32::INFINITY } else { inv_exit.x / delta.x },
            if delta.y == 0.0 { f32::INFINITY } else { inv_exit.y / delta.y },
            if delta.z == 0.0 { f32::INFINITY } else { inv_exit.z / delta.z },
        );

        if entry.x > 1.0 {
            entry.x = -f32::INFINITY;
        }
        if entry.y > 1.0 {
            entry.y = -f32::INFINITY;
        }
        if entry.z > 1.0 {
            entry.z = -f32::INFINITY;
        }

        // Check if any of the entry times are greater to any of the exit times.
        // If so, then there is no collision.
        let entry_time = entry.max_element();
        let exit_time = exit.min_element();

        if entry_time > exit_time {
            return None;
        }
        if entry.cmplt(Vec3A::ZERO).all() {
            return None;
        }
        if entry.x < 0.0 && (self.max.x < other.min.x || self.min.x > other.max.x) {
            return None;
        }
        if entry.y < 0.0 && (self.max.y < other.min.y || self.min.y > other.max.y) {
            return None;
        }
        if entry.z < 0.0 && (self.max.z < other.min.z || self.min.z > other.max.z) {
            return None;
        }

        // We do collide!
        #[allow(clippy::collapsible_else_if)]
        let direction = if entry.x > entry.y {
            if entry.x > entry.z {
                if delta.x > 0.0 {
                    Hit::X
                } else {
                    Hit::NEG_X
                }
            } else {
                if delta.z > 0.0 {
                    Hit::Z
                } else {
                    Hit::NEG_Z
                }
            }
        } else {
            if entry.y > entry.z {
                if delta.y > 0.0 {
                    Hit::Y
                } else {
                    Hit::NEG_Y
                }
            } else {
                if delta.z > 0.0 {
                    Hit::Z
                } else {
                    Hit::NEG_Z
                }
            }
        };

        Some(SweepHit {
            entry_time,
            direction,
        })
    }
}

bitflags! {
    /// A bunch of flags that can be used to determine where a hit occurred.
    #[derive(Debug, Default, Clone, Copy)]
    pub struct Hit: u8 {
        const X = 1 << 0;
        const NEG_X = 1 << 1;
        const Y = 1 << 2;
        const NEG_Y = 1 << 3;
        const Z = 1 << 4;
        const NEG_Z = 1 << 5;

        const HORIZONAL = Self::X.bits() | Self::NEG_X.bits() | Self::Z.bits() | Self::NEG_Z.bits();
    }
}

/// Stores some state to perform collision detection more efficiently.
pub struct CollisionContext {
    /// Stores the positions of the blocks that we might collide with.
    sweep_buffer: Vec<SweepHit>,
}

impl CollisionContext {
    /// Creates a new [`CollisionContext`] instance.
    pub fn new() -> Self {
        Self {
            sweep_buffer: Vec::new(),
        }
    }

    /// Returns whether or not the collider collides with the world at the given position.
    ///
    /// `pos` is the position of the bottom-center of the collider.
    ///
    /// # Returns
    ///
    /// This function returns the offset that should be added to the input position in order to
    /// resolve the collision.
    pub fn sweep(
        &mut self,
        collider: Collider,
        in_pos: &mut Vec3,
        in_vel: &mut Vec3,
        dt: f32,
        world: &World,
    ) -> Hit {
        self.sweep_buffer.clear();

        let mut pos = Vec3A::from(*in_pos);
        let mut vel = Vec3A::from(*in_vel) * dt;

        let mut my_collider = Aabb {
            min: pos + Vec3A::new(-collider.radius, 0.0, -collider.radius),
            max: pos + Vec3A::new(collider.radius, collider.height, collider.radius),
        };

        let broadphase = my_collider.broadphase(vel);

        // Find the other colliders that the box might collide with.
        let min = bns_core::utility::world_pos_of(broadphase.min.into());
        let max = bns_core::utility::world_pos_of(broadphase.max.into());

        // Perform sweep tests against the colliders of the blocks that we might collide with.
        for block_pos in iter_within_bounds(min, max) {
            // Check if the block is actually solid.
            match world.get_block(block_pos) {
                Some(block) if !block.info().flags.contains(BlockFlags::SOLID) => continue,
                // If the block is solid, or if it's not loaded yet, then we perform collision
                // detection against it.
                _ => (),
            }

            let other_collider = Aabb {
                min: block_pos.as_vec3a(),
                max: (block_pos + IVec3::ONE).as_vec3a(),
            };

            if let Some(hit) = my_collider.sweep(vel, &other_collider) {
                // We did hit something!
                self.sweep_buffer.push(hit);
            }
        }

        // Sort the collisions by time of entry, so that we can resolve them in order.
        self.sweep_buffer
            .sort_unstable_by(|a, b| a.entry_time.total_cmp(&b.entry_time));

        let mut result = Hit::empty();

        // Resolve the gathered hits.
        for hit in &self.sweep_buffer {
            // Move as close as possible to the collider.
            let delta = vel * hit.entry_time;
            pos += delta;
            vel *= 1.0 - hit.entry_time;
            *in_vel *= 1.0 - hit.entry_time;

            // Remove the velocity of the axis that we collided with.
            if hit.direction.intersects(Hit::X | Hit::NEG_X) {
                vel.x = 0.0;
                in_vel.x = 0.0;
            } else if hit.direction.intersects(Hit::Y | Hit::NEG_Y) {
                vel.y = 0.0;
                in_vel.y = 0.0;
            } else if hit.direction.intersects(Hit::Z | Hit::NEG_Z) {
                vel.z = 0.0;
                in_vel.z = 0.0;
            }

            result |= hit.direction;

            // If we have no more velocity left, then we're done.
            if vel == Vec3A::ZERO {
                break;
            }

            // Otherwise, we need to check if we can continue moving in the direction of the
            // velocity.
            my_collider.min += delta;
            my_collider.max += delta;
        }

        // Add the remaining velocity to the position.
        pos += vel;

        *in_pos = Vec3::from(pos);
        result
    }
}

/// An axis-aligned square-shaped collider with a height and a radius.
#[derive(Debug, Clone, Copy)]
pub struct Collider {
    /// The height of the collider.
    pub height: f32,
    /// The radius of the collider.
    pub radius: f32,
}

/// Returns an iterator over the positions within the given bounds.
#[rustfmt::skip]
fn iter_within_bounds(min: IVec3, max: IVec3) -> impl Iterator<Item = IVec3> {
    (min.x..=max.x).flat_map(move |x|
    (min.y..=max.y).flat_map(move |y|
    (min.z..=max.z).map(move |z|
    IVec3::new(x, y, z))))
}
