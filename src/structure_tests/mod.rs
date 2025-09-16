pub mod tester;
pub mod voxel_map;
pub mod voxel_list;
pub mod octree;
pub mod quadtree;
pub mod voxel_hasher;

use std::{time::Instant};

use cgmath::Point3;
use tester::{Tester, WORLD_X};

#[allow(unused_imports)]
use octree::Octree;

#[allow(unused_imports)]
use voxel_hasher::VoxelHashMap;

#[allow(unused_imports)]
use voxel_list::VoxelList;

#[allow(unused_imports)]
use voxel_map::VoxelMap;

use crate::world::{
    world::{ ChunkLoaderhandle, World },
    world_holder::{ Voxel, WorldHolding }
};

#[allow(unused_imports)]
use crate::chunks_generators::{
    peaks_and_valleys::GeneratorOfPeaksAndValleys,
    cube::GeneratorOfCube,
    test_1_empty::GeneratorOfTest1Empty,
    test_2_single::GeneratorOfTest2Single,
    test_3_half::GeneratorOfTest3Half,
    test_4_half_random::GeneratorOfTest4HalfRandom,
    test_5_without_single::GeneratorOfTest5WithoutSingle,
    test_6_full::GeneratorOfTest6Full,
    test_7_half_random_with_differenties::GeneratorOfTest7HalfRanfomWithDifferenties,
    test_8_full_with_differenties::GeneratorOfTest8FullWithDifferenties,
    test_9_natural::GeneratorOfTest9Natural,
    test_10_floating_islands::GeneratorOfTest10FloatingIslands,
    test_11_height_map::GeneratorOfTest11HeightMap,
    test_12_peaks_and_valleys::GeneratorOfTest12PeaksAndValleys,
};

pub fn generate_world_as_world( position:Point3<f32> ) -> (World, ChunkLoaderhandle) {
    let world_generator = GeneratorOfTest12PeaksAndValleys::new( 50 );
    let mut world = World::new( Box::new( world_generator ), None );
    // let mut world = World::new( Box::new( world_generator ), Some( 2 ) );
    let chunk_loader = world.create_chunk_loader( (position.x, position.y, position.z), 4 );

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
