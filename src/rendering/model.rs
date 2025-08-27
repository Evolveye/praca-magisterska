use anyhow::Result;
use cgmath::{ vec2, vec3 };
use std::os::raw::c_void;
use std::{
  collections::HashMap,
  io::BufReader,
  fs::File,
  mem::size_of,
  ptr::copy_nonoverlapping as memcpy,
};
use vulkanalia::{
  vk,
  prelude::v1_0::*
};

use crate::world::voxel_vertices::VoxelVertex;

use super::renderer::Renderer;
use super::vertex::{Renderable, RendererModelDescriptions, Vertex};
use super::buffer::{ create_buffer, copy_buffer };

type Vec3 = cgmath::Vector3<f32>;
type Mat4 = cgmath::Matrix4<f32>;

pub const BOX_VERTICES:[ Vertex; 8 ] = [
  // Vertex::new( vec3( -1.0, -1.0, -1.0 ), vec3( 1.0, 0.0, 0.0 ), vec2( 1.0, 0.0 ) ),
  // Vertex::new( vec3(  1.0, -1.0, -1.0 ), vec3( 0.0, 1.0, 0.0 ), vec2( 0.0, 0.0 ) ),
  // Vertex::new( vec3(  1.0, -1.0,  1.0 ), vec3( 0.0, 1.0, 0.0 ), vec2( 0.0, 0.0 ) ),
  // Vertex::new( vec3( -1.0, -1.0,  1.0 ), vec3( 1.0, 0.0, 0.0 ), vec2( 1.0, 0.0 ) ),

  // Vertex::new( vec3( -1.0,  1.0, -1.0 ), vec3( 0.0, 0.0, 1.0 ), vec2( 0.0, 1.0 ) ),
  // Vertex::new( vec3(  1.0,  1.0, -1.0 ), vec3( 1.0, 1.0, 1.0 ), vec2( 1.0, 1.0 ) ),
  // Vertex::new( vec3(  1.0,  1.0,  1.0 ), vec3( 1.0, 1.0, 1.0 ), vec2( 1.0, 1.0 ) ),
  // Vertex::new( vec3( -1.0,  1.0,  1.0 ), vec3( 0.0, 0.0, 1.0 ), vec2( 0.0, 1.0 ) ),

  Vertex::new( vec3(  1.0,  0.0,  0.0 ), vec3( 1.0, 1.0, 1.0 ), vec3(  1.0, -1.0, -1.0 ), vec2( 1.0, 1.0 ) ),
  Vertex::new( vec3(  1.0,  0.0,  1.0 ), vec3( 1.0, 1.0, 1.0 ), vec3(  1.0, -1.0,  1.0 ), vec2( 1.0, 1.0 ) ),
  Vertex::new( vec3(  0.0,  0.0,  1.0 ), vec3( 1.0, 1.0, 1.0 ), vec3( -1.0, -1.0,  1.0 ), vec2( 0.0, 1.0 ) ),
  Vertex::new( vec3(  0.0,  0.0,  0.0 ), vec3( 1.0, 1.0, 1.0 ), vec3( -1.0, -1.0, -1.0 ), vec2( 0.0, 1.0 ) ),

  Vertex::new( vec3(  1.0,  1.0,  0.0 ), vec3( 1.0, 1.0, 1.0 ), vec3(  1.0,  1.0, -1.0 ), vec2( 1.0, 0.0 ) ),
  Vertex::new( vec3(  1.0,  1.0,  1.0 ), vec3( 1.0, 1.0, 1.0 ), vec3(  1.0,  1.0,  1.0 ), vec2( 1.0, 0.0 ) ),
  Vertex::new( vec3(  0.0,  1.0,  1.0 ), vec3( 1.0, 1.0, 1.0 ), vec3( -1.0,  1.0,  1.0 ), vec2( 0.0, 0.0 ) ),
  Vertex::new( vec3(  0.0,  1.0,  0.0 ), vec3( 1.0, 1.0, 1.0 ), vec3( -1.0,  1.0, -1.0 ), vec2( 0.0, 0.0 ) ),
];

pub const BOX_INDICES:&[ u32 ] = &[
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

#[derive(Clone, Debug, Default)]
#[allow(dead_code)]
pub struct Model<TVertex> {
  pub vertices: Vec<TVertex>,
  pub indices: Vec<u32>,
  pub vertex_buffer: vk::Buffer,
  pub vertex_buffer_memory: vk::DeviceMemory,
  pub vertex_buffer_capacity: u64,
  pub vertices_count: u32,
  pub vertex_staging_buffer: vk::Buffer,
  pub vertex_staging_buffer_memory: vk::DeviceMemory,
  pub vertex_staging_mapped_memory: *mut c_void,
  pub index_buffer: vk::Buffer,
  pub index_buffer_memory: vk::DeviceMemory,
  pub instance_buffer: vk::Buffer,
  pub instance_buffer_memory: vk::DeviceMemory,
  pub instance_buffer_capacity: u64,
  pub instances_count: u32,
  pub instance_staging_buffer: vk::Buffer,
  pub instance_staging_buffer_memory: vk::DeviceMemory,
  pub instance_staging_mapped_memory: *mut c_void,
}

#[allow(dead_code)]
impl<TVertex> Model<TVertex> {
  pub unsafe fn new( renderer:&Renderer, vertices:Vec<TVertex>, indices:Vec<u32> ) -> Result<Self> {
    let ( index_buffer, index_buffer_memory ) = Model::<TVertex>::create_index_buffer( renderer, &indices )?;

    let mut model = Self {
      vertices: vec![],
      indices,
      vertex_buffer: Default::default(),
      vertex_buffer_memory: Default::default(),
      vertex_buffer_capacity: 0,
      vertices_count: 0,
      vertex_staging_buffer: Default::default(),
      vertex_staging_buffer_memory: Default::default(),
      vertex_staging_mapped_memory: Default::default(),
      index_buffer,
      index_buffer_memory,
      instance_buffer: Default::default(),
      instance_buffer_memory: Default::default(),
      instance_buffer_capacity: 0,
      instances_count: 0,
      instance_staging_buffer: Default::default(),
      instance_staging_buffer_memory: Default::default(),
      instance_staging_mapped_memory: Default::default(),
    };

    model.update_vertex_buffer::<TVertex>( renderer, vertices )?;
    model.update_instance_buffer::<TVertex>( renderer, vec![] )?;

    Ok( model )
  }

  unsafe fn create_index_buffer( renderer:&Renderer, indices:&Vec<u32> ) -> Result<( vk::Buffer, vk::DeviceMemory )> {
    let Renderer { ref instance, ref device, ref data, .. } = renderer;

    let size = (size_of::<u32>() * indices.len()) as u64;
    let ( staging_buffer, staging_buffer_memory ) = create_buffer(
      instance,
      device,
      data.physical_device,
      size,
      vk::BufferUsageFlags::TRANSFER_SRC,
      vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
    )?;

    let memory = device.map_memory( staging_buffer_memory, 0, size, vk::MemoryMapFlags::empty() )?;

    memcpy( indices.as_ptr(), memory.cast(), indices.len() );
    device.unmap_memory( staging_buffer_memory );

    let ( index_buffer, index_buffer_memory ) = create_buffer(
      instance,
      device,
      data.physical_device,
      size,
      vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
      vk::MemoryPropertyFlags::DEVICE_LOCAL
    )?;

    copy_buffer( device, data.command_pool, data.graphics_queue, staging_buffer, index_buffer, size )?;

    device.destroy_buffer( staging_buffer, None );
    device.free_memory( staging_buffer_memory, None );

    Ok(( index_buffer, index_buffer_memory ))
  }

  pub unsafe fn update_vertex_buffer<T>( &mut self, renderer:&Renderer, vertices:Vec<TVertex> ) -> Result<()> {
    let Renderer { ref instance, ref device, ref data, .. } = renderer;
    let size = vertices.len();
    let size = (size_of::<T>() * if size == 0 { 1 } else { size } as usize) as u64;

    self.vertices_count = vertices.len() as u32;
    self.vertices = vertices;

    if size > self.vertex_buffer_capacity {
      if self.vertex_buffer_capacity != 0 {
        let _ = device.device_wait_idle();

        device.unmap_memory( self.vertex_staging_buffer_memory );

        device.destroy_buffer( self.vertex_buffer, None );
        device.free_memory( self.vertex_buffer_memory, None );

        device.destroy_buffer( self.vertex_staging_buffer, None );
        device.free_memory( self.vertex_staging_buffer_memory, None );
      }

      let ( staging_buffer, staging_buffer_memory ) = create_buffer(
        instance,
        device,
        data.physical_device,
        size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
      )?;

      let ( vertex_buffer, vertex_buffer_memory ) = create_buffer(
        instance,
        device,
        data.physical_device,
        size,
        vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
      )?;

      self.vertex_buffer = vertex_buffer;
      self.vertex_buffer_memory = vertex_buffer_memory;
      self.vertex_staging_buffer = staging_buffer;
      self.vertex_staging_buffer_memory = staging_buffer_memory;
      self.vertex_staging_mapped_memory = device.map_memory( staging_buffer_memory, 0, size, vk::MemoryMapFlags::empty() )?;

      self.vertex_buffer_capacity = size;
    }

    memcpy( self.vertices.as_ptr(), self.vertex_staging_mapped_memory.cast(), self.vertices.len() );
    copy_buffer( device, data.command_pool, data.graphics_queue, self.vertex_staging_buffer, self.vertex_buffer, size )?;

    Ok(())
  }

  pub unsafe fn update_instance_buffer<T>( &mut self, renderer:&Renderer, instances_data:Vec<T> ) -> Result<()> {
    let Renderer { ref instance, ref device, ref data, .. } = renderer;
    let size = instances_data.len();
    let size = (size_of::<T>() * if size == 0 { 1 } else { size } as usize) as u64;

    self.instances_count = instances_data.len() as u32;

    if size > self.instance_buffer_capacity {
      if self.instance_buffer_capacity != 0 {
        let _ = device.device_wait_idle();

        device.unmap_memory( self.instance_staging_buffer_memory );

        device.destroy_buffer( self.instance_buffer, None );
        device.free_memory( self.instance_buffer_memory, None );

        device.destroy_buffer( self.instance_staging_buffer, None );
        device.free_memory( self.instance_staging_buffer_memory, None );
      }

      let ( staging_buffer, staging_buffer_memory ) = create_buffer(
        instance,
        device,
        data.physical_device,
        size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
      )?;

      let ( instance_buffer, instance_buffer_memory ) = create_buffer(
        instance,
        device,
        data.physical_device,
        size,
        vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
      )?;

      self.instance_buffer = instance_buffer;
      self.instance_buffer_memory = instance_buffer_memory;
      self.instance_staging_buffer = staging_buffer;
      self.instance_staging_buffer_memory = staging_buffer_memory;
      self.instance_staging_mapped_memory = device.map_memory( staging_buffer_memory, 0, size, vk::MemoryMapFlags::empty() )?;

      self.instance_buffer_capacity = size;
    }

    memcpy( instances_data.as_ptr(), self.instance_staging_mapped_memory.cast(), instances_data.len() as usize );
    copy_buffer( device, data.command_pool, data.graphics_queue, self.instance_staging_buffer, self.instance_buffer, size )?;

    Ok(())
  }

  pub unsafe fn destroy( &self, device:&Device ) {
    device.destroy_buffer( self.vertex_buffer, None );
    device.free_memory( self.vertex_buffer_memory, None );

    device.destroy_buffer( self.vertex_staging_buffer, None );
    device.free_memory( self.vertex_staging_buffer_memory, None );

    device.destroy_buffer( self.index_buffer, None );
    device.free_memory( self.index_buffer_memory, None );

    device.destroy_buffer( self.instance_buffer, None );
    device.free_memory( self.instance_buffer_memory, None );

    device.destroy_buffer( self.instance_staging_buffer, None );
    device.free_memory( self.instance_staging_buffer_memory, None );
  }
}

impl<TVertex> Renderable for Model<TVertex> {
  unsafe fn render( &self, device:&Device, command_buffer:vk::CommandBuffer ) {
    device.cmd_bind_vertex_buffers( command_buffer, 0, &[ self.vertex_buffer, self.instance_buffer ], &[ 0, 0 ] );
    device.cmd_bind_index_buffer( command_buffer, self.index_buffer, 0, vk::IndexType::UINT32 );
    device.cmd_draw_indexed( command_buffer, self.indices.len() as u32, self.instances_count, 0, 0, 0 );
  }

  fn get_draw_mode( &self ) -> super::vertex::DrawMode {
      super::vertex::DrawMode::EDGES
  }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ModelInstance {
  pub instance_transform: Mat4,
}


impl Model<Vertex> {
  pub unsafe fn new_cube( renderer:&Renderer ) -> Result<Self> {
    Model::new(
      renderer,
      BOX_VERTICES.to_vec(),
      BOX_INDICES.to_vec(),
    )
  }

  pub unsafe fn from_file( renderer:&Renderer, src:&str ) -> Result<Self> {
    let mut vertices = vec![];
    let mut indices = vec![];

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
          normal: vec3(
            loaded_model.mesh.normals[ pos_offset + 0 ],
            loaded_model.mesh.normals[ pos_offset + 1 ],
            loaded_model.mesh.normals[ pos_offset + 2 ],
          ),
        };

        if let Some( index ) = unique_vertices.get( &vertex ) {
          indices.push( *index as u32 )
        } else {
          let index = vertices.len();
          unique_vertices.insert( vertex, index );
          vertices.push( vertex );
          indices.push( index as u32 )
        }
      }
    }

    Model::<Vertex>::new( renderer, vertices, indices )
  }
}

impl<TVertex> RendererModelDescriptions for Model<TVertex> {
  fn binding_description() -> vk::VertexInputBindingDescription {
    vk::VertexInputBindingDescription::builder()
      .binding( 0 )
      .stride( size_of::<VoxelVertex>() as u32 )
      .input_rate( vk::VertexInputRate::VERTEX )
      .build()
  }

  fn attribute_description() -> Vec<vk::VertexInputAttributeDescription> {
    let pos = vk::VertexInputAttributeDescription::builder()
        .binding( 0 )
        .location( 0 )
        .format( vk::Format::R32G32B32_SFLOAT )
        .offset( 0 )
        .build();

    let color = vk::VertexInputAttributeDescription::builder()
        .binding( 0 )
        .location( 1 )
        .format( vk::Format::R32G32B32_SFLOAT )
        .offset( size_of::<Vec3>() as u32 )
        .build();

    vec![ pos, color ]
  }

  fn instances_binding_description() -> vk::VertexInputBindingDescription {
    vk::VertexInputBindingDescription::builder()
      .binding( 1 )
      .stride( size_of::<ModelInstance>() as u32 )
      .input_rate( vk::VertexInputRate::INSTANCE )
      .build()
  }

  fn instances_attribute_description() -> Vec<vk::VertexInputAttributeDescription> {
    let model_x = vk::VertexInputAttributeDescription::builder()
      .binding( 1 )
      .location( 2 )
      .format( vk::Format::R32G32B32_SFLOAT )
      .offset( 0 )
      .build();

    let model_y = vk::VertexInputAttributeDescription::builder()
      .binding( 1 )
      .location( 3 )
      .format( vk::Format::R32G32B32_SFLOAT )
      .offset( size_of::<Vec3>() as u32 )
      .build();

    let model_z = vk::VertexInputAttributeDescription::builder()
      .binding( 1 )
      .location( 4 )
      .format( vk::Format::R32G32B32_SFLOAT )
      .offset( (size_of::<Vec3>() * 2) as u32 )
      .build();

    let model_w = vk::VertexInputAttributeDescription::builder()
      .binding( 1 )
      .location( 5 )
      .format( vk::Format::R32G32B32_SFLOAT )
      .offset( (size_of::<Vec3>() * 3) as u32 )
      .build();

    vec![ model_x, model_y, model_z, model_w ]
  }
}
