use std::{cmp, collections::HashMap, rc::Rc};

use rand::seq::IteratorRandom;

use crate::{
    noise::simplex_noise::SimplexNoise, world::world_holder::{
        Color, CommonVoxelData, Material, Voxel, WorldHolding
    }
};

use super::quadtree::Quadtree;

// pub const RENDER_DISTANCE:u32 = 32 * 16 * 2 + 1;
// pub const RENDER_DISTANCE:u32 = 255;
// pub const WORLD_X:u32 = 16;
pub const WORLD_X:u32 = 32 * 16 * 1 + 1;
// pub const WORLD_X:u32 = 32 * 4;
// pub const WORLD_Y:u32 = 63;
// pub const WORLD_Y:u32 = 15;
pub const WORLD_Y:u32 = 384;
const WORLD_HALF_Y:u32 = WORLD_Y / 2;
pub const WORLD_Z:u32 = WORLD_X;

pub struct TestDataset {
    pub materials: HashMap<String, Rc<Material>>,
    pub colors: HashMap<String, Rc<Color>>,
    pub common_voxel_dataset: HashMap<String, Rc<CommonVoxelData>>,
    pub voxels: HashMap<String, Rc<Voxel>>,
}

impl TestDataset {
    pub fn new() -> Self {
        Self {
            colors: HashMap::new(),
            materials: HashMap::new(),
            common_voxel_dataset: HashMap::new(),
            voxels: HashMap::new(),
        }
    }
    pub fn expand( &mut self, dataset:TestDataset ) {
        self.materials.extend( dataset.materials );
        self.colors.extend( dataset.colors );
        self.common_voxel_dataset.extend( dataset.common_voxel_dataset );
        self.voxels.extend( dataset.voxels );
    }

    pub fn get_size( &self ) {
        println!( "TestDataset sizes (in bytes by default)" );

        println!(
            " - rc size = {};  hashmap of colors size = {}",
            size_of::<Rc<Voxel>>(),
            size_of::<HashMap<String, Rc<Color>>>(),
        );

        println!(
            " - colors = {};  color size = {}",
            self.colors.len(),
            size_of::<Color>(),
        );

        println!(
            " - materials = {};  material size = {}",
            self.materials.len(),
            size_of::<Material>(),
        );

        println!(
            " - common_data = {};  common_data size = {}",
            self.common_voxel_dataset.len(),
            size_of::<CommonVoxelData>(),
        );

        println!(
            " - voxels = {};   voxel size = {}",
            self.voxels.len(),
            size_of::<Voxel>(),
        );
    }
}

pub struct Tester {}

#[allow(dead_code)]
impl Tester {
    pub fn set_0( _world_holder:&dyn WorldHolding ) -> TestDataset {
        TestDataset {
            materials: HashMap::new(),
            colors: HashMap::new(),
            common_voxel_dataset: HashMap::new(),
            voxels: HashMap::new(),
        }
    }

    pub fn set_1( world_holder:&mut dyn WorldHolding ) -> TestDataset {
        let key = String::from( "default" );
        let materials = HashMap::from([ (key.clone(), Rc::new( Material { _density:100 } )) ]);

        let colors = HashMap::from([ (key.clone(), Rc::new( Color { red:50, green:100, blue:200 } )) ]);
        let common_voxel_dataset = HashMap::from([ (key.clone(), Rc::new( CommonVoxelData {
            _material:materials.get( &key ).unwrap().clone(),
            _color:colors.get( &key ).unwrap().clone(),
        } ) ) ]);

        let voxels = HashMap::from([ (key.clone(), Rc::new( Voxel {
            _common_data: common_voxel_dataset.get( &key ).unwrap().clone(),
            _individual_data: vec![],
        }) ) ]);

        world_holder.set_voxel( 0, 0, 0, Some( voxels.get( &key ).unwrap().clone() ) );

        let test_dataset = TestDataset { materials, colors, common_voxel_dataset, voxels };

        test_dataset
    }

    pub fn set_50pc( world_holder:&mut dyn WorldHolding ) -> TestDataset {
        Self::set_n( WORLD_Z * WORLD_Y * WORLD_X / 2, world_holder )
    }

    pub fn set_100pc( world_holder:&mut dyn WorldHolding ) -> TestDataset {
        Self::set_n( WORLD_Z * WORLD_Y * WORLD_X, world_holder )
    }

    pub fn set_50pc_random( world_holder:&mut dyn WorldHolding ) -> TestDataset {
        Self::set_n_random( WORLD_Z * WORLD_Y * WORLD_X / 2, world_holder )
    }

    pub fn set_50pc_uniques( world_holder:&mut dyn WorldHolding ) -> TestDataset {
        Self::set_n_uniques( WORLD_Z * WORLD_Y * WORLD_X / 2, world_holder )
    }

    pub fn set_99pc( world_holder:&mut dyn WorldHolding ) -> TestDataset {
        Self::set_n( WORLD_Z * WORLD_Y * WORLD_X - 1, world_holder )
    }

    pub fn set_100_uniques( world_holder:&mut dyn WorldHolding ) -> TestDataset {
        Self::set_n_uniques( WORLD_Z * WORLD_Y * WORLD_X, world_holder )
    }

    pub fn fill_50pc( world_holder:&mut dyn WorldHolding ) -> TestDataset {
        Self::fill( (0, 0, 0), (WORLD_Z, WORLD_Y, WORLD_X / 2), world_holder )
    }

    pub fn fill_50pc_realistically_flat( world_holder:&mut dyn WorldHolding ) -> TestDataset {
        let dirt_color = Color { red:100, green:60, blue:40 };
        let grass_color = Color { red:10, green:64, blue:10 };
        let mut dataset = TestDataset::new();

        println!( "Filling stone | {:?} to {:?}", (0, 0, 0), (WORLD_Z, WORLD_HALF_Y - 3, WORLD_X) );
        let dataset_stone = Self::fill( (0, 0, 0), (WORLD_Z, WORLD_HALF_Y - 3, WORLD_X), world_holder );
        dataset.expand( dataset_stone );

        println!( "Filling dirt | {:?} to {:?}", (0, WORLD_HALF_Y - 2, 0), (WORLD_Z, WORLD_HALF_Y - 1, WORLD_X) );
        let dataset_dirt = Self::fill_with( (0, WORLD_HALF_Y - 2, 0), (WORLD_Z, WORLD_HALF_Y - 1, WORLD_X), world_holder, (String::from( "dirt" ), dirt_color) );
        dataset.expand( dataset_dirt );

        println!( "Filling grass | {:?} to {:?}", (0, WORLD_HALF_Y, 0), (WORLD_Z, WORLD_HALF_Y, WORLD_X) );
        let dataset_grass = Self::fill_with( (0, WORLD_HALF_Y, 0), (WORLD_Z, WORLD_HALF_Y, WORLD_X), world_holder, (String::from( "grass" ), grass_color) );
        dataset.expand( dataset_grass );

        Tester::fill_50pc_realistically_ending( world_holder, dataset )
    }

    pub fn fill_50pc_realistically( world_holder:&mut dyn WorldHolding ) -> TestDataset {
        let noise = SimplexNoise::new( 50 );
        let max_depth = Quadtree::get_max_depth_for( WORLD_X );
        let noise_frequency = 0.025;
        let generate_value = |x, z| noise.noise3d( x as f64 * noise_frequency, 1.0, z as f64  * noise_frequency );

        println!( "Filling quadtree with max depth = {}...", max_depth );
        let quadtree = Quadtree::from_terrain_generation( max_depth, &generate_value );

        println!( "Filling world holder with height map..." );

        let stone_key = String::from( "stone" );
        let dirt_key = String::from( "dirt" );
        let grass_key = String::from( "grass" );

        let materials = HashMap::from([
            (&stone_key, Rc::new( Material { _density:100 } )),
            (&dirt_key, Rc::new( Material { _density:2 } )),
            (&grass_key, Rc::new( Material { _density:4 } )),
        ]);

        let colors = HashMap::from([
            (&stone_key, Rc::new( Color { red:50, green:50, blue:50 } )),
            (&dirt_key, Rc::new( Color { red:100, green:60, blue:40 } )),
            (&grass_key, Rc::new( Color { red:10, green:64, blue:10 } )),
        ]);

        let common_voxel_dataset = HashMap::from([
            (&stone_key, Rc::new( CommonVoxelData {
                _material: materials.get( &stone_key ).unwrap().clone(),
                _color: colors.get( &stone_key ).unwrap().clone(),
            } ) ),
            (&dirt_key, Rc::new( CommonVoxelData {
                _material: materials.get( &dirt_key ).unwrap().clone(),
                _color: colors.get( &dirt_key ).unwrap().clone(),
            } ) ),
            (&grass_key, Rc::new( CommonVoxelData {
                _material: materials.get( &grass_key ).unwrap().clone(),
                _color: colors.get( &grass_key ).unwrap().clone(),
            } ) ),
        ]);

        let voxels = HashMap::from([
            (&stone_key, Rc::new( Voxel {
                _common_data: common_voxel_dataset.get( &stone_key ).unwrap().clone(),
                _individual_data: vec![],
            }) ),
            (&dirt_key, Rc::new( Voxel {
                _common_data: common_voxel_dataset.get( &dirt_key ).unwrap().clone(),
                _individual_data: vec![],
            }) ),
            (&grass_key, Rc::new( Voxel {
                _common_data: common_voxel_dataset.get( &grass_key ).unwrap().clone(),
                _individual_data: vec![],
            }) ),
        ]);

        let _stone_voxel = voxels.get( &stone_key ).unwrap();
        let _dirt_voxel = voxels.get( &dirt_key ).unwrap();
        let _grass_voxel = voxels.get( &grass_key ).unwrap();

        quadtree.proces_entire_tree( &mut |offset, size, noise_value| {
            let multiplied_noise = noise_value * 10.0;
            let current_min = (WORLD_HALF_Y as i32 + multiplied_noise as i32) as u32;
            if current_min < offset.1 { return offset.1 }

            let size = size - 1;
            let to = (offset.0 + size, current_min, offset.2 + size);

            // {
            //     world_holder.fill_voxels( offset, to, Some( stone_voxel.clone() ) );
            // }

            {
                let below_water = current_min < WORLD_HALF_Y - 5;
                let too_high = current_min > WORLD_HALF_Y + 7;
                let grass_color = Color {
                    red: if current_min % 2 == 0 { 10 }
                        else if below_water { 20 }
                        else if too_high { 250 } else { 128 },
                    green: (127 + (multiplied_noise * 10.0) as i16) as u8,
                    blue: if below_water { 150 }
                        else if too_high { 250 }
                        else { 10 },
                };
                Self::fill_with( offset, to, world_holder, (format!( "grass_{}", current_min ), grass_color) );
            }

            current_min + 1
        } );

        // quadtree.process_entire_surface( &mut |(x, z), noise_value| {
        //     let multiplied_noise = noise_value * 10.0;
        //     let y = (WORLD_HALF_Y as i32 + multiplied_noise as i32) as u32;

        //     world_holder.fill_voxels( (x, y, z), (x, y, z), Some( grass_voxel.clone() ) );
        // } );

        TestDataset::new()
        // Tester::fill_50pc_realistically_ending( world_holder, TestDataset::new() )
    }

    pub fn fill_100pc( world_holder:&mut dyn WorldHolding ) -> TestDataset {
        Self::fill( (0, 0, 0), (WORLD_Z, WORLD_Y, WORLD_X), world_holder )
    }

    fn set_n( n:u32, world_holder:&mut dyn WorldHolding ) -> TestDataset {
        let key = String::from( "default" );
        let materials = HashMap::from([ (key.clone(), Rc::new( Material { _density:100 } )) ]);
        let colors = HashMap::from([ (key.clone(), Rc::new( Color { red:50, green:100, blue:200 } )) ]);

        let common_voxel_dataset = HashMap::from([ (key.clone(), Rc::new( CommonVoxelData {
            _material:materials.get( &key ).unwrap().clone(),
            _color:colors.get( &key ).unwrap().clone(),
        } ) ) ]);

        let voxels = HashMap::from([ (key.clone(), Rc::new( Voxel {
            _common_data: common_voxel_dataset.get( &key ).unwrap().clone(),
            _individual_data: vec![],
        }) ) ]);

        let voxel = voxels.get( &key ).unwrap();

        for num in 0..cmp::min( n, WORLD_Z * WORLD_Y * WORLD_X ) {
            let (x, y, z) = Self::get_3d_indices_from_n( num );
            world_holder.set_voxel( x, y, z, Some( voxel.clone() ) );

            if num == n { break }
            Self::print_num( num, n );
        }

        TestDataset { materials, colors, common_voxel_dataset, voxels }
    }

    fn set_n_random( n:u32, world_holder:&mut dyn WorldHolding ) -> TestDataset {
        let key = String::from( "default" );
        let materials = HashMap::from([ (key.clone(), Rc::new( Material { _density:100 } )) ]);
        let colors = HashMap::from([ (key.clone(), Rc::new( Color { red:50, green:100, blue:200 } )) ]);

        let common_voxel_dataset = HashMap::from([ (key.clone(), Rc::new( CommonVoxelData {
            _material:materials.get( &key ).unwrap().clone(),
            _color:colors.get( &key ).unwrap().clone(),
        } ) ) ]);

        let voxels = HashMap::from([ (key.clone(), Rc::new( Voxel {
            _common_data: common_voxel_dataset.get( &key ).unwrap().clone(),
            _individual_data: vec![],
        }) ) ]);

        let mut rng = rand::rng();
        let random = (0..(WORLD_Z * WORLD_Y * WORLD_X)).choose_multiple( &mut rng, n as usize );

        for num in random {
            let (x, y, z) = Self::get_3d_indices_from_n( num );
            world_holder.set_voxel( x, y, z, Some( voxels.get( &key ).unwrap().clone() ) );

            Self::print_num( num, n );
        }

        TestDataset { materials, colors, common_voxel_dataset, voxels }
    }

    fn set_n_uniques( n:u32, world_holder:&mut dyn WorldHolding ) -> TestDataset {
        let mut materials = HashMap::new();
        let mut colors = HashMap::new();
        let mut common_voxel_dataset = HashMap::new();
        let mut voxels:HashMap<_, Rc<Voxel>> = HashMap::new();

        for num in 0..cmp::min( n, WORLD_Z * WORLD_Y * WORLD_X ) {
            let (x, y, z) = Self::get_3d_indices_from_n( num );
            let red = (z % 255) as u8;
            let green = (y % 255) as u8;
            let blue = (x % 255) as u8;
            let color_key = format!( "{red}-{green}-{blue}" );
            let color = match colors.get( &color_key ) {
                Some(color) => color,
                None => {
                    colors.insert( color_key.clone(), Rc::new( Color { red, blue, green } ) );
                    colors.get( &color_key ).unwrap()
                }
            };

            let density = x + y + z;
            let material_key = format!( "{}", density );
            let material = match materials.get( &material_key ) {
                Some( material ) => material,
                None => {
                    materials.insert( material_key.clone(), Rc::new( Material { _density: density } ) );
                    materials.get( &material_key ).unwrap()
                }
            };

            let common_voxel_dataset_key = format!( "{}-{}", color_key, material_key );
            let common_data = match common_voxel_dataset.get( &common_voxel_dataset_key ) {
                Some( common_data ) => common_data,
                None => {
                    common_voxel_dataset.insert( common_voxel_dataset_key.clone(), Rc::new( CommonVoxelData { _color:color.clone(), _material:material.clone() } ) );
                    common_voxel_dataset.get( &common_voxel_dataset_key ).unwrap()
                }
            };

            let voxel_key = format!( "{}-{}", common_voxel_dataset_key, material_key );
            let voxel = match voxels.get( &voxel_key ) {
                Some( voxel ) => voxel,
                None => {
                    voxels.insert( voxel_key.clone(), Rc::new( Voxel { _common_data:common_data.clone(), _individual_data:vec![ material_key ] } ) );
                    voxels.get( &voxel_key ).unwrap()
                }
            };

            world_holder.set_voxel( x, y, z, Some( voxel.clone() ) );

            Self::print_num( num, n );
        }

        TestDataset { materials, colors, common_voxel_dataset, voxels }
    }

    fn fill_with( from:(u32, u32, u32), to:(u32, u32, u32), world_holder:&mut dyn WorldHolding, setup:(String, Color) ) -> TestDataset {
        let materials = HashMap::from([ (setup.0.clone(), Rc::new( Material { _density:100 } )) ]);
        let colors = HashMap::from([ (setup.0.clone(), Rc::new( setup.1 )) ]);

        let common_voxel_dataset = HashMap::from([ (setup.0.clone(), Rc::new( CommonVoxelData {
            _material: materials.get( &setup.0 ).unwrap().clone(),
            _color: colors.get( &setup.0 ).unwrap().clone(),
        } ) ) ]);

        let voxels = HashMap::from([ (setup.0.clone(), Rc::new( Voxel {
            _common_data: common_voxel_dataset.get( &setup.0 ).unwrap().clone(),
            _individual_data: vec![],
        }) ) ]);

        let voxel = voxels.get( &setup.0 ).unwrap();

        world_holder.fill_voxels( from, to, Some( voxel.clone() ) );

        TestDataset { materials, colors:HashMap::new(), common_voxel_dataset, voxels }
    }

    fn fill( from:(u32, u32, u32), to:(u32, u32, u32), world_holder:&mut dyn WorldHolding ) -> TestDataset {
        Self::fill_with( from, to, world_holder, (String::from("default"), Color { red:50, green:50, blue:50 }) )
    }

    fn fill_50pc_realistically_ending( world_holder:&mut dyn WorldHolding, mut dataset:TestDataset ) -> TestDataset {
        let noise = SimplexNoise::new( 50 );

        let coal_key = String::from( "coal" );
        let bedrock_key = String::from( "bedrock" );
        let bedrock_color = Color { red:15, green:15, blue:15 };
        let coal_color = Color { red:5, green:5, blue:5 };


        println!( "Filling bedrock..." );
        let dataset_bedrock = Self::fill_with( (0, 0, 0), (WORLD_Z, 0, WORLD_X), world_holder, (bedrock_key.clone(), bedrock_color) );
        dataset.expand( dataset_bedrock );

        println!( "Filling deposits..." );
        dataset.colors.insert( coal_key.clone(), Rc::new( coal_color ) );
        dataset.materials.insert( coal_key.clone(), Rc::new( Material { _density:125 } ) );

        dataset.common_voxel_dataset.insert( coal_key.clone(), Rc::new( CommonVoxelData {
            _color: dataset.colors.get( &coal_key ).unwrap().clone(),
            _material: dataset.materials.get( &coal_key ).unwrap().clone()
        } ) );

        dataset.voxels.insert( coal_key.clone(), Rc::new( Voxel {
            _common_data: dataset.common_voxel_dataset.get( &coal_key ).unwrap().clone(),
            _individual_data:vec![]
        } ) );

        let coal = dataset.voxels.get( &coal_key ).unwrap().clone();

        for z in 0..=WORLD_Z {
            for y in 1..(if WORLD_HALF_Y > 15 { WORLD_HALF_Y - 15 } else { WORLD_HALF_Y }) {
                for x in 0..=WORLD_X {
                    let noise_value = noise.noise3d( x as f64, y as f64, z as f64 );

                    if noise_value > 0.85 {
                        world_holder.set_voxel( x, y, z, Some( coal.clone() ) );
                    }
                }
            }
        }

        let bedrock = dataset.voxels.get( &bedrock_key ).unwrap().clone();

        for z in 0..=WORLD_Z {
            for y in 1..3 {
                for x in 0..=WORLD_X {
                    let noise_value = noise.noise3d( x as f64, y as f64, z as f64 );

                    if noise_value > 0.8 / y as f64 {
                        world_holder.set_voxel( x, 3 - y, z, Some( bedrock.clone() ) );
                    }
                }
            }
        }

        println!( "Drawing axies..." );
        let axies_origin = ((WORLD_X / 2) as i32, (WORLD_HALF_Y + 15) as i32, (WORLD_Z / 2) as i32);
        let axies_length = 5;
        let axies_key = String::from( "axis" );
        let axies_makers:Vec<(&str, Box<dyn Fn(u8) -> Color>, Box<dyn Fn(i32) -> (i32, i32, i32)>)> = vec![
            ("x", Box::new( |n| Color { red:(255 / (axies_length + 1)) * (n + 2), green:0, blue:0 } ), Box::new( |n| (n - axies_length as i32 / 2, 0, 0) )),
            ("y", Box::new( |n| Color { red:0, green:(255 / (axies_length + 1)) * (n + 2), blue:0 } ), Box::new( |n| (0, n - axies_length as i32 / 2, 0) )),
            ("z", Box::new( |n| Color { red:0, green:0, blue:(255 / (axies_length + 1)) * (n + 2) } ), Box::new( |n| (0, 0, n - axies_length as i32 / 2) )),
        ];

        dataset.materials.insert( axies_key.clone(), Rc::new( Material { _density:1000 } ) );

        if WORLD_X > 15 {
            for axis in axies_makers {
                let key = format!( "{}_{}", &axies_key, axis.0 );

                for n in 0..axies_length {
                    let coords = axis.2( n as i32 );

                    dataset.colors.insert( key.clone(), Rc::new( axis.1( n ) ) );

                    dataset.common_voxel_dataset.insert( key.clone(), Rc::new( CommonVoxelData {
                        _color: dataset.colors.get( &key ).unwrap().clone(),
                        _material: dataset.materials.get( &axies_key ).unwrap().clone()
                    } ) );

                    dataset.voxels.insert( key.clone(), Rc::new( Voxel {
                        _common_data: dataset.common_voxel_dataset.get( &key ).unwrap().clone(),
                        _individual_data:vec![]
                    } ) );

                    world_holder.set_voxel(
                        (axies_origin.0 + coords.0) as u32, (axies_origin.1 + coords.1) as u32, (axies_origin.2 + coords.2) as u32,
                        Some( dataset.voxels.get( &key ).unwrap().clone() )
                    );
                }
            }

            // world_holder.set_voxel( WORLD_X, WORLD_Y, WORLD_Z, Some( dataset.voxels.get( &format!( "{}_{}", &axies_key, "y" ) ).unwrap().clone() ) );
        }

        // let debug_voxel = ( (WORLD_X / 2) + 18, 28, (WORLD_X / 2) + 20 );
        // world_holder.set_voxel( debug_voxel.0, debug_voxel.1, debug_voxel.2, Some( dataset.voxels.get( &format!( "{}_{}", &axies_key, "x" ) ).unwrap().clone() ) );

        dataset
    }

    fn print_num( num:u32, max:u32 ) {
        if num % 200_000 == 0 {
            println!( " num={num}" );

            if num % 5_000_000 == 0 {
                println!( "  max reminder: {max}" );
            }
        }
    }

    fn get_3d_indices_from_n( n:u32 ) -> (u32, u32, u32) {
        let z = n / (WORLD_Y * WORLD_Z);
        let y = (n % (WORLD_Y * WORLD_Z)) / WORLD_Z;
        let x = n % WORLD_Z;

        (x, y, z)
    }
}
