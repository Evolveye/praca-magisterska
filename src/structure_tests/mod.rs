pub mod tester;
pub mod voxel_map;
pub mod voxel_list;
pub mod octree;
pub mod voxel_hasher;

use std::time::Instant;

use tester::WorldHolder;

#[allow(unused_imports)]
use octree::Octree;

#[allow(unused_imports)]
use voxel_hasher::VoxelHashMap;

#[allow(unused_imports)]
use voxel_list::VoxelList;

#[allow(unused_imports)]
use voxel_map::VoxelMap;

pub fn run_test() {
    println!( "" );
    println!( "Starting tester" );

    let tester = tester::Tester {};
    // let mut world_struct = VoxelMap::new();
    // let mut world_struct = VoxelList::new();
    let mut world_struct = Octree::new( 18 );
    // let mut world_struct = VoxelHashMap::new();
    let time_start = Instant::now();
    let dataset = tester.fill_50pc_realistically( &mut world_struct );
    let time_duration = time_start.elapsed();

    println!( "" );

    println!( "" );
    dataset.get_size();
    println!( "" );
    world_struct.get_size();
    println!( "" );
    println!( "Time duration = {:?}", time_duration );
    println!( "" );
}
