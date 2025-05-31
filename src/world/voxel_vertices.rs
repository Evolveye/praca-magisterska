use cgmath::vec3;

use crate::rendering::vertex::Vec3;

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

pub const VOXEL_INDICES:&[ u32; 36 ] = &[
    0, 1, 2, 2, 3, 0,        // front
    4, 5, 6, 6, 7, 4,        // back
    8, 9, 10,10,11,8,        // left
    12,13,14,14,15,12,       // right
    16,17,18,18,19,16,       // top
    20,21,22,22,23,20,       // bottom
];

pub const VOXEL_SIDE_VERTICES:[ VoxelVertex; 4 ] = [
    // Top (+Y)
    VoxelVertex { pos:vec3(-0.5,  0.5,  0.5), normal:vec3(0.0, 1.0, 0.0), },
    VoxelVertex { pos:vec3( 0.5,  0.5,  0.5), normal:vec3(0.0, 1.0, 0.0), },
    VoxelVertex { pos:vec3( 0.5,  0.5, -0.5), normal:vec3(0.0, 1.0, 0.0), },
    VoxelVertex { pos:vec3(-0.5,  0.5, -0.5), normal:vec3(0.0, 1.0, 0.0), },
];

pub const VOXEL_SIDE_INDICES:&[ u32; 6 ] = &[
    0, 1, 2, 2, 3, 0,        // Top
];

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct VoxelVertex {
    pub pos: Vec3,
    pub normal: Vec3,
}
