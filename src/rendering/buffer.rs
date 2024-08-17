use vulkanalia::prelude::v1_0::*;
use anyhow::{ anyhow, Result };
use super::renderer::AppData;

unsafe fn get_memory_type_index( instance:&Instance, physical_device:vk::PhysicalDevice, properties:vk::MemoryPropertyFlags, requirements:vk::MemoryRequirements ) -> Result<u32> {
  let memory = instance.get_physical_device_memory_properties( physical_device );

  (0..memory.memory_type_count)
    .find( |i| {
      let suitable = (requirements.memory_type_bits & (1 << i)) != 0;
      let memory_type = memory.memory_types[ *i as usize ];
      suitable && memory_type.property_flags.contains( properties )
    } )
    .ok_or_else( || anyhow!( "Failed to find suitable memory type." ) )
}


pub unsafe fn create_buffer( instance:&Instance, device:&Device, physical_device:vk::PhysicalDevice, size:vk::DeviceSize, usage:vk::BufferUsageFlags, properties:vk::MemoryPropertyFlags ) -> Result<(vk::Buffer, vk::DeviceMemory)> {
  let buffer_info = vk::BufferCreateInfo::builder()
    .size( size )
    .usage( usage )
    .sharing_mode( vk::SharingMode::EXCLUSIVE )
    .flags( vk::BufferCreateFlags::empty() );

  let buffer = device.create_buffer( &buffer_info, None )?;

  let requirements = device.get_buffer_memory_requirements( buffer );
  let memory_info = vk::MemoryAllocateInfo::builder()
    .allocation_size( requirements.size )
    .memory_type_index( get_memory_type_index( instance, physical_device, properties, requirements )? );

  let buffer_memory = device.allocate_memory( &memory_info, None )?;

  device.bind_buffer_memory( buffer, buffer_memory, 0 )?;

  Ok(( buffer, buffer_memory ))
}

pub unsafe fn copy_buffer( device:&Device, command_pool:vk::CommandPool, graphics_queue:vk::Queue, source:vk::Buffer, destination:vk::Buffer, size:vk::DeviceSize ) -> Result<()> {
  let command_buffer = begin_single_time_commands( device, command_pool )?;

  let regions = vk::BufferCopy::builder().size( size );
  device.cmd_copy_buffer( command_buffer, source, destination, &[ regions ] );

  end_single_time_commands( device, command_pool, graphics_queue, command_buffer )?;

  Ok(())
}

pub unsafe fn begin_single_time_commands( device:&Device, command_pool:vk::CommandPool ) -> Result<vk::CommandBuffer> {
  let info = vk::CommandBufferAllocateInfo::builder()
    .level( vk::CommandBufferLevel::PRIMARY )
    .command_pool( command_pool )
    .command_buffer_count( 1 );

  let command_buffer = device.allocate_command_buffers( &info )?[ 0 ];

  let info = vk::CommandBufferBeginInfo::builder()
    .flags( vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT );

  device.begin_command_buffer( command_buffer, &info )?;

  Ok( command_buffer )
}

pub unsafe fn end_single_time_commands( device:&Device, command_pool:vk::CommandPool, graphics_queue:vk::Queue, command_buffer:vk::CommandBuffer ) -> Result<()> {
  device.end_command_buffer( command_buffer )?;

  let command_buffers = &[ command_buffer ];
  let info = vk::SubmitInfo::builder()
    .command_buffers( command_buffers );

  device.queue_submit( graphics_queue, &[ info ], vk::Fence::null() )?;
  device.queue_wait_idle( graphics_queue )?;
  device.free_command_buffers( command_pool, &[ command_buffer ] );

  Ok(())
}

pub unsafe fn create_image_view( device:&Device, image:vk::Image, format:vk::Format, aspects:vk::ImageAspectFlags, mip_levels:u32 ) -> Result<vk::ImageView> {
  let subresource_range = vk::ImageSubresourceRange::builder()
    .aspect_mask( aspects )
    .base_mip_level( 0 )
    .level_count( mip_levels )
    .base_array_layer( 0 )
    .layer_count( 1 );

  let info = vk::ImageViewCreateInfo::builder()
    .image( image )
    .view_type( vk::ImageViewType::_2D )
    .format( format )
    .subresource_range( subresource_range );

  Ok( device.create_image_view( &info, None )? )
}
pub unsafe fn create_image(
  instance: &Instance,
  device: &Device,
  data: &AppData,
  width: u32,
  height: u32,
  mip_levels: u32,
  samples: vk::SampleCountFlags,
  format: vk::Format,
  tiling: vk::ImageTiling,
  usage: vk::ImageUsageFlags,
  properties: vk::MemoryPropertyFlags
) -> Result<( vk::Image, vk::DeviceMemory )> {
  let info = vk::ImageCreateInfo::builder()
    .image_type( vk::ImageType::_2D )
    .extent( vk::Extent3D { width, height, depth:1 } )
    .mip_levels( mip_levels )
    .array_layers( 1 )
    .format( format )
    .tiling( tiling )
    .initial_layout( vk::ImageLayout::UNDEFINED )
    .usage( usage )
    .sharing_mode( vk::SharingMode::EXCLUSIVE )
    .samples( samples );

  let image = device.create_image( &info, None )?;
  let requirements = device.get_image_memory_requirements( image );

  let info = vk::MemoryAllocateInfo::builder()
    .allocation_size( requirements.size )
    .memory_type_index( get_memory_type_index( instance, data.physical_device, properties, requirements )? );

  let image_memory = device.allocate_memory( &info, None )?;

  device.bind_image_memory( image, image_memory, 0 )?;

  Ok((image, image_memory))
}

pub unsafe fn create_color_objects( instance:&Instance, device:&Device, data:&mut AppData ) -> Result<()> {
  let (color_image, color_image_memory) = create_image(
    instance, device, data,
    data.swapchain_extent.width,
    data.swapchain_extent.height,
    1,
    data.msaa_samples,
    data.swapchain_format,
    vk::ImageTiling::OPTIMAL,
    vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSIENT_ATTACHMENT,
    vk::MemoryPropertyFlags::DEVICE_LOCAL,
  )?;

  data.color_image = color_image;
  data.color_image_memory = color_image_memory;
  data.color_image_view = create_image_view(
    device,
    color_image,
    data.swapchain_format,
    vk::ImageAspectFlags::COLOR,
    1,
  )?;

  Ok(())
}

pub unsafe fn transition_image_layout( device:&Device, data:&AppData, image:vk::Image, format:vk::Format, old_layout:vk::ImageLayout, new_layout:vk::ImageLayout, mip_levels:u32 ) -> Result<()> {
  let (
    src_access_mask,
    dst_access_mask,
    src_stage_mask,
    dst_stage_mask,
  ) = match (old_layout, new_layout) {
    (vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL) => (
      vk::AccessFlags::empty(),
      vk::AccessFlags::TRANSFER_WRITE,
      vk::PipelineStageFlags::TOP_OF_PIPE,
      vk::PipelineStageFlags::TRANSFER,
    ),

    (vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL) => (
      vk::AccessFlags::TRANSFER_WRITE,
      vk::AccessFlags::SHADER_READ,
      vk::PipelineStageFlags::TRANSFER,
      vk::PipelineStageFlags::FRAGMENT_SHADER,
    ),

    (vk::ImageLayout::UNDEFINED, vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL) => (
      vk::AccessFlags::empty(),
      vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
      vk::PipelineStageFlags::TOP_OF_PIPE,
      vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
    ),

    _ => return Err( anyhow!( "Unsupported image layout transition!" ) )
  };

  let command_buffer = begin_single_time_commands( device, data.command_pool )?;
  let aspect_mask = if new_layout == vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL {
    match format {
      vk::Format::D32_SFLOAT_S8_UINT | vk::Format::D24_UNORM_S8_UINT => vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL,
      _ => vk::ImageAspectFlags::DEPTH,
    }
  } else {
    vk::ImageAspectFlags::COLOR
  };

  let subresource = vk::ImageSubresourceRange::builder()
    .aspect_mask( aspect_mask )
    .base_mip_level( 0 )
    .level_count( mip_levels )
    .base_array_layer( 0 )
    .layer_count( 1 );

  let barrier = vk::ImageMemoryBarrier::builder()
    .old_layout( old_layout )
    .new_layout( new_layout )
    .src_queue_family_index( vk::QUEUE_FAMILY_IGNORED )
    .dst_queue_family_index( vk::QUEUE_FAMILY_IGNORED )
    .image( image )
    .subresource_range( subresource )
    .src_access_mask( src_access_mask )
    .dst_access_mask( dst_access_mask );

  device.cmd_pipeline_barrier(
    command_buffer,
    src_stage_mask,
    dst_stage_mask,
    vk::DependencyFlags::empty(),
    &[] as &[ vk::MemoryBarrier ],
    &[] as &[ vk::BufferMemoryBarrier ],
    &[ barrier ],
  );

  end_single_time_commands( device, data.command_pool, data.graphics_queue, command_buffer )?;

  Ok(())
}

pub unsafe fn copy_buffer_to_image( device:&Device, data:&AppData, buffer:vk::Buffer, image:vk::Image, width:u32, height:u32 ) -> Result<()> {
  let command_buffer = begin_single_time_commands( device, data.command_pool )?;
  let subresource = vk::ImageSubresourceLayers::builder()
    .aspect_mask( vk::ImageAspectFlags::COLOR )
    .mip_level( 0 )
    .base_array_layer( 0 )
    .layer_count( 1 );

  let region = vk::BufferImageCopy::builder()
    .buffer_offset( 0 )
    .buffer_row_length( 0 )
    .buffer_image_height( 0 )
    .image_subresource( subresource )
    .image_offset( vk::Offset3D { x:0, y:0, z:0 } )
    .image_extent( vk::Extent3D { width, height, depth:1 } );

  device.cmd_copy_buffer_to_image( command_buffer, buffer, image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &[ region ] );

  end_single_time_commands( device, data.command_pool, data.graphics_queue, command_buffer )?;

  Ok(())
}
