use vulkanalia::prelude::v1_0::*;
use anyhow::{ anyhow, Result };
use std::{
  fs::File,
  ptr::copy_nonoverlapping as memcpy
};

use super::renderer::{
  AppData,
  create_buffer, begin_single_time_commands, end_single_time_commands,
  create_image, create_image_view, transition_image_layout, copy_buffer_to_image,
};

#[derive(Clone, Debug, Default)]
pub struct Texture {
  image: vk::Image,
  image_memory: vk::DeviceMemory,
  image_view: vk::ImageView,
  mip_levels: u32,
  sampler: vk::Sampler,
  pub descriptor_set: vk::DescriptorSet,
}

impl Texture {
  pub unsafe fn load( instance:&Instance, device:&Device, data:&AppData, src:&str ) -> Result<Self> {
    let ( image, image_memory, mip_levels ) = Texture::create_texture_image( instance, device, data, src )?;
    let image_view = Texture::create_texture_image_view( device, image, mip_levels )?;
    let sampler = Texture::create_texture_sampler( device, mip_levels )?;
    let descriptor_set = Self::create_texture_descriptor_set( device, data.texture_descriptor_set_layout, data.descriptor_pool, image_view, sampler )?;

    Ok( Self {
      image,
      image_memory,
      mip_levels,
      image_view,
      sampler,
      descriptor_set,
    } )
  }

  unsafe fn create_texture_image( instance:&Instance, device:&Device, data:&AppData, src:&str ) -> Result<( vk::Image, vk::DeviceMemory, u32 )> {
    let image = match File::open( src ) {
      Ok( file ) => file,
      Err( err ) => {
        println!( "Current path: {:?}", std::env::current_dir() );
        println!( "{:?}", err );
        return Err( anyhow!( err ) );
      }
    };

    let decoder = png::Decoder::new( image );
    let mut reader = decoder.read_info()?;

    let mut pixels = vec![ 0; reader.info().raw_bytes() ];
    reader.next_frame( &mut pixels )?;

    let size = reader.info().raw_bytes() as u64;
    let ( width, height ) = reader.info().size();
    let mip_levels = (width.max( height ) as f32).log2().floor() as u32 + 1;

    let (stagging_buffer, stagging_buffer_memory) = create_buffer(
      instance, device, data.physical_device, size,
      vk::BufferUsageFlags::TRANSFER_SRC,
      vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
    )?;

    let memory = device.map_memory( stagging_buffer_memory, 0, size, vk::MemoryMapFlags::empty() )?;

    memcpy( pixels.as_ptr(), memory.cast(), pixels.len() );

    device.unmap_memory( stagging_buffer_memory );

    let ( texture_image, texture_image_memory ) = create_image(
      instance, device, data, width, height,
      mip_levels,
      vk::SampleCountFlags::_1,
      vk::Format::R8G8B8A8_SRGB,
      vk::ImageTiling::OPTIMAL,
      vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::TRANSFER_SRC,
      vk::MemoryPropertyFlags::DEVICE_LOCAL
    )?;

    transition_image_layout(
      device, data, texture_image,
      vk::Format::R8G8B8A8_SRGB,
      vk::ImageLayout::UNDEFINED,
      vk::ImageLayout::TRANSFER_DST_OPTIMAL,
      mip_levels
    )?;

    copy_buffer_to_image( device, data, stagging_buffer, texture_image, width, height )?;

    device.destroy_buffer( stagging_buffer, None );
    device.free_memory( stagging_buffer_memory, None );

    Texture::generate_mipmaps( instance, device, data, texture_image, vk::Format::R8G8B8A8_SRGB, width, height, mip_levels )?;

    Ok(( texture_image, texture_image_memory, mip_levels ))
  }

  unsafe fn create_texture_image_view( device:&Device, texture_image:vk::Image, mip_levels:u32 ) -> Result<vk::ImageView> {
    let image_view = create_image_view( device, texture_image, vk::Format::R8G8B8A8_SRGB, vk::ImageAspectFlags::COLOR, mip_levels )?;
    Ok( image_view )
  }

  unsafe fn create_texture_sampler( device:&Device, mip_levels:u32 ) -> Result<vk::Sampler> {
    let info = vk::SamplerCreateInfo::builder()
      .mag_filter( vk::Filter::LINEAR )
      .min_filter( vk::Filter::LINEAR )
      .address_mode_u( vk::SamplerAddressMode::REPEAT )
      .address_mode_v( vk::SamplerAddressMode::REPEAT )
      .address_mode_w( vk::SamplerAddressMode::REPEAT )
      .anisotropy_enable( true )
      .max_anisotropy( 16.0 )
      .border_color( vk::BorderColor::INT_OPAQUE_BLACK )
      .unnormalized_coordinates( false )
      .compare_enable( false )
      .compare_op( vk::CompareOp::ALWAYS )
      .mipmap_mode( vk::SamplerMipmapMode::LINEAR )
      .min_lod( 0.0 )
      .max_lod( mip_levels as f32 )
      .mip_lod_bias( 0.0 );

    let sampler = device.create_sampler( &info, None )?;

    Ok( sampler )
  }

  unsafe fn create_texture_descriptor_set( device:&Device, descriptor_set_layout:vk::DescriptorSetLayout, descriptor_pool:vk::DescriptorPool, image_view:vk::ImageView, sampler:vk::Sampler ) -> Result<vk::DescriptorSet> {
    let layouts = &[ descriptor_set_layout ];
    let info = vk::DescriptorSetAllocateInfo::builder()
      .descriptor_pool( descriptor_pool )
      .set_layouts( layouts );

    let descriptor_set = device.allocate_descriptor_sets( &info )?[ 0 ];

    let info = vk::DescriptorImageInfo::builder()
      .image_layout( vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL )
      .image_view( image_view )
      .sampler( sampler );

    let image_info = &[ info ];
    let sampler_write = vk::WriteDescriptorSet::builder()
      .dst_set( descriptor_set )
      .dst_binding( 0 )
      .dst_array_element( 0 )
      .descriptor_type( vk::DescriptorType::COMBINED_IMAGE_SAMPLER )
      .image_info( image_info );

    device.update_descriptor_sets( &[ sampler_write ], &[] as &[ vk::CopyDescriptorSet ] );

    Ok( descriptor_set )
  }

  pub unsafe fn recreate_descriptor_set( &mut self, device:&Device, descriptor_set_layout:vk::DescriptorSetLayout, descriptor_pool:vk::DescriptorPool, ) -> Result<()> {
    let descriptor_set = Texture::create_texture_descriptor_set( device, descriptor_set_layout, descriptor_pool, self.image_view, self.sampler )?;
    self.descriptor_set = descriptor_set;
    Ok(())
  }

  pub unsafe fn destroy( &self, device:&Device ) {
    device.destroy_sampler( self.sampler, None );
    device.destroy_image_view( self.image_view, None);

    device.destroy_image( self.image, None );
    device.free_memory( self.image_memory, None );
  }

  //

  unsafe fn generate_mipmaps( instance:&Instance, device:&Device, data:&AppData, image:vk::Image, format:vk::Format, width:u32, height:u32, mip_levels:u32 ) -> Result<()> {
    if !instance.get_physical_device_format_properties( data.physical_device, format )
      .optimal_tiling_features
      .contains( vk::FormatFeatureFlags::SAMPLED_IMAGE_FILTER_LINEAR )
    {
      return Err( anyhow!( "Texture image format does not support linear blitting!" ) )
    }

    let command_buffer = begin_single_time_commands( device, data.command_pool )?;

    let subresource = vk::ImageSubresourceRange::builder()
      .aspect_mask( vk::ImageAspectFlags::COLOR )
      .base_array_layer( 0 )
      .layer_count( 1 )
      .level_count( 1 );

    let mut barrier = vk::ImageMemoryBarrier::builder()
      .image( image )
      .src_queue_family_index( vk::QUEUE_FAMILY_IGNORED )
      .dst_queue_family_index( vk::QUEUE_FAMILY_IGNORED )
      .subresource_range( subresource );

    let mut mip_width = width;
    let mut mip_height = height;

    for i in 1..mip_levels {
      barrier.subresource_range.base_mip_level = i - 1;
      barrier.old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
      barrier.new_layout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
      barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
      barrier.dst_access_mask = vk::AccessFlags::TRANSFER_READ;

      device.cmd_pipeline_barrier(
        command_buffer,
        vk::PipelineStageFlags::TRANSFER,
        vk::PipelineStageFlags::TRANSFER,
        vk::DependencyFlags::empty(),
        &[] as &[vk::MemoryBarrier],
        &[] as &[vk::BufferMemoryBarrier],
        &[ barrier ],
      );

      let src_subresource = vk::ImageSubresourceLayers::builder()
        .aspect_mask( vk::ImageAspectFlags::COLOR )
        .mip_level( i - 1 )
        .base_array_layer( 0 )
        .layer_count( 1 );

      let dst_subresource = vk::ImageSubresourceLayers::builder()
        .aspect_mask( vk::ImageAspectFlags::COLOR )
        .mip_level( i )
        .base_array_layer( 0 )
        .layer_count( 1 );

      let blit = vk::ImageBlit::builder()
        .src_offsets([
          vk::Offset3D { x:0, y:0, z:0 },
          vk::Offset3D {
            x: mip_width as i32,
            y: mip_height as i32,
            z: 1
          },
        ])
        .src_subresource( src_subresource )
        .dst_offsets([
          vk::Offset3D { x:0, y:0, z:0 },
          vk::Offset3D {
            x: (if mip_width  > 1 { mip_width  / 2 } else { 1 }) as i32,
            y: (if mip_height > 1 { mip_height / 2 } else { 1 }) as i32,
            z: 1
          },
        ])
        .dst_subresource( dst_subresource );

      device.cmd_blit_image(
        command_buffer,
        image,
        vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
        image,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        &[ blit ],
        vk::Filter::LINEAR,
      );

      barrier.old_layout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
      barrier.new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
      barrier.src_access_mask = vk::AccessFlags::TRANSFER_READ;
      barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;

      device.cmd_pipeline_barrier(
        command_buffer,
        vk::PipelineStageFlags::TRANSFER,
        vk::PipelineStageFlags::FRAGMENT_SHADER,
        vk::DependencyFlags::empty(),
        &[] as &[vk::MemoryBarrier],
        &[] as &[vk::BufferMemoryBarrier],
        &[ barrier ],
      );

      if mip_width > 1 {
        mip_width /= 2;
      }

      if mip_height > 1 {
        mip_height /= 2;
      }
    }

    barrier.subresource_range.base_mip_level = mip_levels - 1;
    barrier.old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
    barrier.new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
    barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
    barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;

    device.cmd_pipeline_barrier(
      command_buffer,
      vk::PipelineStageFlags::TRANSFER,
      vk::PipelineStageFlags::FRAGMENT_SHADER,
      vk::DependencyFlags::empty(),
      &[] as &[vk::MemoryBarrier],
      &[] as &[vk::BufferMemoryBarrier],
      &[ barrier ],
    );

    end_single_time_commands( device, data.command_pool, data.graphics_queue, command_buffer )?;

    Ok(())
  }
}