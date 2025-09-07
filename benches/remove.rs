use std::hint::black_box;
use criterion::{BatchSize, Criterion};

use praca_magisterska::{
    chunks_generators::utilities::{create_voxel, generate_unique},
    structure_tests::{ octree::Octree, voxel_hasher::VoxelHashMap, voxel_list::VoxelList, voxel_map::VoxelMap },
    world::world_holder::{ Color, Material, Voxel, VoxelDataset, WorldHolding }
};


const SIZE:u32 = 100;

pub fn measure_structs_remove( c:&mut Criterion ) {
    let mut group = c.benchmark_group( "Remove from the structs" );

    let voxel= create_voxel(
        &mut VoxelDataset::new(),
        (String::from( "material" ), Material { _density:100 }),
        (String::from( "color" ), Color { red:250, green:10, blue:10 }),
    );

    group.bench_function( format!( "VoxelMap (size {SIZE})" ), |b| b.iter_batched(
        || {
            let mut wh = VoxelMap::<Voxel>::from_max_size( SIZE );
            wh.fill_voxels( (25, 25, 25), (74, 74, 74), black_box( Some( voxel.clone() ) ));
            wh
        },
        |mut wh| {
            for i in 0..1000 {
                let z = 50 + i / 100;
                let x = i % 100;

                black_box( wh.set_voxel( black_box( x ), black_box( 55 ), black_box( z ), black_box( None )) );
            }
        },
        BatchSize::SmallInput,
    ) );

    group.bench_function( format!( "Octree (size {SIZE})" ), |b| b.iter_batched(
        || {
            let mut wh = Octree::<Voxel>::from_max_size( SIZE );
            wh.fill_voxels( (25, 25, 25), (74, 74, 74), black_box( Some( voxel.clone() ) ));
            wh
        },
        |mut wh| {
            for i in 0..1000 {
                let x = 50 + i / 100;
                let z = i % 100;

                black_box( wh.set_voxel( black_box( x ), black_box( 55 ), black_box( z ), black_box( None )) );
            }
        },
        BatchSize::SmallInput,
    ) );

    group.bench_function( "VoxelList", |b| b.iter_batched(
        || {
            let mut wh = VoxelList::<Voxel>::new();
            wh.fill_voxels( (25, 25, 25), (74, 74, 74), black_box( Some( voxel.clone() ) ));
            wh
        },
        |mut wh| {
            for i in 0..1000 {
                let x = 50 + i / 100;
                let z = i % 100;

                black_box( wh.set_voxel( black_box( x ), black_box( 55 ), black_box( z ), black_box( None )) );
            }
        },
        BatchSize::SmallInput,
    ) );

    group.bench_function( "VoxelHashMap", |b| b.iter_batched(
        || {
            let mut wh = VoxelHashMap::<Voxel>::new();
            wh.fill_voxels( (25, 25, 25), (74, 74, 74), black_box( Some( voxel.clone() ) ));
            wh
        },
        |mut wh| {
            for i in 0..1000 {
                let x = 50 + i / 100;
                let z = i % 100;

                black_box( wh.set_voxel( black_box( x ), black_box( 55 ), black_box( z ), black_box( None )) );
            }
        },
        BatchSize::SmallInput,
    ) );
}


pub fn measure_structs_remove_bulk( c:&mut Criterion ) {
    let mut group = c.benchmark_group( "Bulk deletion from the structs" );

    let voxel= create_voxel(
        &mut VoxelDataset::new(),
        (String::from( "material" ), Material { _density:100 }),
        (String::from( "color" ), Color { red:250, green:10, blue:10 }),
    );

    group.bench_function( format!( "VoxelMap (size {SIZE})" ), |b| b.iter_batched(
        || {
            let mut wh = VoxelMap::<Voxel>::from_max_size( 100 );
            wh.fill_voxels( (5, 5, 5), (94, 94, 94), black_box( Some( voxel.clone() ) ));
            wh
        },
        |mut wh| black_box( wh.fill_voxels( black_box( (25, 25, 25) ), black_box( (74, 74, 74) ), black_box( None )) ),
        BatchSize::SmallInput,
    ) );

    group.bench_function( format!( "Octree (size {SIZE})" ), |b| b.iter_batched(
        || {
            let mut wh = Octree::<Voxel>::from_max_size( 100 );
            wh.fill_voxels( (5, 5, 5), (94, 94, 94), black_box( Some( voxel.clone() ) ));
            wh
        },
        |mut wh| black_box( wh.fill_voxels( black_box( (25, 25, 25) ), black_box( (74, 74, 74) ), black_box( None )) ),
        BatchSize::SmallInput,
    ) );

    group.bench_function( "VoxelList", |b| b.iter_batched(
        || {
            let mut wh = VoxelList::<Voxel>::new();
            wh.fill_voxels( (5, 5, 5), (94, 94, 94), black_box( Some( voxel.clone() ) ));
            wh
        },
        |mut wh| black_box( wh.fill_voxels( black_box( (25, 25, 25) ), black_box( (74, 74, 74) ), black_box( None )) ),
        BatchSize::SmallInput,
    ) );

    group.bench_function( "VoxelHashMap", |b| b.iter_batched(
        || {
            let mut wh = VoxelHashMap::<Voxel>::new();
            wh.fill_voxels( (5, 5, 5), (94, 94, 94), black_box( Some( voxel.clone() ) ));
            wh
        },
        |mut wh| black_box( wh.fill_voxels( black_box( (25, 25, 25) ), black_box( (74, 74, 74) ), black_box( None )) ),
        BatchSize::SmallInput,
    ) );
}

pub fn measure_structs_remove_random( c:&mut Criterion ) {
    let size_cube = SIZE * SIZE * SIZE;
    let randoms = generate_unique( 0, 1000 ).iter().map( |i| {
        let i = i % size_cube;

        (
            i % SIZE,
            (i / SIZE) % SIZE,
            i / (SIZE * SIZE),
        )
    } ).collect::<Vec<(u32, u32, u32)>>();

    let mut group = c.benchmark_group( "Random deletion from the structs" );
    let voxel= create_voxel(
        &mut VoxelDataset::new(),
        (String::from( "material" ), Material { _density:100 }),
        (String::from( "color" ), Color { red:250, green:10, blue:10 }),
    );

    group.bench_function( format!( "VoxelMap (size {SIZE})" ), |b| b.iter_batched(
        || {
            let mut wh = VoxelMap::<Voxel>::from_max_size( SIZE );
            wh.fill_voxels( (25, 25, 25), (74, 74, 74), black_box( Some( voxel.clone() ) ));
            wh
        },
        |mut wh| {
            for (x, y, z) in &randoms {
                black_box( wh.set_voxel( black_box( *x ), black_box( *y ), black_box( *z ), black_box( None ) ) );
            }
        },
        BatchSize::SmallInput,
    ) );

    group.bench_function( format!( "Octree (size {SIZE})" ), |b| b.iter_batched(
        || {
            let mut wh = Octree::<Voxel>::from_max_size( SIZE );
            wh.fill_voxels( (5, 5, 5), (94, 94, 94), black_box( Some( voxel.clone() ) ));
            wh
        },
        |mut wh| {
            for (x, y, z) in &randoms {
                black_box( wh.set_voxel( black_box( *x ), black_box( *y ), black_box( *z ), black_box( None ) ) );
            }
        },
        BatchSize::SmallInput,
    ) );

    group.bench_function( "VoxelList", |b| b.iter_batched(
        || {
            let mut wh = VoxelList::<Voxel>::new();
            wh.fill_voxels( (5, 5, 5), (94, 94, 94), black_box( Some( voxel.clone() ) ));
            wh
        },
        |mut wh| {
            for (x, y, z) in &randoms {
                black_box( wh.set_voxel( black_box( *x ), black_box( *y ), black_box( *z ), black_box( None ) ) );
            }
        },
        BatchSize::SmallInput,
    ) );

    group.bench_function( "VoxelHashMap", |b| b.iter_batched(
        || {
            let mut wh = VoxelHashMap::<Voxel>::new();
            wh.fill_voxels( (5, 5, 5), (94, 94, 94), black_box( Some( voxel.clone() ) ));
            wh
        },
        |mut wh| {
            for (x, y, z) in &randoms {
                black_box( wh.set_voxel( black_box( *x ), black_box( *y ), black_box( *z ), black_box( None ) ) );
            }
        },
        BatchSize::SmallInput,
    ) );
}
