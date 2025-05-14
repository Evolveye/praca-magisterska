use std::rc::Rc;

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

pub trait WorldHolder {
    fn get_voxel( &self, x:Coordinate, y:Coordinate, z:Coordinate ) -> Option<Rc<Voxel>>;
    fn get_all_voxels( &self ) -> Vec<(u32, u32, u32, Rc<Voxel>)>;
    fn get_all_visible_voxels_from( &self, from:(Coordinate, Coordinate, Coordinate) ) -> Vec<(u32, u32, u32, Rc<Voxel>)>;

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
