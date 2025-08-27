use std::hash::{ Hash, Hasher };
use vulkanalia::{
  vk,
  prelude::v1_0::Device,
};

pub type Vec2 = cgmath::Vector2<f32>;
pub type Vec3 = cgmath::Vector3<f32>;


pub enum DrawMode {
  FULL,
  EDGES,
}

pub trait Renderable {
  unsafe fn render( &self, device:&Device, command_buffer:vk::CommandBuffer );
  fn get_draw_mode( &self ) -> DrawMode {
    DrawMode::FULL
  }
}

pub trait RendererModelDescriptions {
  fn binding_description() -> vk::VertexInputBindingDescription;
  fn attribute_description() -> Vec<vk::VertexInputAttributeDescription>;
  fn instances_binding_description() -> vk::VertexInputBindingDescription;
  fn instances_attribute_description() -> Vec<vk::VertexInputAttributeDescription>;
}



#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct SimpleVertex {
  pub pos: Vec3,
  pub color: Vec3,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
  pub pos: Vec3,
  pub color: Vec3,
  pub normal: Vec3,
  pub tex_coord: Vec2,
}



impl Vertex {
  pub const fn new( pos:Vec3, color:Vec3, normal:Vec3, tex_coord:Vec2 ) -> Self {
    Self { pos, color, normal, tex_coord }
  }
}

impl PartialEq for Vertex {
  fn eq( &self, other:&Self ) -> bool {
    self.pos == other.pos && self.color == other.color && self.tex_coord == other.tex_coord
  }
}

impl Eq for Vertex {}

impl Hash for Vertex {
  fn hash<H:Hasher>( &self, state:&mut H ) {
    self.pos[ 0 ].to_bits().hash( state );
    self.pos[ 1 ].to_bits().hash( state );
    self.pos[ 2 ].to_bits().hash( state );
    self.color[ 0 ].to_bits().hash( state );
    self.color[ 1 ].to_bits().hash( state );
    self.color[ 2 ].to_bits().hash( state );
    self.normal[ 0 ].to_bits().hash( state );
    self.normal[ 1 ].to_bits().hash( state );
    self.normal[ 2 ].to_bits().hash( state );
    self.tex_coord[ 0 ].to_bits().hash( state );
    self.tex_coord[ 1 ].to_bits().hash( state );
  }
}
