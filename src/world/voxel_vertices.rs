use cgmath::vec3;
use crate::rendering::vertex::{SimpleVertex, Vec3};

#[allow(dead_code)]
pub const VOXEL_VERTICES:[ VoxelVertex; 24 ] = [
    // Front (+Z)
    VoxelVertex { pos:vec3(-0.5, -0.5,  0.5), normal:vec3(0.0, 0.0, 1.0), },
    VoxelVertex { pos:vec3( 0.5, -0.5,  0.5), normal:vec3(0.0, 0.0, 1.0), },
    VoxelVertex { pos:vec3( 0.5,  0.5,  0.5), normal:vec3(0.0, 0.0, 1.0), },
    VoxelVertex { pos:vec3(-0.5,  0.5,  0.5), normal:vec3(0.0, 0.0, 1.0), },

    // Back (-Z)
    VoxelVertex { pos:vec3( 0.5, -0.5, -0.5), normal:vec3(0.0, 0.0, -1.0) },
    VoxelVertex { pos:vec3(-0.5, -0.5, -0.5), normal:vec3(0.0, 0.0, -1.0) },
    VoxelVertex { pos:vec3(-0.5,  0.5, -0.5), normal:vec3(0.0, 0.0, -1.0) },
    VoxelVertex { pos:vec3( 0.5,  0.5, -0.5), normal:vec3(0.0, 0.0, -1.0) },

    // Left (-X)
    VoxelVertex { pos:vec3(-0.5, -0.5, -0.5), normal:vec3(-1.0, 0.0, 0.0) },
    VoxelVertex { pos:vec3(-0.5, -0.5,  0.5), normal:vec3(-1.0, 0.0, 0.0) },
    VoxelVertex { pos:vec3(-0.5,  0.5,  0.5), normal:vec3(-1.0, 0.0, 0.0) },
    VoxelVertex { pos:vec3(-0.5,  0.5, -0.5), normal:vec3(-1.0, 0.0, 0.0) },

    // Right (+X)
    VoxelVertex { pos:vec3( 0.5, -0.5,  0.5), normal:vec3(1.0, 0.0, 0.0), },
    VoxelVertex { pos:vec3( 0.5, -0.5, -0.5), normal:vec3(1.0, 0.0, 0.0), },
    VoxelVertex { pos:vec3( 0.5,  0.5, -0.5), normal:vec3(1.0, 0.0, 0.0), },
    VoxelVertex { pos:vec3( 0.5,  0.5,  0.5), normal:vec3(1.0, 0.0, 0.0), },

    // Top (+Y)
    VoxelVertex { pos:vec3(-0.5,  0.5,  0.5), normal:vec3(0.0, 1.0, 0.0), },
    VoxelVertex { pos:vec3( 0.5,  0.5,  0.5), normal:vec3(0.0, 1.0, 0.0), },
    VoxelVertex { pos:vec3( 0.5,  0.5, -0.5), normal:vec3(0.0, 1.0, 0.0), },
    VoxelVertex { pos:vec3(-0.5,  0.5, -0.5), normal:vec3(0.0, 1.0, 0.0), },

    // Bottom (-Y)
    VoxelVertex { pos:vec3(-0.5, -0.5, -0.5), normal:vec3(0.0, -1.0, 0.0) },
    VoxelVertex { pos:vec3( 0.5, -0.5, -0.5), normal:vec3(0.0, -1.0, 0.0) },
    VoxelVertex { pos:vec3( 0.5, -0.5,  0.5), normal:vec3(0.0, -1.0, 0.0) },
    VoxelVertex { pos:vec3(-0.5, -0.5,  0.5), normal:vec3(0.0, -1.0, 0.0) },
];

#[allow(dead_code)]
pub const VOXEL_INDICES:&[ u32; 36 ] = &[
    0, 1, 2, 2, 3, 0,        // front
    4, 5, 6, 6, 7, 4,        // back
    8, 9, 10,10,11,8,        // left
    12,13,14,14,15,12,       // right
    16,17,18,18,19,16,       // top
    20,21,22,22,23,20,       // bottom
];

#[allow(dead_code)]
pub const VOXEL_SIDE_VERTICES:[ VoxelVertex; 4 ] = [
    // Top (+Y)
    VoxelVertex { pos:vec3(-0.5,  0.5,  0.5), normal:vec3(0.0, 1.0, 0.0), },
    VoxelVertex { pos:vec3( 0.5,  0.5,  0.5), normal:vec3(0.0, 1.0, 0.0), },
    VoxelVertex { pos:vec3(-0.5,  0.5, -0.5), normal:vec3(0.0, 1.0, 0.0), },
    VoxelVertex { pos:vec3( 0.5,  0.5, -0.5), normal:vec3(0.0, 1.0, 0.0), },

    // VoxelVertex { pos:vec3(-0.5,  0.5,  0.5), normal:vec3(0.0, 1.0, 0.0), },
    // VoxelVertex { pos:vec3( 0.5,  0.5,  0.5), normal:vec3(0.0, 1.0, 0.0), },
    // VoxelVertex { pos:vec3( 0.0,  0.5,  0.0), normal:vec3(0.0, 1.0, 0.0), },
    // VoxelVertex { pos:vec3( 0.5,  0.5, -0.5), normal:vec3(0.0, 1.0, 0.0), },
    // VoxelVertex { pos:vec3(-0.5,  0.5, -0.5), normal:vec3(0.0, 1.0, 0.0), },
    // VoxelVertex { pos:vec3(-0.5,  0.5,  0.25), normal:vec3(0.0, 1.0, 0.0), },
];

#[allow(dead_code)]
pub const VOXEL_SIDE_INDICES:&[ u32; 6 ] = &[
    0, 1, 2, 2, 3, 0,        // Top
];

#[allow(dead_code)]
pub const VOXEL_CORNERS:&[ SimpleVertex; 8 ] = &[
    // Back (-Z)
    SimpleVertex { pos:vec3(-0.5, -0.5, -0.5), color:vec3( 1.0, 1.0, 1.0 ) },
    SimpleVertex { pos:vec3( 0.5, -0.5, -0.5), color:vec3( 1.0, 1.0, 1.0 ) },
    SimpleVertex { pos:vec3( 0.5,  0.5, -0.5), color:vec3( 1.0, 1.0, 1.0 ) },
    SimpleVertex { pos:vec3(-0.5,  0.5, -0.5), color:vec3( 1.0, 1.0, 1.0 ) },

    // Front (+Z)
    SimpleVertex { pos:vec3(-0.5, -0.5,  0.5), color:vec3( 1.0, 1.0, 1.0 ), },
    SimpleVertex { pos:vec3( 0.5, -0.5,  0.5), color:vec3( 1.0, 1.0, 1.0 ), },
    SimpleVertex { pos:vec3( 0.5,  0.5,  0.5), color:vec3( 1.0, 1.0, 1.0 ), },
    SimpleVertex { pos:vec3(-0.5,  0.5,  0.5), color:vec3( 1.0, 1.0, 1.0 ), },
];

#[allow(dead_code)]
pub const VOXEL_EDGES_INDICES:&[ u32; 24 ] = &[
    0,1, 1,2, 2,3, 3,0,   // front face edges
    4,5, 5,6, 6,7, 7,4,   // back face edges
    // 0,5, 1,4, 2,7, 3,6,   // connections front <-> back
    0,4, 1,5, 2,6, 3,7,   // connections front <-> back
];

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct VoxelVertex {
    pub pos: Vec3,
    pub normal: Vec3,
}

impl Into<SimpleVertex> for VoxelVertex {
    fn into( self ) -> SimpleVertex {
        SimpleVertex {
            pos: self.pos,
            color: self.normal,
        }
    }
}
