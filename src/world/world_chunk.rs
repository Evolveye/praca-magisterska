use std::sync::RwLockReadGuard;

use crate::{
    structure_tests::octree::Octree,
    world::{
        world::{ GridPosition, Position, CHUNK_SIZE, CHUNK_SIZE_X2, CHUNK_SIZE_X3 },
        world_holder::{ Voxel, VoxelSide, WorldHolding }
    }
};

#[derive(Debug)]
pub enum WorldChunkState {
    Empty,
    Dirty,
    Meshed,
    Calculable,
    Stashing,
    Disabled,
}

struct WorldChunkData {
    data: Octree<Voxel>,
    solids_mask: ChunkBitmask,
}

#[allow(dead_code)]
pub struct WorldChunk {
    structure: Option<WorldChunkData>,
    pub state: WorldChunkState,
    pub renderables: Vec<VoxelSide>,
}

impl WorldChunk {
    pub fn new() -> Self {

        Self {
            state: WorldChunkState::Empty,
            renderables: vec![],
            structure: None
        }
    }

    pub fn new_disabled() -> Self {
        let mut chunk = Self::new();
        chunk.state = WorldChunkState::Disabled;
        chunk
    }

    pub fn set_data( &mut self, data:Octree<Voxel> ) {
        self.structure = Some( WorldChunkData { solids_mask:data.to_bitmask(), data } );
        self.state = WorldChunkState::Dirty;
    }

    pub fn remesh( &mut self, offset:GridPosition, neighbours:Vec<RwLockReadGuard<'_, WorldChunk>> ) -> bool {
        let Some( ref structure ) = self.structure else { return false };

        if matches!( self.state, WorldChunkState::Meshed | WorldChunkState::Disabled ) {
            return false
        }

        // self.renderables = self.data.get_visible_with_flood( (0, self.data.get_size() as u32 - 1, 0) )
        //     .into_iter()
        //     .filter_map( |mut s| {
        //         // if s.get_position().x != 0.0 { return None }
        //         s.move_by( (offset.0 as f32, offset.1 as f32, offset.2 as f32) );
        //         Some( s )
        //     } )
        //     .collect::<Vec<VoxelSide>>();
        // return;

        // println!( "Remeshing chunk {:?}", offset );

        let world_offset = (
            offset.0 * CHUNK_SIZE as i64,
            offset.1 * CHUNK_SIZE as i64,
            offset.2 * CHUNK_SIZE as i64,
        );

        let mut renderables = vec![];
        let mut col_face_masks = vec![ 0; CHUNK_SIZE_X3 * 2 ];
        let neighbour_shift = CHUNK_SIZE - 1;
        let axies_neighbours = [
            (&neighbours[ 13 ], &neighbours[ 12 ]), // (right, left)
            (&neighbours[ 21 ], &neighbours[  4 ]), // (top,   bottom)
            (&neighbours[ 15 ], &neighbours[ 10 ]), // (front, back)
        ];

        for axis in 0..3 {
            for i in 0..CHUNK_SIZE_X2 {
                let index = CHUNK_SIZE_X2 * axis + i;
                let column = structure.solids_mask.data[ index ];

                let neighbour_a_shift = if let Some( ref neighbour_a_shift ) = axies_neighbours[ axis ].0.structure {
                    (neighbour_a_shift.solids_mask.data[ index ] & 1) << neighbour_shift
                } else {
                    match axies_neighbours[ axis ].0.state {
                        WorldChunkState::Disabled => 0,
                        _ => Self::panic_meshing_missing_neighbour( axies_neighbours[ axis ].0, offset, axis, -1 )
                    }
                };

                let neighbour_b_shift = if let Some( ref neighbour_b_shift ) = axies_neighbours[ axis ].1.structure {
                    (neighbour_b_shift.solids_mask.data[ index ] >> neighbour_shift) & 1
                } else {
                    match axies_neighbours[ axis ].1.state {
                        WorldChunkState::Disabled => 0,
                        _ => Self::panic_meshing_missing_neighbour( axies_neighbours[ axis ].1, offset, axis,  1 )
                    }
                };

                col_face_masks[ CHUNK_SIZE_X2 * (axis * 2    ) + i ] = column & !(column << 1 | neighbour_b_shift);
                col_face_masks[ CHUNK_SIZE_X2 * (axis * 2 + 1) + i ] = column & !(column >> 1 | neighbour_a_shift);
            }
        }

        // let mut collected = HashSet::new();

        for axis_turn in 0..6 {
            for x in 0..CHUNK_SIZE {
                for y in 0..CHUNK_SIZE {
                    let column_index = x + y * CHUNK_SIZE + CHUNK_SIZE_X2 * axis_turn;
                    let mut num = col_face_masks[ column_index ];

                    while num != 0 {
                        let z = num.trailing_zeros();

                        let voxel_pos = match axis_turn {
                            0 | 1 => (z, x as u32, y as u32), // y,z=x 1,2 X
                            2 | 3 => (x as u32, z, y as u32), // x,z=y 3,4 Y
                            _     => (x as u32, y as u32, z), // x,y=z 5,6 Z
                        };

                        // if collected.contains( &voxel_pos ) {
                        //     num &= num - 1;
                        //     continue
                        // }

                        // collected.insert( voxel_pos );

                        if let Some( voxel ) = structure.data.get( voxel_pos.0, voxel_pos.1, voxel_pos.2 ) {
                            renderables.push( VoxelSide::from_voxel_rc(
                                world_offset.0 + voxel_pos.0 as i64,
                                world_offset.1 + voxel_pos.1 as i64,
                                world_offset.2 + voxel_pos.2 as i64,
                                axis_turn as u8 + 1,
                                &voxel
                            ) );
                        }

                        num &= num - 1;
                    }
                }
            }
        }

        self.renderables = renderables;
        self.state = WorldChunkState::Meshed;

        true
    }

    fn panic_meshing_missing_neighbour( neighbour:&RwLockReadGuard<'_, WorldChunk>, chunk_pos:GridPosition, axis:usize, addition:i64 ) -> ! {
        let neighbour_pos = match axis {
            0 => (chunk_pos.0 - addition, chunk_pos.1, chunk_pos.2),
            1 => (chunk_pos.0, chunk_pos.1 + addition, chunk_pos.2),
            2 => (chunk_pos.0, chunk_pos.1, chunk_pos.2 - addition),
            _ => unreachable!(),
        };

        panic!( "Neighbours of remeshed chunk must have terrain (axis={axis}, addition={addition}, chunk_pos={chunk_pos:?}, neighbour_pos={neighbour_pos:?}, neighbour_state={:?})", neighbour.state )
    }

    #[allow(unused)]
    pub fn print_bitmask_layer( &self, layer:usize ) {
        let Some( ref structure ) = self.structure else { return };
        let size = structure.data.get_size() as usize;
        let mut num = 0;

        print!( "   " );
        for _ in 0..=1 {
            for _ in 0..size {
                print!( "{num}" );
                if num < 9 { num += 1 } else { num = 0 }
            }

            num = 0;
            print!( " " );
        }

        print!( "\n  ,{}+{}", "-".repeat( size ), "-".repeat( size ) );

        for z in 0..size {
            print!( "\n{num} |" );

            for x in 0..size {
                let bit = (structure.solids_mask.data[ z + (layer * CHUNK_SIZE) + CHUNK_SIZE_X2     ] >> x) & 1;
                print!("{}", if bit == 0 { " " } else { "#" } );
            }

            print!( "|" );

            for x in 0..size {
                let bit = (structure.solids_mask.data[ x + (layer * CHUNK_SIZE) + CHUNK_SIZE_X2 * 2 ] >> z) & 1;
                print!("{}", if bit == 0 { " " } else { "#" } );
            }

            if num < 9 { num += 1 } else { num = 0 }
        }

        println!();
    }

    pub fn get_chunk_position_from_world_position( world_position:Position ) -> GridPosition {
        let chunk_size = CHUNK_SIZE as i64;

        (
            (world_position.0 as i64).div_euclid( chunk_size ),
            (world_position.1 as i64).div_euclid( chunk_size ),
            (world_position.2 as i64).div_euclid( chunk_size ),
        )
    }
}

pub struct ChunkBitmask {
    pub data: Vec<u64>,
}

impl ChunkBitmask {
    pub fn new( size:usize ) -> Self {
        Self {
            data: vec![ 0; size ]
        }
    }
}
