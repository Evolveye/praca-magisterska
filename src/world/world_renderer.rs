use cgmath::{ vec2, vec3, Vector3 };
use vulkanalia::prelude::v1_0::*;
use vulkanalia::vk;

use crate::rendering::vertex::{RendererModelDescriptions, Vertex};

use super::world_holder::Voxel;

type Vec2 = cgmath::Vector2<f32>;
type Vec3 = cgmath::Vector3<f32>;

pub trait WorldRenderer {
    fn render_world( &self );
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct VoxelInstance {
  pub translate: Vec3
}

impl RendererModelDescriptions for Voxel {
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

      vec![ pos, color, normal ]
    }

    fn instances_binding_description() -> vk::VertexInputBindingDescription {
      vk::VertexInputBindingDescription::builder()
        .binding( 1 )
        .stride( size_of::<VoxelInstance>() as u32 )
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