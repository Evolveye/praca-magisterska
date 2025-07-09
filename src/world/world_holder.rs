use std::{collections::HashMap, rc::Rc};

use cgmath::Vector3;

use crate::{rendering::vertex::Vec3, world::world_chunk::ChunkBitmask};

pub type Coordinate = u32;

pub struct VoxelDataset {
    pub materials: HashMap<String, Rc<Material>>,
    pub colors: HashMap<String, Rc<Color>>,
    pub common_voxel_dataset: HashMap<String, Rc<CommonVoxelData>>,
    pub voxels: HashMap<String, Rc<Voxel>>,
}

#[allow(dead_code)]
impl VoxelDataset {
    pub fn new() -> Self {
        Self {
            colors: HashMap::new(),
            materials: HashMap::new(),
            common_voxel_dataset: HashMap::new(),
            voxels: HashMap::new(),
        }
    }
    pub fn expand( &mut self, dataset:Self ) {
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

#[derive(Debug)]
pub struct Material {
    pub _density: u32
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[derive(Debug)]
pub struct CommonVoxelData {
    pub _material: Rc<Material>,
    pub _color: Rc<Color>,
}

#[derive(Debug)]
pub struct Voxel {
    pub _individual_data: Vec<String>,
    pub _common_data: Rc<CommonVoxelData>,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VoxelSide {
    pos: Vec3,
    color: Color,
    direction: u8,
}

#[allow(dead_code)]
impl VoxelSide {
    pub fn new( pos:Vec3, color:Color, direction:u8 ) -> Self {
        Self { pos, color, direction }
    }

    pub fn from_voxel_rc( x:i64, y:i64, z:i64, direction:u8, voxel:&Rc<Voxel> ) -> Self {
        Self {
            pos: Vector3::new( x as f32, y as f32, z as f32 ),
            direction,
            color: (*voxel._common_data._color).clone(),
        }
    }

    pub fn get_color( &self ) -> Color {
        self.color.clone()
    }
    pub fn get_position( &self ) -> Vec3 {
        self.pos.clone()
    }
    pub fn move_by( &mut self, vec:(f32, f32, f32)) {
        self.pos.x += vec.0;
        self.pos.y += vec.1;
        self.pos.z += vec.2;
    }
}

#[allow(dead_code)]
pub trait WorldHolding {
    fn get_voxel( &self, x:Coordinate, y:Coordinate, z:Coordinate ) -> Option<Rc<Voxel>>;
    fn get_all_voxels( &self ) -> Vec<(u32, u32, u32, Rc<Voxel>)>;
    fn get_all_visible_voxels_from( &self, from:(Coordinate, Coordinate, Coordinate) ) -> Vec<VoxelSide>;

    fn set_voxel( &mut self, x:Coordinate, y:Coordinate, z:Coordinate, voxel:Option<Rc<Voxel>> );
    fn fill_voxels( &mut self, from:(Coordinate, Coordinate, Coordinate), to:(Coordinate, Coordinate, Coordinate), voxel:Option<Rc<Voxel>> );

    fn to_bitmask( &self ) -> ChunkBitmask;
    fn get_size( &self );
    fn get_bytes_with_prefixes( &self, bytes:usize ) -> String {
        match bytes {
            size if size / 1024 / 1024 / 1024 > 0 => format!( "{size} B = {} KiB = {} MiB = {} GiB", size / 1024, size / 1024 / 1024, size / 1024 / 1024 / 1024 ),
            size if size / 1024 / 1024 > 0 => format!( "{size} B = {} KiB = {} MiB", size / 1024, size / 1024 / 1024 ),
            size if size / 1024 > 0 => format!( "{size} B = {} KiB", size / 1024 ),
            size => format!( "{size} B" ),
        }
    }
}

#[allow(dead_code)]
pub fn fill_with( from:(u32, u32, u32), to:(u32, u32, u32), world_holder:&mut dyn WorldHolding, setup:(&str, Color) ) -> VoxelDataset {
    let materials = HashMap::from([ (setup.0.to_string(), Rc::new( Material { _density:100 } )) ]);
    let colors = HashMap::from([ (setup.0.to_string(), Rc::new( setup.1 )) ]);

    let common_voxel_dataset = HashMap::from([ (setup.0.to_string(), Rc::new( CommonVoxelData {
        _material: materials.get( setup.0 ).unwrap().clone(),
        _color: colors.get( setup.0 ).unwrap().clone(),
    } ) ) ]);

    let voxels = HashMap::from([ (setup.0.to_string(), Rc::new( Voxel {
        _common_data: common_voxel_dataset.get( setup.0 ).unwrap().clone(),
        _individual_data: vec![],
    }) ) ]);

    let voxel = voxels.get( setup.0 ).unwrap();

    world_holder.fill_voxels( from, to, Some( voxel.clone() ) );

    VoxelDataset { materials, colors, common_voxel_dataset, voxels }
}

#[allow(dead_code)]
pub fn fill( from:(u32, u32, u32), to:(u32, u32, u32), world_holder:&mut dyn WorldHolding ) -> VoxelDataset {
    fill_with( from, to, world_holder, ("default", Color { red:50, green:50, blue:50 }) )
}
