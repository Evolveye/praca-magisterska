use crate::{
    chunks_generators::utilities::create_voxel,
    noise::simplex_noise::SimplexNoise,
    structure_tests::octree::Octree,
    world::{
        world_generator::WorldGenerative,
        world_holder::{ Material, Voxel, VoxelDataset, WorldHolding }
    }
};

pub struct GeneratorOfCube {
    noise: SimplexNoise,
    noise_frequency: f64,
    dimensions: (i64, i64, i64),
    half_dimensions:(i64, i64, i64),
    max_dimension: i64,
    color_start: (u8, u8, u8),
    color_deep_1: (u8, u8, u8),
    color_deep_2: (u8, u8, u8),
    color_near_center: (u8, u8, u8),
    color_center: (u8, u8, u8),
}

impl GeneratorOfCube {
    #[allow(dead_code)]
    pub fn new( seed:u32) -> Self {
        Self::new_with_size( seed, (128, 128, 128) )
    }

    pub fn new_with_size( seed:u32, dimensions:(i64, i64, i64)) -> Self {
        Self {
            noise: SimplexNoise::new( seed ),
            noise_frequency: 0.025,
            dimensions,
            half_dimensions: (
                dimensions.0 / 2,
                dimensions.1 / 2,
                dimensions.2 / 2,
            ),
            max_dimension: dimensions.0.max( dimensions.1 ).max( dimensions.2 ),
            color_start: (25, 150, 15),
            color_deep_1: (100, 50, 15),
            color_deep_2: (53, 10, 0),
            color_near_center: (255, 0, 0),
            color_center: (250, 100, 20),
        }
    }

    pub fn generate( &self, dataset:&mut VoxelDataset, world_holder:Octree<Voxel>, world_origin:(i64, i64, i64), size:u8 ) -> Octree<Voxel> {
        self.generate_with_shift( dataset, world_holder, world_origin, (0, 0, 0), size )
    }

    pub fn generate_with_shift( &self, dataset:&mut VoxelDataset, mut world_holder:Octree<Voxel>, world_origin:(i64, i64, i64), shift:(i64, i64, i64), size:u8 ) -> Octree<Voxel> {
        let size = size as i64;
        let half_max_dim = self.max_dimension as f64 / 2.0;

        let range_size = (
            size.min( self.dimensions.0 ),
            size.min( self.dimensions.1 ),
            size.min( self.dimensions.2 ),
        );

        // println!( "{world_origin:?} {range_size:?} {:?}", self.dimensions );

        for x in world_origin.0 + shift.0..world_origin.0 + range_size.0 {
            let x_mod = if x >= 0 { x } else { (x + 1).abs() } % self.dimensions.0;

            for y in world_origin.1 + shift.1..world_origin.1 + range_size.1 {
                let y_mod = if y >= 0 { y } else { (y + 1).abs() } % self.dimensions.1;

                for z in world_origin.2 + shift.2..world_origin.2 + range_size.2 {
                    let z_mod = if z >= 0 { z } else { (z + 1).abs() } % self.dimensions.2;

                    let value = self.noise.noise3d(
                        (x as f64 + 1.0) * self.noise_frequency,
                        (y as f64 + 1.0) * self.noise_frequency,
                        (z as f64 + 1.0) * self.noise_frequency,
                    );

                    // if y_mod == 0 && z_mod == 0 {
                    //     println!( " - coords: {x_mod} {y_mod} {z_mod}" )
                    // }

                    let max_coord = (self.half_dimensions.0 - x_mod.min( self.dimensions.0 - 1 - x_mod ))
                        .max( self.half_dimensions.1 - y_mod.min( self.dimensions.1 - 1 - y_mod ) )
                        .max( self.half_dimensions.2 - z_mod.min( self.dimensions.2 - 1 - z_mod ) );

                    let gradient_value = max_coord as f64 / half_max_dim;
                    let gradient_value_inv = 1.0 - gradient_value;

                    let voxel = if value < 0.25 || max_coord < 5 { None } else {
                        let color = if gradient_value_inv == 0.0 {
                            self.color_start
                        } else if gradient_value_inv < 0.8 {
                            let local_t = gradient_value_inv / 0.8;
                            Self::lerp_color(self.color_deep_1, self.color_deep_2, local_t)
                        } else if gradient_value_inv < 0.9 {
                            let local_t = (gradient_value_inv - 0.8) / 0.2;
                            Self::lerp_color(self.color_deep_2, self.color_near_center, local_t)
                        } else {
                            self.color_center
                        };

                        Some( create_voxel(
                            dataset,
                            (String::from( "grass" ), Material { _density:10 }),
                            (format!( "grass-{}", gradient_value ), color.into() ),
                        ) )
                    };

                    world_holder.set_voxel(
                        (world_origin.0 + x) as u32,
                        (world_origin.1 + y) as u32,
                        (world_origin.2 + z) as u32,
                        // shift.0 + (x - world_origin.0) as u32,
                        // shift.1 + (y - world_origin.1) as u32,
                        // shift.2 + (z - world_origin.2) as u32,
                        voxel
                    );
                }
            }
        }

        world_holder
    }

    #[allow(dead_code)]
    pub fn set_colors( &mut self, color_start:(u8,u8,u8), color_deep_1:(u8,u8,u8), color_deep_2:(u8,u8,u8), color_near_center:(u8,u8,u8), color_center:(u8,u8,u8) ) {
        self.color_start = color_start;
        self.color_deep_1 = color_deep_1;
        self.color_deep_2 = color_deep_2;
        self.color_near_center = color_near_center;
        self.color_center = color_center;
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
        let world_holder = Octree::from_max_size( size as u32 );

        if origin.0 < 0 || origin.1 < 0 || origin.2 < 0 {
            return world_holder
        }

        if origin.0 >= self.dimensions.0 || origin.1 >= self.dimensions.1 || origin.2 >= self.dimensions.2 {
            return world_holder
        }

        let world_origin = (origin.0 * size as i64, origin.1 * size as i64, origin.2 * size as i64);
        self.generate( dataset, world_holder, world_origin, size )
    }
}
