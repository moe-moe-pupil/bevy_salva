use salva::math::Real;
#[cfg(feature = "dim3")]
use crate::math::Vect;

#[cfg(feature = "dim3")]
pub fn cube_particle_positions(ni: usize, nj: usize, nk: usize, particle_rad: f32) -> Vec<Vect> {
    let mut points = Vec::new();
    let half_extents = Vect::new(ni as f32, nj as f32, nk as f32) * particle_rad;

    for i in 0..ni {
        for j in 0..nj {
            for k in 0..nk {
                let x = (i as f32) * particle_rad * 2.0;
                let y = (j as f32) * particle_rad * 2.0;
                let z = (k as f32) * particle_rad * 2.0;
                points.push(Vect::new(x, y, z) + Vect::splat(particle_rad) - half_extents);
            }
        }
    }

    points
}

pub fn particle_volume(particle_radius: Real) -> Real {
    // The volume of a fluid is computed as the volume of a cuboid of half-width equal to particle_radius.
    // It is multiplied by 0.8 so that there is no pressure when the cuboids are aligned on a grid.
    // This mass computation method is inspired from the SplishSplash project.
    #[cfg(feature = "dim2")]
    let particle_volume = particle_radius * particle_radius * na::convert::<_, Real>(4.0 * 0.8);
    #[cfg(feature = "dim3")]
    let particle_volume =
        particle_radius * particle_radius * particle_radius * na::convert::<_, Real>(8.0 * 0.8);
    particle_volume
}

