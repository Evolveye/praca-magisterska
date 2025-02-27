pub mod tester;
pub mod voxel_map;

use tester::WorldHolder;
use voxel_map::VoxelMap;

pub fn run_test() {
    println!( "" );
    println!( "Starting tester" );

    let tester = tester::Tester {};
    let mut voxel_map = VoxelMap::new();
    let dataset = tester.fill_100( &mut voxel_map );

    println!( "" );
    dataset.get_size();
    println!( "" );
    voxel_map.get_size();
    println!( "" );
}
