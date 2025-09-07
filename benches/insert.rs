use std::hint::black_box;
use criterion::{BatchSize, Criterion};

use praca_magisterska::{
    chunks_generators::utilities::{create_voxel, generate_unique},
    structure_tests::{ octree::Octree, voxel_hasher::VoxelHashMap, voxel_list::VoxelList, voxel_map::VoxelMap },
    world::world_holder::{ Color, Material, Voxel, VoxelDataset, WorldHolding }
};

const SIZE:u32 = 100;

pub fn measure_structs_insert( c:&mut Criterion ) {
    let mut group = c.benchmark_group( "Insertion into the structs" );
    let voxel= create_voxel(
        &mut VoxelDataset::new(),
        (String::from( "material" ), Material { _density:100 }),
        (String::from( "color" ), Color { red:250, green:10, blue:10 }),
    );

    group.bench_function( "VoxelMap (size 100)", |b| b.iter_batched(
        || VoxelMap::<Voxel>::from_max_size( 100 ),
        |mut wh| {
            for i in 0..1000 {
                let z = 50 + i / 100;
                let x = i % 100;

                black_box( wh.set_voxel( black_box( x ), black_box( 55 ), black_box( z ), black_box( Some( voxel.clone() ) )) );
            }
        },
        BatchSize::SmallInput,
    ) );

    group.bench_function( "Octree (size 100)", |b| b.iter_batched(
        || Octree::<Voxel>::from_max_size( 100 ),
        |mut wh| {
            for i in 0..1000 {
                let x = 50 + i / 100;
                let z = i % 100;

                black_box( wh.set_voxel( black_box( x ), black_box( 55 ), black_box( z ), black_box( Some( voxel.clone() ) )) );
            }
        },
        BatchSize::SmallInput,
    ) );

    group.bench_function( "VoxelList", |b| b.iter_batched(
        || VoxelList::<Voxel>::new(),
        |mut wh| {
            for i in 0..1000 {
                let x = 50 + i / 100;
                let z = i % 100;

                black_box( wh.set_voxel( black_box( x ), black_box( 55 ), black_box( z ), black_box( Some( voxel.clone() ) )) );
            }
        },
        BatchSize::SmallInput,
    ) );

    group.bench_function( "VoxelHashMap", |b| b.iter_batched(
        || VoxelHashMap::<Voxel>::new(),
        |mut wh| {
            for i in 0..1000 {
                let x = 50 + i / 100;
                let z = i % 100;

                black_box( wh.set_voxel( black_box( x ), black_box( 55 ), black_box( z ), black_box( Some( voxel.clone() ) )) );
            }
        },
        BatchSize::SmallInput,
    ) );
}

pub fn measure_structs_insert_fill( c:&mut Criterion ) {
    let mut group = c.benchmark_group( "Fill the structs (from (0, 0, 0) to (99, 99, 99))" );

    let voxel= create_voxel(
        &mut VoxelDataset::new(),
        (String::from( "material" ), Material { _density:100 }),
        (String::from( "color" ), Color { red:250, green:10, blue:10 }),
    );

    group.bench_function( "VoxelMap (size 100)", |b| b.iter_batched(
        || VoxelMap::<Voxel>::from_max_size( 100 ),
        |mut wh| black_box( wh.fill_voxels( black_box( (0, 0, 0) ), black_box( (99, 99, 99) ), black_box( Some( voxel.clone() ) )) ),
        BatchSize::SmallInput,
    ) );

    group.bench_function( "Octree (size 100)", |b| b.iter_batched(
        || Octree::<Voxel>::from_max_size( 100 ),
        |mut wh| black_box( wh.fill_voxels( black_box( (0, 0, 0) ), black_box( (99, 99, 99) ), black_box( Some( voxel.clone() ) )) ),
        BatchSize::SmallInput,
    ) );

    group.bench_function( "VoxelList", |b| b.iter_batched(
        || VoxelList::<Voxel>::new(),
        |mut wh| black_box( wh.fill_voxels( black_box( (0, 0, 0) ), black_box( (99, 99, 99) ), black_box( Some( voxel.clone() ) )) ),
        BatchSize::SmallInput,
    ) );

    group.bench_function( "VoxelHashMap", |b| b.iter_batched(
        || VoxelHashMap::<Voxel>::new(),
        |mut wh| black_box( wh.fill_voxels( black_box( (0, 0, 0) ), black_box( (99, 99, 99) ), black_box( Some( voxel.clone() ) )) ),
        BatchSize::SmallInput,
    ) );
}

pub fn measure_structs_insert_fill_padded( c:&mut Criterion ) {
    let mut group = c.benchmark_group( "Fill the structs (from (25, 25, 25) to (74, 74, 74))" );

    let voxel= create_voxel(
        &mut VoxelDataset::new(),
        (String::from( "material" ), Material { _density:100 }),
        (String::from( "color" ), Color { red:250, green:10, blue:10 }),
    );

    group.bench_function( "VoxelMap (size 100)", |b| b.iter_batched(
        || VoxelMap::<Voxel>::from_max_size( 100 ),
        |mut wh| black_box( wh.fill_voxels( black_box( (25, 25, 25) ), black_box( (74, 74, 74) ), black_box( Some( voxel.clone() ) )) ),
        BatchSize::SmallInput,
    ) );

    group.bench_function( "Octree (size 100)", |b| b.iter_batched(
        || Octree::<Voxel>::from_max_size( 100 ),
        |mut wh| black_box( wh.fill_voxels( black_box( (25, 25, 25) ), black_box( (74, 74, 74) ), black_box( Some( voxel.clone() ) )) ),
        BatchSize::SmallInput,
    ) );

    group.bench_function( "VoxelList", |b| b.iter_batched(
        || VoxelList::<Voxel>::new(),
        |mut wh| black_box( wh.fill_voxels( black_box( (25, 25, 25) ), black_box( (74, 74, 74) ), black_box( Some( voxel.clone() ) )) ),
        BatchSize::SmallInput,
    ) );

    group.bench_function( "VoxelHashMap", |b| b.iter_batched(
        || VoxelHashMap::<Voxel>::new(),
        |mut wh| black_box( wh.fill_voxels( black_box( (25, 25, 25) ), black_box( (74, 74, 74) ), black_box( Some( voxel.clone() ) )) ),
        BatchSize::SmallInput,
    ) );
}

pub fn measure_structs_insert_random( c:&mut Criterion ) {
    let size_cube = SIZE * SIZE * SIZE;
    let randoms = generate_unique( 0, 1000 ).iter().map( |i| {
        let i = i % size_cube;

        (
            i % SIZE,
            (i / SIZE) % SIZE,
            i / (SIZE * SIZE),
        )
    } ).collect::<Vec<(u32, u32, u32)>>();

    let mut group = c.benchmark_group( "Random insertion into the structs" );
    let voxel= create_voxel(
        &mut VoxelDataset::new(),
        (String::from( "material" ), Material { _density:100 }),
        (String::from( "color" ), Color { red:250, green:10, blue:10 }),
    );

    group.bench_function( format!( "VoxelMap (size {SIZE})" ), |b| b.iter_batched(
        || VoxelMap::<Voxel>::from_max_size( SIZE ),
        |mut wh| {
            for (x, y, z) in &randoms {
                black_box( wh.set_voxel( black_box( *x ), black_box( *y ), black_box( *z ), black_box( Some( voxel.clone() ) )) );
            }
        },
        BatchSize::SmallInput,
    ) );

    group.bench_function( format!( "Octree (size {SIZE})" ), |b| b.iter_batched(
        || Octree::<Voxel>::from_max_size( SIZE ),
        |mut wh| {
            for (x, y, z) in &randoms {
                black_box( wh.set_voxel( black_box( *x ), black_box( *y ), black_box( *z ), black_box( Some( voxel.clone() ) )) );
            }
        },
        BatchSize::SmallInput,
    ) );

    group.bench_function( "VoxelList", |b| b.iter_batched(
        || VoxelList::<Voxel>::new(),
        |mut wh| {
            for (x, y, z) in &randoms {
                black_box( wh.set_voxel( black_box( *x ), black_box( *y ), black_box( *z ), black_box( Some( voxel.clone() ) )) );
            }
        },
        BatchSize::SmallInput,
    ) );

    group.bench_function( "VoxelHashMap", |b| b.iter_batched(
        || VoxelHashMap::<Voxel>::new(),
        |mut wh| {
            for (x, y, z) in &randoms {
                black_box( wh.set_voxel( black_box( *x ), black_box( *y ), black_box( *z ), black_box( Some( voxel.clone() ) )) );
            }
        },
        BatchSize::SmallInput,
    ) );
}
