use std::sync::Arc;

use crate::{
    chunks_generators::utilities::{create_voxel, generate_unique},
    structure_tests::octree::Octree,
    world::{
        world_generator::WorldGenerative,
        world_holder::{ Material, Voxel, VoxelDataset, WorldHolding }
    }
};

pub struct GeneratorOfTest4HalfRandom {
    color: (u8, u8, u8),
}

impl GeneratorOfTest4HalfRandom {
    #[allow(dead_code)]
    pub fn new( _seed:u32) -> Self {
        Self {
            color: (25, 150, 15)
        }
    }
}

impl WorldGenerative for GeneratorOfTest4HalfRandom {
    fn generate_chunk( &self, dataset:&mut VoxelDataset, origin:(i64, i64, i64), size:u8 ) -> Octree<Voxel> {
        let mut world_holder = Octree::from_max_size( size as u32 );
        let size = size as usize;
        let randoms = generate_unique(
            (origin.0.abs() as u64) << 6 | (origin.1.abs() as u64) << 3 | (origin.2.abs() as u64),
            size * size * size / 2
        );

        let size = size as u32;
        let voxel = create_voxel(
            dataset,
            (String::from( "grass" ), Material { _density:10 }),
            (String::from( "grass" ), self.color.into() ),
        );

        for i in randoms {
            let x = i % size;
            let y = (i / size) % size;
            let z = i / (size * size);

            world_holder.set_voxel(
                x,
                y,
                z,
                Some( Arc::clone( &voxel ) ),
            );
        }

        world_holder
    }
}
