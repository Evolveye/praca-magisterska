use std::{ collections::HashMap, mem::{size_of, size_of_val}, rc::Rc };

use rand::seq::IteratorRandom;

pub const RENDER_DISTANCE:u32 = 512;
pub const WORLD_Z:u32 = RENDER_DISTANCE * 2 + 1;
pub const WORLD_Y:u32 = 384;
pub const WORLD_X:u32 = RENDER_DISTANCE * 2 + 1;

pub struct Material {
    density: u32
}
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

pub struct CommonVoxelData {
    pub material: Rc<Material>,
    pub color: Rc<Color>,
}

pub struct Voxel {
    pub _individual_data: Vec<String>,
    pub common_data: Rc<CommonVoxelData>,
}

pub trait WorldHolder {
    fn get_voxel( &self, x:u32, y:u32, z:u32 ) -> Option<Rc<Voxel>>;
    fn set_voxel( &mut self, x:u32, y:u32, z:u32, voxel:Rc<Voxel> );
    fn get_size( &self );
}

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
            size_of::<Rc<Color>>(),
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
    pub fn fill_0( &self, _world_holder:&dyn WorldHolder ) -> TestDataset {
        TestDataset {
            materials: HashMap::new(),
            colors: HashMap::new(),
            common_voxel_dataset: HashMap::new(),
            voxels: HashMap::new(),
        }
    }

    pub fn fill_1( &self, world_holder:&mut dyn WorldHolder ) -> TestDataset {
        let key = String::from( "default" );
        let materials = HashMap::from([ (key.clone(), Rc::new( Material { density:100 } )) ]);

        let colors = HashMap::from([ (key.clone(), Rc::new( Color { red:50, green:100, blue:200 } )) ]);
        let common_voxel_dataset = HashMap::from([ (key.clone(), Rc::new( CommonVoxelData {
            material:materials.get( &key ).unwrap().clone(),
            color:colors.get( &key ).unwrap().clone(),
        } ) ) ]);

        let voxels = HashMap::from([ (key.clone(), Rc::new( Voxel {
            common_data: common_voxel_dataset.get( &key ).unwrap().clone(),
            _individual_data: vec![],
        }) ) ]);

        world_holder.set_voxel( 0, 0, 0, voxels.get( &key ).unwrap().clone() );

        let test_dataset = TestDataset { materials, colors, common_voxel_dataset, voxels };

        test_dataset
    }

    pub fn fill_50( &self, world_holder:&mut dyn WorldHolder ) -> TestDataset {
        todo!()
    }
    pub fn fill_50_random( &self, world_holder:&mut dyn WorldHolder ) -> TestDataset {
        self.fill_n_random( WORLD_Z * WORLD_Y * WORLD_X / 2, world_holder )
    }
    pub fn fill_50_diff( &self, world_holder:&mut dyn WorldHolder ) -> TestDataset {
        self.fill_n_diff( WORLD_Z / 2, WORLD_Y / 2, WORLD_X / 2, world_holder )
    }
    pub fn fill_99( &self, world_holder:&mut dyn WorldHolder ) -> TestDataset {
        let dataset = self.fill_n( WORLD_Z - 1, WORLD_Y, WORLD_X, world_holder );
        let key = String::from( "default" );

        for y in 0..(WORLD_Y - 1) {
            for x in 0..WORLD_X {
                world_holder.set_voxel( x, y, WORLD_Z - 1, dataset.voxels.get( &key ).unwrap().clone() );
            }
        }

        for x in 0..(WORLD_X - 1) {
            world_holder.set_voxel( x, WORLD_Y - 1, WORLD_Z - 1, dataset.voxels.get( &key ).unwrap().clone() );
        }

        dataset
    }
    pub fn fill_100( &self, world_holder:&mut dyn WorldHolder ) -> TestDataset {
        self.fill_n( WORLD_Z, WORLD_Y, WORLD_X, world_holder )
    }
    pub fn fill_100_diff( &self, world_holder:&mut dyn WorldHolder ) -> TestDataset {
        self.fill_n_diff( WORLD_Z, WORLD_Y, WORLD_X, world_holder )
    }

    fn fill_n( &self, max_z:u32, max_y:u32, max_x:u32, world_holder:&mut dyn WorldHolder ) -> TestDataset {
        let key = String::from( "default" );
        let materials = HashMap::from([ (key.clone(), Rc::new( Material { density:100 } )) ]);
        let colors = HashMap::from([ (key.clone(), Rc::new( Color { red:50, green:100, blue:200 } )) ]);

        let common_voxel_dataset = HashMap::from([ (key.clone(), Rc::new( CommonVoxelData {
            material:materials.get( &key ).unwrap().clone(),
            color:colors.get( &key ).unwrap().clone(),
        } ) ) ]);

        let voxels = HashMap::from([ (key.clone(), Rc::new( Voxel {
            common_data: common_voxel_dataset.get( &key ).unwrap().clone(),
            _individual_data: vec![],
        }) ) ]);

        for z in 0..max_z {
            for y in 0..max_y {
                for x in 0..max_x {
                    world_holder.set_voxel( x, y, z, voxels.get( &key ).unwrap().clone() );
                }
            }

            println!( " z={}", z );
        }

        let test_dataset = TestDataset { materials, colors, common_voxel_dataset, voxels };

        test_dataset
    }

    fn fill_n_random( &self, n:u32, world_holder:&mut dyn WorldHolder ) -> TestDataset {
        let key = String::from( "default" );
        let materials = HashMap::from([ (key.clone(), Rc::new( Material { density:100 } )) ]);
        let colors = HashMap::from([ (key.clone(), Rc::new( Color { red:50, green:100, blue:200 } )) ]);

        let common_voxel_dataset = HashMap::from([ (key.clone(), Rc::new( CommonVoxelData {
            material:materials.get( &key ).unwrap().clone(),
            color:colors.get( &key ).unwrap().clone(),
        } ) ) ]);

        let voxels = HashMap::from([ (key.clone(), Rc::new( Voxel {
            common_data: common_voxel_dataset.get( &key ).unwrap().clone(),
            _individual_data: vec![],
        }) ) ]);

        let mut rng = rand::rng();
        let random = (0..(WORLD_Z * WORLD_Y * WORLD_X)).choose_multiple( &mut rng, n as usize );

        for num in random {
            let z = num / (WORLD_Y * WORLD_Z);
            let y = (num % (WORLD_Y * WORLD_Z)) / WORLD_Z;
            let x = num % WORLD_Z;

            world_holder.set_voxel( x, y, z, voxels.get( &key ).unwrap().clone() );
        }

        let test_dataset = TestDataset { materials, colors, common_voxel_dataset, voxels };

        test_dataset
    }

    fn fill_n_diff( &self, max_z:u32, max_y:u32, max_x:u32, world_holder:&mut dyn WorldHolder ) -> TestDataset {
        let mut materials = HashMap::new();
        let mut colors = HashMap::new();
        let mut common_voxel_dataset = HashMap::new();
        let mut voxels:HashMap<_, Rc<Voxel>> = HashMap::new();

        println!( "Loop generating world (max_z={}; max_y={}; max_x={}):", max_z, max_y, max_x );

        for z in 0..max_z {
            for y in 0..max_y {
                for x in 0..max_x {
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
                            materials.insert( material_key.clone(), Rc::new( Material { density } ) );
                            materials.get( &material_key ).unwrap()
                        }
                    };

                    let common_voxel_dataset_key = format!( "{}-{}", color_key, material_key );
                    let common_data = match common_voxel_dataset.get( &common_voxel_dataset_key ) {
                        Some( common_data ) => common_data,
                        None => {
                            common_voxel_dataset.insert( common_voxel_dataset_key.clone(), Rc::new( CommonVoxelData { color:color.clone(), material:material.clone() } ) );
                            common_voxel_dataset.get( &common_voxel_dataset_key ).unwrap()
                        }
                    };

                    let voxel_key = format!( "{}-{}", common_voxel_dataset_key, material_key );
                    let voxel = match voxels.get( &voxel_key ) {
                        Some( voxel ) => voxel,
                        None => {
                            voxels.insert( voxel_key.clone(), Rc::new( Voxel { common_data:common_data.clone(), _individual_data:vec![ material_key ] } ) );
                            voxels.get( &voxel_key ).unwrap()
                        }
                    };

                    world_holder.set_voxel( x, y, z, voxel.clone() );
                }
            }

            println!( " z={};", z );
        }

        let test_dataset = TestDataset { materials, colors, common_voxel_dataset, voxels };

        test_dataset
    }
}
