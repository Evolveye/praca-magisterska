use std::hint::black_box;

use praca_magisterska::{
    chunks_generators::utilities::create_voxel,
    structure_tests::{
        octree::{ Octree, OctreeNode },
        voxel_hasher::VoxelHashMap,
        voxel_list::{ VoxelInWorld, VoxelList },
        voxel_map::VoxelMap
    },
    world::{
        world::Position,
        world_holder::{ Color, Material, Voxel, VoxelDataset, WorldHolding }
    }
};

pub fn measure() {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    // initialization();
    // fill_half();
    fill_whole();

    // println!( "VoxelMap = {}", size_of::<VoxelMap::<Voxel>>() );
    // println!( "VoxelList = {} + {}", size_of::<VoxelList::<Voxel>>(), size_of::<VoxelInWorld::<Voxel>>() );
    // println!( "VoxelHashMap = {} + {}", size_of::<VoxelHashMap::<Voxel>>(), size_of::<Position>() );
    // println!( "Octree = {} + {}", size_of::<Octree::<Voxel>>(), size_of::<OctreeNode::<Voxel>>() );
}

#[allow(dead_code)]
fn initialization() {
    black_box( VoxelMap::<Voxel>::from_max_size( black_box( 100 ) ) );
    // black_box( VoxelList::<Voxel>::new() );
    // black_box( VoxelHashMap::<Voxel>::new() );
    // black_box( Octree::<Voxel>::from_max_size( black_box( 100 ) ) );
}

#[allow(dead_code)]
fn fill_half() {
    let voxel= create_voxel(
        &mut VoxelDataset::new(),
        (String::from( "material" ), Material { _density:100 }),
        (String::from( "color" ), Color { red:250, green:10, blue:10 }),
    );

    let mut world_holder = VoxelMap::<Voxel>::from_max_size( black_box( 100 ) );
    // let mut world_holder = VoxelList::<Voxel>::new();
    // let mut world_holder = VoxelHashMap::<Voxel>::new();
    // let mut world_holder = Octree::<Voxel>::from_max_size( black_box( 100 ) );

    black_box( world_holder.fill_voxels(
        black_box( (0, 0, 0) ),
        black_box( (99, 49, 99) ),
        black_box( Some( voxel ) )
    ) );
}

#[allow(dead_code)]
fn fill_whole() {
    let voxel= create_voxel(
        &mut VoxelDataset::new(),
        (String::from( "material" ), Material { _density:100 }),
        (String::from( "color" ), Color { red:250, green:10, blue:10 }),
    );

    let mut world_holder = VoxelMap::<Voxel>::from_max_size( black_box( 100 ) );
    // let mut world_holder = VoxelList::<Voxel>::new();
    // let mut world_holder = VoxelHashMap::<Voxel>::new();
    // let mut world_holder = Octree::<Voxel>::from_max_size( black_box( 100 ) );
    // let mut world_holder = Octree::<Voxel>::from_max_size( black_box( 100 ) );

    black_box( world_holder.fill_voxels(
        black_box( (0, 0, 0) ),
        black_box( (99, 99, 99) ),

        // black_box( (1, 1, 1) ),
        // black_box( (126, 126, 126) ),

        black_box( Some( voxel ) )
    ) );
}
