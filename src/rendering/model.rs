use anyhow::Result;
use cgmath::{ vec2, vec3 };
use std::{
  collections::HashMap,
  io::BufReader,
  fs::File,
};
use super::vertex::Vertex;

type Vec3 = cgmath::Vector3<f32>;
type Mat4 = cgmath::Matrix4<f32>;

static VERTICES:[ Vertex; 8 ] = [
  // Vertex::new( vec3( -1.0, -1.0, -1.0 ), vec3( 1.0, 0.0, 0.0 ), vec2( 1.0, 0.0 ) ),
  // Vertex::new( vec3(  1.0, -1.0, -1.0 ), vec3( 0.0, 1.0, 0.0 ), vec2( 0.0, 0.0 ) ),
  // Vertex::new( vec3(  1.0, -1.0,  1.0 ), vec3( 0.0, 1.0, 0.0 ), vec2( 0.0, 0.0 ) ),
  // Vertex::new( vec3( -1.0, -1.0,  1.0 ), vec3( 1.0, 0.0, 0.0 ), vec2( 1.0, 0.0 ) ),

  // Vertex::new( vec3( -1.0,  1.0, -1.0 ), vec3( 0.0, 0.0, 1.0 ), vec2( 0.0, 1.0 ) ),
  // Vertex::new( vec3(  1.0,  1.0, -1.0 ), vec3( 1.0, 1.0, 1.0 ), vec2( 1.0, 1.0 ) ),
  // Vertex::new( vec3(  1.0,  1.0,  1.0 ), vec3( 1.0, 1.0, 1.0 ), vec2( 1.0, 1.0 ) ),
  // Vertex::new( vec3( -1.0,  1.0,  1.0 ), vec3( 0.0, 0.0, 1.0 ), vec2( 0.0, 1.0 ) ),

  Vertex::new( vec3(  1.0, -1.0, -1.0 ), vec3( 0.0, 1.0, 0.0 ), vec2( 0.0, 0.0 ) ),
  Vertex::new( vec3(  1.0, -1.0,  1.0 ), vec3( 0.0, 1.0, 0.0 ), vec2( 0.0, 0.0 ) ),
  Vertex::new( vec3( -1.0, -1.0,  1.0 ), vec3( 1.0, 0.0, 0.0 ), vec2( 1.0, 0.0 ) ),
  Vertex::new( vec3( -1.0, -1.0, -1.0 ), vec3( 1.0, 0.0, 0.0 ), vec2( 1.0, 0.0 ) ),

  Vertex::new( vec3(  1.0,  1.0, -1.0 ), vec3( 1.0, 1.0, 1.0 ), vec2( 1.0, 1.0 ) ),
  Vertex::new( vec3(  1.0,  1.0,  1.0 ), vec3( 1.0, 1.0, 1.0 ), vec2( 1.0, 1.0 ) ),
  Vertex::new( vec3( -1.0,  1.0,  1.0 ), vec3( 0.0, 0.0, 1.0 ), vec2( 0.0, 1.0 ) ),
  Vertex::new( vec3( -1.0,  1.0, -1.0 ), vec3( 0.0, 0.0, 1.0 ), vec2( 0.0, 1.0 ) ),
];

const INDICES:&[ u32 ] = &[
  1, 2, 0, 2, 3, 0, // bottom
  5, 4, 6, 4, 7, 6, // up
  1, 0, 5, 0, 4, 5, // back
  0, 3, 4, 3, 7, 4, // left
  3, 2, 7, 2, 6, 7, // front
  2, 1, 6, 1, 5, 6, // right

  // 1, 2, 3,
  // 7, 6, 5,
  // 4, 5, 1,
  // 5, 6, 2,
];

pub struct Model {
  pub vertices: Vec<Vertex>,
  pub indices: Vec<u32>,
}

impl Model {
  #[allow(dead_code)]
  pub fn from_file( src:&str ) -> Result<Self> {
    let mut model = Self {
      vertices: vec![],
      indices: vec![],
    };

    let mut unique_vertices = HashMap::new();
    let mut reader = BufReader::new( File::open( src )? );

    let ( models, _ ) = tobj::load_obj_buf(
      &mut reader,
      &tobj::LoadOptions { triangulate:true, single_index:true, ..Default::default() },
      |_| std::result::Result::Ok( Default::default() ),
    )?;

    let get_min_max = |a:(f32, f32), b:f32| (
      if a.0 > b { b } else { a.0 },
      if a.1 > b { a.1 } else { b },
    );

    for loaded_model in models {
      let mut min_max_x = (0.0, 0.0);
      let mut min_max_y = (0.0, 0.0);
      let mut min_max_z = (0.0, 0.0);

      for index in loaded_model.mesh.indices.clone() {
        let pos_offset = (3 * index) as usize;

        min_max_x = get_min_max( min_max_x, loaded_model.mesh.positions[ pos_offset + 0 ] );
        min_max_y = get_min_max( min_max_y, loaded_model.mesh.positions[ pos_offset + 1 ] );
        min_max_z = get_min_max( min_max_z, loaded_model.mesh.positions[ pos_offset + 2 ] );
      }

      for index in loaded_model.mesh.indices {
        let pos_offset = (3 * index) as usize;
        let tex_coord_offset = (2 * index) as usize;

        let vertex = Vertex {
          pos: vec3(
            loaded_model.mesh.positions[ pos_offset + 0 ],
            loaded_model.mesh.positions[ pos_offset + 1 ],
            loaded_model.mesh.positions[ pos_offset + 2 ],
          ),
          color: vec3( 1.0, 1.0, 1.0 ),
          tex_coord: if loaded_model.mesh.texcoords.is_empty() { vec2( 0.0, 0.0 ) } else {
            vec2(
              loaded_model.mesh.texcoords[ tex_coord_offset + 0 ],
              1.0 - loaded_model.mesh.texcoords[ tex_coord_offset + 1 ],
            )
          },
        };

        if let Some( index ) = unique_vertices.get( &vertex ) {
          model.indices.push( *index as u32 )
        } else {
          let index = model.vertices.len();
          unique_vertices.insert( vertex, index );
          model.vertices.push( vertex );
          model.indices.push( index as u32 )
        }
      }
    }

    Ok( model )
  }

  #[allow(dead_code)]
  pub fn new_cube() -> Self{
    Self {
      vertices: VERTICES.into(),
      indices: INDICES.into(),
    }
  }
}
