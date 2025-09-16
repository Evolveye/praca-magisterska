mod initialisations;
mod insert;
mod remove;
mod get;
use criterion::{ criterion_group, criterion_main };

#[allow(unused)]
use crate::{
    initialisations::measure_structs_initialization,
    get::{ measure_structs_get, measure_structs_get_random, measure_structs_get_reference },
    insert::{ measure_structs_insert, measure_structs_insert_fill, measure_structs_insert_fill_padded, measure_structs_insert_random },
    remove::{ measure_structs_remove, measure_structs_remove_bulk, measure_structs_remove_random }
};

criterion_group!(
    benches,
    // measure_structs_initialization,
    // measure_structs_insert,
    // measure_structs_insert_fill,
    // measure_structs_insert_fill_padded,
    // measure_structs_insert_random,
    // measure_structs_remove,
    // measure_structs_remove_bulk,
    // measure_structs_remove_random,
    measure_structs_get_reference,
    // measure_structs_get,
    // measure_structs_get_random,
);

criterion_main!( benches );
