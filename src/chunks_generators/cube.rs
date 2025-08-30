use crate::{
    chunks_generators::create_voxel::create_voxel,
    noise::simplex_noise::SimplexNoise,
    structure_tests::octree::Octree,
    world::{
        world_generator::WorldGenerative,
        world_holder::{
            Color, Material, Voxel, VoxelDataset, WorldHolding
        }
    }
};

pub struct GeneratorOfCube {
    noise: SimplexNoise,
    noise_frequency: f64,
    size_in_cubes: i64,
    color_start: (u8, u8, u8),
    color_1: (u8, u8, u8),
    color_2: (u8, u8, u8),
    color_3: (u8, u8, u8),
    color_end: (u8, u8, u8),
}

impl GeneratorOfCube {
    #[allow(dead_code)]
    pub fn new( seed:u32) -> Self {
        Self {
            noise: SimplexNoise::new( seed ),
            noise_frequency: 0.025,
            size_in_cubes: 2,
            color_start: (25, 150, 15),
            color_1: (100, 50, 15),
            color_2: (53, 10, 0),
            color_3: (255, 0, 0),
            color_end: (250, 100, 20),
        }
    }

    fn lerp(a: u8, b: u8, t: f64) -> u8 {
        ((a as f64) + (b as f64 - a as f64) * t).round() as u8
    }

    fn lerp_color( c1:(u8,u8,u8), c2:(u8,u8,u8), t:f64 ) -> (u8, u8, u8) {
        (
            Self::lerp( c1.0, c2.0, t ),
            Self::lerp( c1.1, c2.1, t ),
            Self::lerp( c1.2, c2.2, t ),
        )
    }
}

impl WorldGenerative for GeneratorOfCube {
    fn generate_chunk( &self, dataset:&mut VoxelDataset, origin:(i64, i64, i64), size:u8 ) -> Octree<Voxel> {
        let mut world_holder = Octree::from_max_size( size as u32 );

        if origin.0 < 0 || origin.1 < 0 || origin.2 < 0 {
            return world_holder
        }

        if origin.0 >= self.size_in_cubes || origin.1 >= self.size_in_cubes || origin.2 >= self.size_in_cubes {
            return world_holder
        }

        let world_origin = (origin.0 * size as i64, origin.1 * size as i64, origin.2 * size as i64);
        let size = size as i64;
        let cube_edge = self.size_in_cubes * size;
        let half_cube_edge = cube_edge / 2;

        for x in world_origin.0..world_origin.0 + size {
            let x_mod = x % cube_edge;

            for y in world_origin.1..world_origin.1 + size {
                let y_mod =  y % cube_edge;

                for z in world_origin.2..world_origin.2 + size {
                    let z_mod = z % cube_edge;

                    let value = self.noise.noise3d(
                        (x as f64 + 1.0) * self.noise_frequency,
                        (y as f64 + 1.0) * self.noise_frequency,
                        (z as f64 + 1.0) * self.noise_frequency,
                    );

                    let max_coord = (half_cube_edge - x_mod.min( cube_edge - 1 - x_mod ))
                        .max( half_cube_edge - y_mod.min( cube_edge - 1 - y_mod ) )
                        .max( half_cube_edge - z_mod.min( cube_edge - 1 - z_mod ) );

                    let gradient_value = max_coord as f64 / (cube_edge as f64 / 2.0);
                    let gradient_value_inv = 1.0 - gradient_value;

                    let voxel = if value < 0.25 || max_coord < 5 { None } else {

                        let color = if gradient_value_inv == 0.0 {
                            self.color_start
                        } else if gradient_value_inv < 0.8 {
                            let local_t = gradient_value_inv / 0.8;
                            Self::lerp_color(self.color_1, self.color_2, local_t)
                        } else if gradient_value_inv < 0.9 {
                            let local_t = (gradient_value_inv - 0.8) / 0.2;
                            Self::lerp_color(self.color_2, self.color_3, local_t)
                        } else {
                            self.color_end
                        };

                        Some( create_voxel(
                            dataset,
                            (String::from( "grass" ), Material { _density:10 }),
                            (format!( "grass-{}", gradient_value ), Color {
                                red: color.0,
                                green: color.1,
                                blue: color.2,
                            } ),
                        ) )
                    };

                    world_holder.set_voxel(
                        (x - world_origin.0) as u32,
                        (y - world_origin.1) as u32,
                        (z - world_origin.2) as u32,
                        voxel
                    );
                }
            }
        }

        world_holder
    }
}
