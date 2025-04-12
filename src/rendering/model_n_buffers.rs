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

use super::vertex::Vertex;
use super::buffer::{ create_buffer, copy_buffer };

type Mat4 = cgmath::Matrix4<f32>;
type Vec3 = cgmath::Vector3<f32>;



#[derive(Clone, Debug, Default)]
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
  pub unsafe fn new(
    instance: &Instance,
    device: &Device,
    physical_device: vk::PhysicalDevice,
    command_pool: vk::CommandPool,
    graphics_queue: vk::Queue,
    vertices: Vec<Vertex>,
    indices: Vec<u32>
  ) -> Result<Self> {
    let instances_count = 1;

    let ( index_buffer, index_buffer_memory ) = Model::create_index_buffer( instance, device, physical_device, command_pool, graphics_queue, &indices )?;
    let ( vertex_buffer, vertex_buffer_memory ) = Model::create_vertex_buffer( instance, device, physical_device, command_pool, graphics_queue, &vertices )?;
    let ( instance_buffer, instance_buffer_memory ) = Model::create_instance_buffer( instance, device, physical_device, command_pool, graphics_queue, instances_count )?;

    Ok( Self {
      vertices,
      indices,
      vertex_buffer,
      vertex_buffer_memory,
      index_buffer,
      index_buffer_memory,
      instances_count,
      instance_buffer,
      instance_buffer_memory,
    } )
  }

  pub unsafe fn from_file(
    instance: &Instance,
    device: &Device,
    physical_device: vk::PhysicalDevice,
    command_pool: vk::CommandPool,
    graphics_queue: vk::Queue,
    src: &str,
  ) -> Result<Self> {
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

    Model::new( instance, device, physical_device, command_pool, graphics_queue, vertices, indices )
  }

  unsafe fn create_index_buffer(
    instance: &Instance,
    device: &Device,
    physical_device: vk::PhysicalDevice,
    command_pool: vk::CommandPool,
    graphics_queue: vk::Queue,
    indices: &Vec<u32>
  ) -> Result<( vk::Buffer, vk::DeviceMemory )> {
    let size = (size_of::<u32>() * indices.len()) as u64;
    let ( staging_buffer, staging_buffer_memory ) = create_buffer(
      instance,
      device,
      physical_device,
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
      physical_device,
      size,
      vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
      vk::MemoryPropertyFlags::DEVICE_LOCAL
    )?;

    copy_buffer( device, command_pool, graphics_queue, staging_buffer, index_buffer, size )?;

    device.destroy_buffer( staging_buffer, None );
    device.free_memory( staging_buffer_memory, None );

    Ok(( index_buffer, index_buffer_memory ))
  }

  unsafe fn create_vertex_buffer(
    instance: &Instance,
    device: &Device,
    physical_device: vk::PhysicalDevice,
    command_pool: vk::CommandPool,
    graphics_queue: vk::Queue,
    vertices: &Vec<Vertex>
  ) -> Result<( vk::Buffer, vk::DeviceMemory )> {
    let size = (size_of::<Vertex>() * vertices.len()) as u64;
    let ( staging_buffer, staging_buffer_memory ) = create_buffer(
      instance,
      device,
      physical_device,
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
      physical_device,
      size,
      vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
      vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    copy_buffer( device, command_pool, graphics_queue, staging_buffer, vertex_buffer, size )?;

    device.destroy_buffer( staging_buffer, None );
    device.free_memory( staging_buffer_memory, None );

    Ok(( vertex_buffer, vertex_buffer_memory ))
  }

  unsafe fn create_instance_buffer(
    instance: &Instance,
    device: &Device,
    physical_device: vk::PhysicalDevice,
    command_pool: vk::CommandPool,
    graphics_queue: vk::Queue,
    instances_count: u32,
  ) -> Result<( vk::Buffer, vk::DeviceMemory )> {
    let size = (size_of::<ModelInstance>() * instances_count as usize) as u64;
    let ( staging_buffer, staging_buffer_memory ) = create_buffer(
      instance,
      device,
      physical_device,
      size,
      vk::BufferUsageFlags::TRANSFER_SRC,
      vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
    )?;

    let memory = device.map_memory( staging_buffer_memory, 0, size, vk::MemoryMapFlags::empty() )?;

    // TODO Instances data
    let radius = 10.0;
    let instances_data = (0..instances_count).map( |model_index| {
      let theta = 2.0 * std::f32::consts::PI * (model_index as f32) / (instances_count as f32);
      let x = radius * theta.cos();
      let z = radius * theta.sin();

      let translate = Vector3::new( x, z / 3.0, z );

      ModelInstance { translate }
    } )
    .collect::<Vec<ModelInstance>>();

    memcpy( instances_data.as_ptr(), memory.cast(), instances_count as usize );
    device.unmap_memory( staging_buffer_memory );

    let ( instance_buffer, instance_buffer_memory ) = create_buffer(
      instance,
      device,
      physical_device,
      size,
      vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
      vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    copy_buffer( device, command_pool, graphics_queue, staging_buffer, instance_buffer, size )?;

    device.destroy_buffer( staging_buffer, None );
    device.free_memory( staging_buffer_memory, None );

    Ok(( instance_buffer, instance_buffer_memory ))
  }

  pub unsafe fn destroy( &self, device:&Device ) {
    device.destroy_buffer( self.vertex_buffer, None );
    device.free_memory( self.vertex_buffer_memory, None );

    device.destroy_buffer( self.index_buffer, None );
    device.free_memory( self.index_buffer_memory, None );

    device.destroy_buffer( self.instance_buffer, None );
    device.free_memory( self.instance_buffer_memory, None );
  }

  pub unsafe fn render( &self, device:&Device, command_buffer:vk::CommandBuffer ) {
    device.cmd_bind_vertex_buffers( command_buffer, 0, &[ self.vertex_buffer ], &[ 0, 0 ] );
    device.cmd_bind_index_buffer( command_buffer, self.index_buffer, 0, vk::IndexType::UINT32 );
    device.cmd_draw_indexed( command_buffer, self.indices.len() as u32, 1, 0, 0, 0 );

    // pub unsafe fn render( &self, device:&Device, command_buffer:vk::CommandBuffer ) {
    // device.cmd_bind_vertex_buffers( command_buffer, 0, &[ self.vertex_buffer, self.instance_buffer ], &[ 0, 0 ] );
    // device.cmd_draw_indexed( command_buffer, self.indices.len() as u32, self.instances_count, 0, 0, 0 );
  }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ModelInstance {
  pub translate: Vec3
}
