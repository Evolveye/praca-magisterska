use anyhow::Result;
use cgmath::{ vec2, vec3, Vector3 };
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

use super::renderer::Renderer;
use super::vertex::{Renderable, RendererModelDescriptions, Vertex};
use super::buffer::{ create_buffer, copy_buffer };

type Vec3 = cgmath::Vector3<f32>;

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
pub struct Model {
  pub vertices: Vec<Vertex>,
  pub indices: Vec<u32>,
  pub vertex_buffer: vk::Buffer,
  pub vertex_buffer_memory: vk::DeviceMemory,
  pub index_buffer: vk::Buffer,
  pub index_buffer_memory: vk::DeviceMemory,
  pub instances_count: u32,
  pub instance_buffer: vk::Buffer,
  pub instance_buffer_memory: vk::DeviceMemory,
}

impl Model {
  pub unsafe fn new<TInstance>( renderer:&Renderer, vertices:Vec<Vertex>, indices:Vec<u32> ) -> Result<Self> {
    let ( index_buffer, index_buffer_memory ) = Model::create_index_buffer( renderer, &indices )?;
    let ( vertex_buffer, vertex_buffer_memory ) = Model::create_vertex_buffer( renderer, &vertices )?;
    // let ( instance_buffer, instance_buffer_memory ) = Model::create_instance_buffer::<TInstance>( renderer, vec![] )?;

    Ok( Self {
      vertices,
      indices,
      vertex_buffer,
      vertex_buffer_memory,
      index_buffer,
      index_buffer_memory,
      instances_count: 0,
      instance_buffer: Default::default(),
      instance_buffer_memory: Default::default(),
    } )
  }

  pub unsafe fn new_cube( renderer:&Renderer ) -> Result<Self> {
    Model::new::<ModelInstance>(
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

    Model::new::<ModelInstance>( renderer, vertices, indices )
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

  unsafe fn create_vertex_buffer( renderer:&Renderer, vertices:&Vec<Vertex> ) -> Result<( vk::Buffer, vk::DeviceMemory )> {
    let Renderer { ref instance, ref device, ref data, .. } = renderer;

    let size = (size_of::<Vertex>() * vertices.len()) as u64;
    let ( staging_buffer, staging_buffer_memory ) = create_buffer(
      instance,
      device,
      data.physical_device,
      size,
      vk::BufferUsageFlags::TRANSFER_SRC,
      vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
    )?;

    let memory = device.map_memory( staging_buffer_memory, 0, size, vk::MemoryMapFlags::empty() )?;

    memcpy( vertices.as_ptr(), memory.cast(), vertices.len() );
    device.unmap_memory( staging_buffer_memory );

    let ( vertex_buffer, vertex_buffer_memory ) = create_buffer(
      instance,
      device,
      data.physical_device,
      size,
      vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
      vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    copy_buffer( device, data.command_pool, data.graphics_queue, staging_buffer, vertex_buffer, size )?;

    device.destroy_buffer( staging_buffer, None );
    device.free_memory( staging_buffer_memory, None );

    Ok(( vertex_buffer, vertex_buffer_memory ))
  }

  pub unsafe fn create_instance_buffer<T>( renderer:&Renderer, instances_data:Vec<T> ) -> Result<( vk::Buffer, vk::DeviceMemory )> {
    let Renderer { ref instance, ref device, ref data, .. } = renderer;

    let size = (size_of::<T>() * instances_data.len() as usize) as u64;
    let ( staging_buffer, staging_buffer_memory ) = create_buffer(
      instance,
      device,
      data.physical_device,
      size,
      vk::BufferUsageFlags::TRANSFER_SRC,
      vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
    )?;

    let memory = device.map_memory( staging_buffer_memory, 0, size, vk::MemoryMapFlags::empty() )?;

    memcpy( instances_data.as_ptr(), memory.cast(), instances_data.len() as usize );
    device.unmap_memory( staging_buffer_memory );

    let ( instance_buffer, instance_buffer_memory ) = create_buffer(
      instance,
      device,
      data.physical_device,
      size,
      vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
      vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    copy_buffer( device, data.command_pool, data.graphics_queue, staging_buffer, instance_buffer, size )?;

    device.destroy_buffer( staging_buffer, None );
    device.free_memory( staging_buffer_memory, None );

    Ok(( instance_buffer, instance_buffer_memory ))
  }

  pub unsafe fn update_unstances_buffer_with_defaults( &mut self, renderer:&Renderer, instances_count:u32 ) -> Result<()> {
    let instances_data = (0..instances_count).map( |model_index| {
        // // RING
        // let radius = 10.0;
        // let theta = 2.0 * std::f32::consts::PI * (model_index as f32) / (instances_count as f32);
        // let x = radius * theta.cos();
        // let z = radius * theta.sin();
        // let translate = Vector3::new( x, 0.0, z );
        // ModelInstance { translate }

        let side_size = 50;
        let x = (model_index % side_size) as f32;
        let y = (model_index / (side_size * side_size)) as f32;
        let z = ((model_index / side_size) % side_size) as f32;
        let translate = Vector3::new( x, y, z );
        ModelInstance { translate }
    } ).collect::<Vec<ModelInstance>>();

    let ( instance_buffer, instance_buffer_memory ) = Model::create_instance_buffer::<ModelInstance>( renderer, instances_data )?;

    self.instance_buffer = instance_buffer;
    self.instance_buffer_memory = instance_buffer_memory;
    self.instances_count = instances_count;

    Ok(())
  }

  pub unsafe fn destroy( &self, device:&Device ) {
    device.destroy_buffer( self.vertex_buffer, None );
    device.free_memory( self.vertex_buffer_memory, None );

    device.destroy_buffer( self.index_buffer, None );
    device.free_memory( self.index_buffer_memory, None );

    device.destroy_buffer( self.instance_buffer, None );
    device.free_memory( self.instance_buffer_memory, None );
  }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ModelInstance {
  pub translate: Vec3
}

impl Renderable for Model {
  // pub unsafe fn render( &self, device:&Device, command_buffer:vk::CommandBuffer ) {
  //   device.cmd_bind_vertex_buffers( command_buffer, 0, &[ self.vertex_buffer ], &[ 0, 0 ] );
  //   device.cmd_bind_index_buffer( command_buffer, self.index_buffer, 0, vk::IndexType::UINT32 );
  //   device.cmd_draw_indexed( command_buffer, self.indices.len() as u32, 1, 0, 0, 0 );

  unsafe fn render( &self, device:&Device, command_buffer:vk::CommandBuffer ) {
    device.cmd_bind_vertex_buffers( command_buffer, 0, &[ self.vertex_buffer, self.instance_buffer ], &[ 0, 0 ] );
    device.cmd_bind_index_buffer( command_buffer, self.index_buffer, 0, vk::IndexType::UINT32 );
    device.cmd_draw_indexed( command_buffer, self.indices.len() as u32, self.instances_count, 0, 0, 0 );
  }
}

impl RendererModelDescriptions for Model {
  fn binding_description() -> vk::VertexInputBindingDescription {
    vk::VertexInputBindingDescription::builder()
      .binding( 0 )
      .stride( size_of::<Vertex>() as u32 )
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

    let normal = vk::VertexInputAttributeDescription::builder()
      .binding( 0 )
      .location( 2 )
      .format( vk::Format::R32G32B32_SFLOAT )
      .offset( (size_of::<Vec3>() + size_of::<Vec3>()) as u32 )
      .build();

    let tex_coord = vk::VertexInputAttributeDescription::builder()
      .binding( 0 )
      .location( 3 )
      .format( vk::Format::R32G32_SFLOAT )
      .offset( (size_of::<Vec3>() + size_of::<Vec3>() + size_of::<Vec3>()) as u32 )
      .build();

    vec![ pos, color, normal, tex_coord ]
  }

  fn instances_binding_description() -> vk::VertexInputBindingDescription {
    vk::VertexInputBindingDescription::builder()
      .binding( 1 )
      .stride( size_of::<ModelInstance>() as u32 )
      .input_rate( vk::VertexInputRate::INSTANCE )
      .build()
  }

  fn instances_attribute_description() -> Vec<vk::VertexInputAttributeDescription> {
    let instance_matrix = vk::VertexInputAttributeDescription::builder()
      .binding( 1 )
      .location( 4 )
      .format( vk::Format::R32G32B32_SFLOAT )
      // .format( vk::Format::R32G32B32A32_SFLOAT )
      .offset( 0 )
      .build();

    vec![ instance_matrix ]
  }
}