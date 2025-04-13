use std::{cmp, collections::HashMap, rc::Rc};

use rand::seq::IteratorRandom;

use crate::{noise::simplex_noise::SimplexNoise, world::world_holder::{Color, CommonVoxelData, Material, Voxel, WorldHolder}};

// pub const RENDER_DISTANCE:u32 = 32 * 16;
pub const RENDER_DISTANCE:u32 = 32 * 1;
pub const WORLD_Z:u32 = RENDER_DISTANCE * 2 + 1;
pub const WORLD_Y:u32 = 384;
pub const WORLD_X:u32 = RENDER_DISTANCE * 2 + 1;

pub struct TestDataset {
    pub materials: HashMap<String, Rc<Material>>,
    pub colors: HashMap<String, Rc<Color>>,
    pub common_voxel_dataset: HashMap<String, Rc<CommonVoxelData>>,
    pub voxels: HashMap<String, Rc<Voxel>>,
}

impl TestDataset {
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

impl Tester {
    #[allow(dead_code)]
    pub fn set_0( _world_holder:&dyn WorldHolder ) -> TestDataset {
        TestDataset {
            materials: HashMap::new(),
            colors: HashMap::new(),
            common_voxel_dataset: HashMap::new(),
            voxels: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn set_1( world_holder:&mut dyn WorldHolder ) -> TestDataset {
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

    #[allow(dead_code)]
    pub fn set_50pc( world_holder:&mut dyn WorldHolder ) -> TestDataset {
        Self::set_n( WORLD_Z * WORLD_Y * WORLD_X / 2, world_holder )
    }

    #[allow(dead_code)]
    pub fn set_100pc( world_holder:&mut dyn WorldHolder ) -> TestDataset {
        Self::set_n( WORLD_Z * WORLD_Y * WORLD_X, world_holder )
    }

    #[allow(dead_code)]
    pub fn set_50pc_random( world_holder:&mut dyn WorldHolder ) -> TestDataset {
        Self::set_n_random( WORLD_Z * WORLD_Y * WORLD_X / 2, world_holder )
    }

    #[allow(dead_code)]
    pub fn set_50pc_uniques( world_holder:&mut dyn WorldHolder ) -> TestDataset {
        Self::set_n_uniques( WORLD_Z * WORLD_Y * WORLD_X / 2, world_holder )
    }

    #[allow(dead_code)]
    pub fn set_99pc( world_holder:&mut dyn WorldHolder ) -> TestDataset {
        Self::set_n( WORLD_Z * WORLD_Y * WORLD_X - 1, world_holder )
    }

    #[allow(dead_code)]
    pub fn set_100_uniques( world_holder:&mut dyn WorldHolder ) -> TestDataset {
        Self::set_n_uniques( WORLD_Z * WORLD_Y * WORLD_X, world_holder )
    }

    #[allow(dead_code)]
    pub fn fill_50pc( world_holder:&mut dyn WorldHolder ) -> TestDataset {
        Self::fill( (0, 0, 0), (WORLD_Z, WORLD_Y, WORLD_X / 2), world_holder )
    }

    pub fn fill_50pc_realistically( world_holder:&mut dyn WorldHolder ) -> TestDataset {
        println!( "Filling base" );
        let mut dataset = Self::fill( (0, WORLD_Y / 2, 0), (WORLD_Z, WORLD_Y, WORLD_X), world_holder );

        println!( "Filling deposits" );
        let noise = SimplexNoise::new( 50 );
        let coal_key = String::from( "coal" );

        dataset.colors.insert( coal_key.clone(), Rc::new( Color { red:2, green:2, blue:2 } ) );
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

        for z in 0..WORLD_Z {
            for y in (WORLD_Y / 2)..WORLD_Y {
                for x in 0..WORLD_X {
                    let noise_value = noise.noise3d( x as f64, y as f64, z as f64 );

                    if noise_value > 0.85 {
                        world_holder.set_voxel( x, y, z, Some( coal.clone() ) );
                    }
                }
            }
        }

        dataset
    }

    #[allow(dead_code)]
    pub fn fill_100pc( world_holder:&mut dyn WorldHolder ) -> TestDataset {
        Self::fill( (0, 0, 0), (WORLD_Z, WORLD_Y, WORLD_X), world_holder )
    }

    fn set_n( n:u32, world_holder:&mut dyn WorldHolder ) -> TestDataset {
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

    fn set_n_random( n:u32, world_holder:&mut dyn WorldHolder ) -> TestDataset {
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

    fn set_n_uniques( n:u32, world_holder:&mut dyn WorldHolder ) -> TestDataset {
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

    fn fill( from:(u32, u32, u32), to:(u32, u32, u32), world_holder:&mut dyn WorldHolder ) -> TestDataset {
        let key = String::from( "default" );
        let materials = HashMap::from([ (key.clone(), Rc::new( Material { _density:100 } )) ]);
        let colors = HashMap::from([ (key.clone(), Rc::new( Color { red:50, green:50, blue:50 } )) ]);

        let common_voxel_dataset = HashMap::from([ (key.clone(), Rc::new( CommonVoxelData {
            _material:materials.get( &key ).unwrap().clone(),
            _color:colors.get( &key ).unwrap().clone(),
        } ) ) ]);

        let voxels = HashMap::from([ (key.clone(), Rc::new( Voxel {
            _common_data: common_voxel_dataset.get( &key ).unwrap().clone(),
            _individual_data: vec![],
        }) ) ]);

        let voxel = voxels.get( &key ).unwrap();

        world_holder.fill_voxels( from, to, Some( voxel.clone() ) );

        TestDataset { materials, colors, common_voxel_dataset, voxels }
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
