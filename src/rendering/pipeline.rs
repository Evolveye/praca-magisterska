use vulkanalia::bytecode::Bytecode;
use vulkanalia::prelude::v1_0::*;
use anyhow::Result;
use super::vertex::RendererModelDescriptions;
use super::renderer::{AppData, AppMode};

pub struct PipelineCreateInfoSet<'a> {
  // death code is needed, beacuse this fields are used only as references holders for "C" langs create info

  pub modules: Vec<vk::ShaderModule>,
  stages: Vec<vk::PipelineShaderStageCreateInfo>,
  #[allow(dead_code)] binding_descriptions: Vec<vk::VertexInputBindingDescription>,
  #[allow(dead_code)] attribute_descriptions: Vec<vk::VertexInputAttributeDescription>,
  vertex_input_state: vk::PipelineVertexInputStateCreateInfo,
  input_assembly_state: vk::PipelineInputAssemblyStateCreateInfo,
  #[allow(dead_code)] viewports: Vec<vk::ViewportBuilder>,
  #[allow(dead_code)] scissors: Vec<vk::Rect2DBuilder>,
  viewport_state: vk::PipelineViewportStateCreateInfo,
  rasterization_state: vk::PipelineRasterizationStateCreateInfo,
  multisample_state: vk::PipelineMultisampleStateCreateInfo,
  depth_stencil_state: vk::PipelineDepthStencilStateCreateInfo,
  #[allow(dead_code)] color_blend_state_attachments: Vec<vk::PipelineColorBlendAttachmentState>,
  color_blend_state: vk::PipelineColorBlendStateCreateInfo,

  device: &'a Device
}

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

unsafe fn create_pipeline_layout( device:&Device, data:&mut AppData ) -> Result<vk::PipelineLayout> {
  let vert_push_constant_range = vk::PushConstantRange::builder()
    .stage_flags( vk::ShaderStageFlags::VERTEX )
    .offset( 0 )
    .size( 64 );

  let frag_push_constant_range = vk::PushConstantRange::builder()
    .stage_flags( vk::ShaderStageFlags::FRAGMENT )
    .offset( 64 )
    .size( 4 );

  let set_layouts = match data.mode {
    AppMode::Voxels | AppMode::VoxelSides | AppMode::VoxelSidesStrip | AppMode::TerrainAndMobs => vec![ data.uniform_descriptor_set_layout ],
    AppMode::InstancesTexturedLighted | AppMode::InstancesUntexturedUnlighted | AppMode::Model => vec![ data.uniform_descriptor_set_layout, data.texture_descriptor_set_layout ],
    _ => unreachable!()
  };

  let push_constant_ranges = &[ vert_push_constant_range, frag_push_constant_range ];
  let layout_info = vk::PipelineLayoutCreateInfo::builder()
    .set_layouts( &set_layouts )
    .push_constant_ranges( push_constant_ranges );

  let leyout = device.create_pipeline_layout( &layout_info, None )?;
  Ok( leyout )
}

pub fn create_pipeline_create_info_set<'a, T:RendererModelDescriptions>( device:&'a Device, data:&mut AppData, primitive_topology:vk::PrimitiveTopology, polygon_mode:vk::PolygonMode, shaders:(&[u8], &[u8]) ) -> Result<PipelineCreateInfoSet<'a>> {
  let ( (vert_shader_module, vert_stage), (frag_shader_module, frag_stage) ) = unsafe {(
    create_shader_stage( device, shaders.0, vk::ShaderStageFlags::VERTEX )?,
    create_shader_stage( device, shaders.1, vk::ShaderStageFlags::FRAGMENT )?,
  )};

  let (binding_descriptions, attribute_descriptions) = {
    let binding_descriptions = vec![ T::binding_description(), T::instances_binding_description() ];

    let mut attribute_descriptions = Vec::new();
    attribute_descriptions.extend( T::attribute_description() );
    attribute_descriptions.extend( T::instances_attribute_description() );

    (binding_descriptions, attribute_descriptions)
  };

  let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
    .vertex_binding_descriptions( &binding_descriptions )
    .vertex_attribute_descriptions( &attribute_descriptions )
    .build();

  let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
    .topology( primitive_topology )
    .primitive_restart_enable( false )
    .build();

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

  let viewports = vec![ viewport ];
  let scissors = vec![ scissor ];
  let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
    .viewports( &viewports )
    .scissors( &scissors )
    .build();

  let rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
    .depth_clamp_enable( false )
    .rasterizer_discard_enable( false )
    .polygon_mode( polygon_mode )
    .line_width( 5.0 )
    .cull_mode( vk::CullModeFlags::BACK )
    .front_face( vk::FrontFace::COUNTER_CLOCKWISE )
    .depth_bias_enable( false )
    .build();

  let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
    .rasterization_samples( data.msaa_samples )
    .sample_shading_enable( true )
    .min_sample_shading( 0.2 )
    .build();

  let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::builder()
    .depth_test_enable( true )
    .depth_write_enable( true )
    .depth_compare_op( vk::CompareOp::LESS )
    .depth_bounds_test_enable( false )
    .min_depth_bounds( 0.0 )
    .max_depth_bounds( 1.0 )
    .stencil_test_enable( false )
    // .front( vk::StencilOpState )
    // .back( vk::StencilOpState );
    .build();

  let attachment = vk::PipelineColorBlendAttachmentState::builder()
    .color_write_mask( vk::ColorComponentFlags::all() )
    .blend_enable( true )
    .src_color_blend_factor( vk::BlendFactor::SRC_ALPHA )
    .dst_color_blend_factor( vk::BlendFactor::ONE_MINUS_SRC_ALPHA )
    .color_blend_op( vk::BlendOp::ADD )
    .src_alpha_blend_factor( vk::BlendFactor::ONE )
    .dst_alpha_blend_factor( vk::BlendFactor::ZERO )
    .alpha_blend_op( vk::BlendOp::ADD )
    .build();

  let color_blend_state_attachments = vec![ attachment ];
  let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
    .logic_op_enable( false )
    .logic_op( vk::LogicOp::COPY )
    .attachments( &color_blend_state_attachments )
    .blend_constants([ 0.0, 0.0, 0.0, 0.0 ])
    .build();

  Ok( PipelineCreateInfoSet::<'a> {
    modules: vec![ vert_shader_module, frag_shader_module ],
    stages: vec![ vert_stage.build(), frag_stage.build() ],
    binding_descriptions,
    attribute_descriptions,
    vertex_input_state,
    input_assembly_state,
    viewports,
    scissors,
    viewport_state,
    rasterization_state,
    multisample_state,
    depth_stencil_state,
    color_blend_state_attachments,
    color_blend_state,
    device,
  } )
}

impl<'a> PipelineCreateInfoSet<'a> {
  pub unsafe fn get_builder( &'a self, data:&mut AppData ) -> vk::GraphicsPipelineCreateInfoBuilder<'a> {
    let builder = vk::GraphicsPipelineCreateInfo::builder()
      .stages( &self.stages )
      .vertex_input_state( &self.vertex_input_state )
      .input_assembly_state( &self.input_assembly_state )
      .viewport_state( &self.viewport_state )
      .rasterization_state( &self.rasterization_state )
      .multisample_state( &self.multisample_state )
      .depth_stencil_state( &self.depth_stencil_state )
      .color_blend_state( &self.color_blend_state )
      .render_pass( data.render_pass )
      .subpass( 0 )
      .base_pipeline_handle( vk::Pipeline::null() )
      .base_pipeline_index( -1 );

    builder
  }
}

impl<'a> Drop for PipelineCreateInfoSet<'a> {
  fn drop( &mut self ) {
    unsafe {
      for module in &self.modules {
        self.device.destroy_shader_module( *module, None );
      }
    }
  }
}

pub unsafe fn create_pipeline_edges<T:RendererModelDescriptions>( device:&Device, data:&mut AppData, info_set:&PipelineCreateInfoSet, primitive_topology:vk::PrimitiveTopology, shaders:(&[u8], &[u8]) ) -> Result<(vk::Pipeline, vk::PipelineLayout)> {
  let ((vert_shader_module, vert_stage), (frag_shader_module, frag_stage)) = unsafe {(
    create_shader_stage( device, shaders.0, vk::ShaderStageFlags::VERTEX )?,
    create_shader_stage( device, shaders.1, vk::ShaderStageFlags::FRAGMENT )?,
  )};

  let (binding_descriptions, attribute_descriptions) = {
    let binding_descriptions = vec![ T::binding_description(), T::instances_binding_description() ];

    let mut attribute_descriptions = Vec::new();
    attribute_descriptions.extend( T::attribute_description() );
    attribute_descriptions.extend( T::instances_attribute_description() );

    (binding_descriptions, attribute_descriptions)
  };

  let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
    .vertex_binding_descriptions( &binding_descriptions )
    .vertex_attribute_descriptions( &attribute_descriptions )
    .build();

  let pipeline_layout = create_pipeline_layout( device, data )?;
  let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
    .topology( primitive_topology )
    .primitive_restart_enable( false );

  let builder = info_set.get_builder( data );
  let stages = &[vert_stage.build(), frag_stage.build()];
  let builder = builder
    .stages( stages )
    .vertex_input_state( &vertex_input_state )
    .input_assembly_state( &input_assembly_state )
    .layout( pipeline_layout );

  let pipeline = device.create_graphics_pipelines( vk::PipelineCache::null(), &[ builder ], None )?.0[ 0 ];

  device.destroy_shader_module( vert_shader_module, None );
  device.destroy_shader_module( frag_shader_module, None );

  Ok( (pipeline, pipeline_layout) )
}

pub unsafe fn create_pipeline<T:RendererModelDescriptions>( device:&Device, data:&mut AppData, info_set:&PipelineCreateInfoSet ) -> Result<(vk::Pipeline, vk::PipelineLayout)> {
  let pipeline_layout = create_pipeline_layout( device, data )?;
  let builder = info_set.get_builder( data ).layout( pipeline_layout );
  let pipeline = device.create_graphics_pipelines( vk::PipelineCache::null(), &[ builder ], None )?.0[ 0 ];
  Ok( (pipeline, pipeline_layout) )
}
