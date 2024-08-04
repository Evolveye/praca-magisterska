#version 450

layout( binding=0 ) uniform UniformBufferObject {
  mat4 view;
  mat4 proj;
} ubo;

layout( push_constant ) uniform PushConstants {
  mat4 model;
} pcs;

layout( location=0 ) in vec3 inPosition;
layout( location=3 ) in vec2 inTexCoord;

layout( location=1 ) out vec2 outTexCoord;

void main() {
  vec4 worldPos = pcs.model * vec4( inPosition, 1.0 );
  gl_Position = ubo.proj * ubo.view * worldPos;

  outTexCoord = inTexCoord;
}
