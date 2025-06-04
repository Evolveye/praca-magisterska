use anyhow::Result;
use cgmath::{ vec2, vec3, Vector3 };
use std::{
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

#[derive(Clone, Debug, Default)]
#[allow(dead_code)]
pub struct ModelStrip<TVertex> {
  pub vertices: Vec<TVertex>,
  pub vertex_buffer: vk::Buffer,
  pub vertex_buffer_memory: vk::DeviceMemory,
  pub instances_count: u32,
  pub instance_buffer: vk::Buffer,
  pub instance_buffer_memory: vk::DeviceMemory,
}

impl<TVertex> ModelStrip<TVertex> {
  pub unsafe fn new( renderer:&Renderer, vertices:Vec<TVertex> ) -> Result<Self> {
    let ( vertex_buffer, vertex_buffer_memory ) = Self::create_vertex_buffer( renderer, &vertices )?;

    Ok( Self {
      vertices,
      vertex_buffer,
      vertex_buffer_memory,
      instances_count: 0,
      instance_buffer: Default::default(),
      instance_buffer_memory: Default::default(),
    } )
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

  unsafe fn create_vertex_buffer( renderer:&Renderer, vertices:&Vec<TVertex> ) -> Result<( vk::Buffer, vk::DeviceMemory )> {
    let Renderer { ref instance, ref device, ref data, .. } = renderer;

    let size = (size_of::<TVertex>() * vertices.len()) as u64;
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

  pub unsafe fn update_instances_buffer_with_defaults( &mut self, renderer:&Renderer, instances_count:u32 ) -> Result<()> {
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

    self.update_instances_buffer( renderer, instances_data )
  }

  pub unsafe fn update_instances_buffer<T>( &mut self, renderer:&Renderer, instances_data:Vec<T> ) -> Result<()> {
    let instances_count = instances_data.len() as u32;
    let ( instance_buffer, instance_buffer_memory ) = Self::create_instance_buffer::<T>( renderer, instances_data )?;

    self.instance_buffer = instance_buffer;
    self.instance_buffer_memory = instance_buffer_memory;
    self.instances_count = instances_count;

    Ok(())
  }

  pub unsafe fn destroy( &self, device:&Device ) {
    device.destroy_buffer( self.vertex_buffer, None );
    device.free_memory( self.vertex_buffer_memory, None );

    device.destroy_buffer( self.instance_buffer, None );
    device.free_memory( self.instance_buffer_memory, None );
  }
}

impl<TVertex> Renderable for ModelStrip<TVertex> {
  unsafe fn render( &self, device:&Device, command_buffer:vk::CommandBuffer ) {
    device.cmd_bind_vertex_buffers( command_buffer, 0, &[ self.vertex_buffer, self.instance_buffer ], &[ 0, 0 ] );
    device.cmd_draw( command_buffer, self.vertices.len() as u32, self.instances_count, 0, 0 );
  }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ModelInstance {
  pub translate: Vec3,
}

impl RendererModelDescriptions for ModelStrip<Vertex> {
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
