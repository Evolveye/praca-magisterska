use vulkanalia::bytecode::Bytecode;
use vulkanalia::prelude::v1_0::*;
use anyhow::Result;
use super::vertex::RendererModelDescriptions;
use super::renderer::{AppData, AppMode};

unsafe fn create_shader_module( device:&Device, bytecode:&[u8] ) -> Result<vk::ShaderModule> {
  let bytecode = Bytecode::new( bytecode ).unwrap();
  let info = vk::ShaderModuleCreateInfo::builder()
    .code_size( bytecode.code_size() )
    .code( bytecode.code() );

  Ok( device.create_shader_module( &info, None )? )
}

unsafe fn create_shader_stage<'a>( device:&Device, shader_bytes:&[u8], stage:vk::ShaderStageFlags ) -> Result<(vk::ShaderModule, vk::PipelineShaderStageCreateInfoBuilder<'a>)> {
  let shader_module = create_shader_module( device, shader_bytes )?;

  let vert_stage = vk::PipelineShaderStageCreateInfo::builder()
    .stage( stage )
    .module( shader_module )
    .name( b"main\0" );

  Ok( (shader_module, vert_stage) )
}

pub unsafe fn create_pipeline_for_model<T:RendererModelDescriptions>( device:&Device, data:&mut AppData ) -> Result<()> {
  let ( vert_shader_module, vert_stage ) = create_shader_stage( device, include_bytes!( "./shaders/model-untextured-lighted/vert.spv" ), vk::ShaderStageFlags::VERTEX )?;
  let ( frag_shader_module, frag_stage ) = create_shader_stage( device, include_bytes!( "./shaders/model-untextured-lighted/frag.spv" ), vk::ShaderStageFlags::FRAGMENT )?;

  let binding_descriptions = &[ T::binding_description() ];
  let attribute_descriptions = T::attribute_description();

  let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
    .vertex_binding_descriptions( binding_descriptions )
    .vertex_attribute_descriptions( &attribute_descriptions );

  let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
    .topology( vk::PrimitiveTopology::TRIANGLE_LIST )
    .primitive_restart_enable( false );

  let viewport = vk::Viewport::builder()
    .x( 0.0 )
    .y( 0.0 )
    .width( data.swapchain_extent.width as f32 )
    .height( data.swapchain_extent.height as f32 )
    .min_depth( 0.0 )
    .max_depth( 1.0 );

  let scissor = vk::Rect2D::builder()
    .offset( vk::Offset2D { x:0, y:0 } )
    .extent( data.swapchain_extent );

  let viewports = &[ viewport ];
  let scissors = &[ scissor ];
  let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
    .viewports( viewports )
    .scissors( scissors );

  let rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
    .depth_clamp_enable( false )
    .rasterizer_discard_enable( false )
    .polygon_mode( vk::PolygonMode::FILL )
    .line_width( 1.0 )
    .cull_mode( vk::CullModeFlags::BACK )
    .front_face( vk::FrontFace::COUNTER_CLOCKWISE )
    .depth_bias_enable( false );

  let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
    .rasterization_samples( data.msaa_samples )
    .sample_shading_enable( true )
    .min_sample_shading( 0.2 );

  let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::builder()
    .depth_test_enable( true )
    .depth_write_enable( true )
    .depth_compare_op( vk::CompareOp::LESS )
    .depth_bounds_test_enable( false )
    .min_depth_bounds( 0.0 )
    .max_depth_bounds( 1.0 )
    .stencil_test_enable( false );
    // .front( vk::StencilOpState )
    // .back( vk::StencilOpState );

  let attachment = vk::PipelineColorBlendAttachmentState::builder()
    .color_write_mask( vk::ColorComponentFlags::all() )
    .blend_enable( true )
    .src_color_blend_factor( vk::BlendFactor::SRC_ALPHA )
    .dst_color_blend_factor( vk::BlendFactor::ONE_MINUS_SRC_ALPHA )
    .color_blend_op( vk::BlendOp::ADD )
    .src_alpha_blend_factor( vk::BlendFactor::ONE )
    .dst_alpha_blend_factor( vk::BlendFactor::ZERO )
    .alpha_blend_op( vk::BlendOp::ADD );

  let attachments = &[ attachment ];
  let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
    .logic_op_enable( false )
    .logic_op( vk::LogicOp::COPY )
    .attachments( attachments )
    .blend_constants([ 0.0, 0.0, 0.0, 0.0 ]);

  let vert_push_constant_range = vk::PushConstantRange::builder()
    .stage_flags( vk::ShaderStageFlags::VERTEX )
    .offset( 0 )
    .size( 64 );

  let frag_push_constant_range = vk::PushConstantRange::builder()
    .stage_flags( vk::ShaderStageFlags::FRAGMENT )
    .offset( 64 )
    .size( 4 );

  let set_layouts = match data.mode {
    AppMode::Voxels => vec![ data.uniform_descriptor_set_layout ],
    _ => vec![ data.uniform_descriptor_set_layout, data.texture_descriptor_set_layout ],
  };

  let push_constant_ranges = &[ vert_push_constant_range, frag_push_constant_range ];
  let layout_info = vk::PipelineLayoutCreateInfo::builder()
    .set_layouts( &set_layouts )
    .push_constant_ranges( push_constant_ranges );

  data.pipeline_layout = device.create_pipeline_layout( &layout_info, None )?;

  let stages = &[ vert_stage, frag_stage ];
  let info = vk::GraphicsPipelineCreateInfo::builder()
    .stages( stages )
    .vertex_input_state( &vertex_input_state )
    .input_assembly_state( &input_assembly_state )
    .viewport_state( &viewport_state )
    .rasterization_state( &rasterization_state )
    .multisample_state( &multisample_state )
    .depth_stencil_state( &depth_stencil_state )
    .color_blend_state( &color_blend_state )
    .layout( data.pipeline_layout )
    .render_pass( data.render_pass )
    .subpass( 0 )
    .base_pipeline_handle( vk::Pipeline::null() )
    .base_pipeline_index( -1 );

  data.pipeline = device.create_graphics_pipelines( vk::PipelineCache::null(), &[ info ], None )?.0[ 0 ];

  device.destroy_shader_module( vert_shader_module, None );
  device.destroy_shader_module( frag_shader_module, None );

  Ok(())
}

pub unsafe fn create_pipeline_for_instances<T:RendererModelDescriptions>( device:&Device, data:&mut AppData ) -> Result<()> {
  let ( (vert_shader_module, vert_stage), (frag_shader_module, frag_stage) ) = match data.mode {
    AppMode::Voxels => (
      create_shader_stage( device, include_bytes!( "./shaders/voxels/vert.spv" ), vk::ShaderStageFlags::VERTEX )?,
      create_shader_stage( device, include_bytes!( "./shaders/voxels/frag.spv" ), vk::ShaderStageFlags::FRAGMENT )?
    ),

    AppMode::InstancesTexturedLighted => (
      create_shader_stage( device, include_bytes!( "./shaders/instances-textured-lighted/vert.spv" ), vk::ShaderStageFlags::VERTEX )?,
      create_shader_stage( device, include_bytes!( "./shaders/instances-textured-lighted/frag.spv" ), vk::ShaderStageFlags::FRAGMENT )?
    ),

    AppMode::InstancesUntexturedUnlighted => (
      create_shader_stage( device, include_bytes!( "./shaders/instances-untextured-unlighted/vert.spv" ), vk::ShaderStageFlags::VERTEX )?,
      create_shader_stage( device, include_bytes!( "./shaders/instances-untextured-unlighted/frag.spv" ), vk::ShaderStageFlags::FRAGMENT )?
    ),

    _ => todo!(),
  };

  let (binding_descriptions, attribute_descriptions) = {
    let binding_descriptions = [ T::binding_description(), T::instances_binding_description() ];
    let attribute_descriptions = {
      let vertex_description = T::attribute_description();
      let instance_description = T::instances_attribute_description().to_vec();
      [ vertex_description, instance_description ].concat()
    };

    (binding_descriptions, attribute_descriptions)
  };

  let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
    .vertex_binding_descriptions( &binding_descriptions )
    .vertex_attribute_descriptions( &attribute_descriptions );

  // let instance_binding_descriptions = &[ ModelInstance::binding_description() ];
  // let instance_attribute_descriptions = ModelInstance::attribute_description();
  // let instance_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
  //   .vertex_binding_descriptions( instance_binding_descriptions )
  //   .vertex_attribute_descriptions( &instance_attribute_descriptions );

  let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
    .topology( vk::PrimitiveTopology::TRIANGLE_LIST )
    .primitive_restart_enable( false );

  let viewport = vk::Viewport::builder()
    .x( 0.0 )
    .y( 0.0 )
    .width( data.swapchain_extent.width as f32 )
    .height( data.swapchain_extent.height as f32 )
    .min_depth( 0.0 )
    .max_depth( 1.0 );

  let scissor = vk::Rect2D::builder()
    .offset( vk::Offset2D { x:0, y:0 } )
    .extent( data.swapchain_extent );

  let viewports = &[ viewport ];
  let scissors = &[ scissor ];
  let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
    .viewports( viewports )
    .scissors( scissors );

  let rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
    .depth_clamp_enable( false )
    .rasterizer_discard_enable( false )
    .polygon_mode( vk::PolygonMode::FILL )
    .line_width( 1.0 )
    .cull_mode( vk::CullModeFlags::BACK )
    .front_face( vk::FrontFace::COUNTER_CLOCKWISE )
    .depth_bias_enable( false );

  let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
    .rasterization_samples( data.msaa_samples )
    .sample_shading_enable( true )
    .min_sample_shading( 0.2 );

  let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::builder()
    .depth_test_enable( true )
    .depth_write_enable( true )
    .depth_compare_op( vk::CompareOp::LESS )
    .depth_bounds_test_enable( false )
    .min_depth_bounds( 0.0 )
    .max_depth_bounds( 1.0 )
    .stencil_test_enable( false );
    // .front( vk::StencilOpState )
    // .back( vk::StencilOpState );

  let attachment = vk::PipelineColorBlendAttachmentState::builder()
    .color_write_mask( vk::ColorComponentFlags::all() )
    .blend_enable( true )
    .src_color_blend_factor( vk::BlendFactor::SRC_ALPHA )
    .dst_color_blend_factor( vk::BlendFactor::ONE_MINUS_SRC_ALPHA )
    .color_blend_op( vk::BlendOp::ADD )
    .src_alpha_blend_factor( vk::BlendFactor::ONE )
    .dst_alpha_blend_factor( vk::BlendFactor::ZERO )
    .alpha_blend_op( vk::BlendOp::ADD );

  let attachments = &[ attachment ];
  let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
    .logic_op_enable( false )
    .logic_op( vk::LogicOp::COPY )
    .attachments( attachments )
    .blend_constants([ 0.0, 0.0, 0.0, 0.0 ]);

  let vert_push_constant_range = vk::PushConstantRange::builder()
    .stage_flags( vk::ShaderStageFlags::VERTEX )
    .offset( 0 )
    .size( 64 );

  let frag_push_constant_range = vk::PushConstantRange::builder()
    .stage_flags( vk::ShaderStageFlags::FRAGMENT )
    .offset( 64 )
    .size( 4 );

  let set_layouts = &[ data.uniform_descriptor_set_layout, data.texture_descriptor_set_layout ];
  let push_constant_ranges = &[ vert_push_constant_range, frag_push_constant_range ];
  let layout_info = vk::PipelineLayoutCreateInfo::builder()
    .set_layouts( set_layouts )
    .push_constant_ranges( push_constant_ranges );

  data.pipeline_layout = device.create_pipeline_layout( &layout_info, None )?;

  let stages = &[ vert_stage, frag_stage ];
  let info = vk::GraphicsPipelineCreateInfo::builder()
    .stages( stages )
    .vertex_input_state( &vertex_input_state )
    .input_assembly_state( &input_assembly_state )
    .viewport_state( &viewport_state )
    .rasterization_state( &rasterization_state )
    .multisample_state( &multisample_state )
    .depth_stencil_state( &depth_stencil_state )
    .color_blend_state( &color_blend_state )
    .layout( data.pipeline_layout )
    .render_pass( data.render_pass )
    .subpass( 0 )
    .base_pipeline_handle( vk::Pipeline::null() )
    .base_pipeline_index( -1 );

  data.pipeline = device.create_graphics_pipelines( vk::PipelineCache::null(), &[ info ], None )?.0[ 0 ];

  device.destroy_shader_module( vert_shader_module, None );
  device.destroy_shader_module( frag_shader_module, None );

  Ok(())
}
