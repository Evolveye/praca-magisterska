pub mod tester;
pub mod tester_generators;
pub mod voxel_map;
pub mod voxel_list;
pub mod octree;
pub mod quadtree;
pub mod voxel_hasher;

use std::{time::Instant};

use tester::{Tester, WORLD_X};

#[allow(unused_imports)]
use octree::Octree;

#[allow(unused_imports)]
use voxel_hasher::VoxelHashMap;

#[allow(unused_imports)]
use voxel_list::VoxelList;

#[allow(unused_imports)]
use voxel_map::VoxelMap;

use crate::{structure_tests::tester_generators::GeneratorOfRealisticallyTerrain, world::{world::{ChunkLoaderhandle, World, CHUNK_SIZE}, world_holder::{Voxel, WorldHolding}}};

pub fn generate_world_as_world() -> (World, ChunkLoaderhandle) {
    let world_generator = GeneratorOfRealisticallyTerrain::new( 50 );
    let mut world = World::new( Box::new( world_generator ) );
    let chunk_loader = world.create_chunk_loader( (CHUNK_SIZE as f32 / 2.0, 20.0, CHUNK_SIZE as f32 / 2.0), 3 );

    (world, chunk_loader)
}

#[allow(dead_code)]
pub fn generate_world_as_holder() -> impl WorldHolding {
    println!( "" );
    println!( "Starting tester" );

    // let mut world_struct = VoxelMap::new();
    // let mut world_struct = VoxelList::new();
    let mut world_struct = Octree::from_max_size( WORLD_X );
    println!( "Created world struct with depth = {}", Octree::<Voxel>::get_max_depth_for( WORLD_X ) );
    // let mut world_struct = Octree::new( 11 );
    // let mut world_struct = Octree::new( 18 );
    // let mut world_struct = VoxelHashMap::new();
    let time_start = Instant::now();
    // let dataset = Tester::set_1( &mut world_struct );
    // let dataset = Tester::fill_50pc_realistically_flat( &mut world_struct );
    let dataset = Tester::fill_50pc_realistically( &mut world_struct );
    let time_duration = time_start.elapsed();

    // world_struct.get_all_visible_voxels();

    println!( "" );

    println!( "" );
    dataset.get_size();
    println!( "" );
    world_struct.get_size();
    println!( "" );
    println!( "Time duration = {:?}", time_duration );
    println!( "" );

    world_struct
}
