use std::rc::Rc;

use cgmath::Vector3;

use crate::rendering::vertex::Vec3;

pub type Coordinate = u32;

#[derive(Debug)]
pub struct Material {
    pub _density: u32
}

#[derive(Debug, Clone)]
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
#[derive(Debug)]
pub struct VoxelSide {
    pos: Vec3,
    color: Color,
    direction: u8,
}

impl VoxelSide {
    pub fn from_voxel_rc( x:u32, y:u32, z:u32, direction:u8, voxel:&Rc<Voxel> ) -> Self {
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
}

pub trait WorldHolder {
    fn get_voxel( &self, x:Coordinate, y:Coordinate, z:Coordinate ) -> Option<Rc<Voxel>>;
    fn get_all_voxels( &self ) -> Vec<(u32, u32, u32, Rc<Voxel>)>;
    fn get_all_visible_voxels_from( &self, from:(Coordinate, Coordinate, Coordinate) ) -> Vec<VoxelSide>;

    fn set_voxel( &mut self, x:Coordinate, y:Coordinate, z:Coordinate, voxel:Option<Rc<Voxel>> );
    fn fill_voxels( &mut self, from:(Coordinate, Coordinate, Coordinate), to:(Coordinate, Coordinate, Coordinate), voxel:Option<Rc<Voxel>> );

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
