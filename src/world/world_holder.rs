use std::rc::Rc;

#[derive(Debug)]
pub struct Material {
    pub _density: u32
}
#[derive(Debug)]
pub struct Color {
    pub _red: u8,
    pub _green: u8,
    pub _blue: u8,
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

pub trait WorldHolder {
    fn get_voxel( &self, x:u32, y:u32, z:u32 ) -> Option<Rc<Voxel>>;

    fn set_voxel( &mut self, x:u32, y:u32, z:u32, voxel:Option<Rc<Voxel>> );
    fn fill_voxels( &mut self, from:(u32, u32, u32), to:(u32, u32, u32), voxel:Option<Rc<Voxel>> );

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
