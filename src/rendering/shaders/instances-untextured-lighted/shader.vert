#version 450

layout( binding=0 ) uniform UniformBufferObject {
  mat4 view;
  mat4 proj;
  // mat4 model;
} ubo;

layout( push_constant ) uniform PushConstants {
  mat4 model;
  // mat4 view; ?
  // mat4 proj; ?
} pcs;

layout( location=0 ) in vec3 inPosition;
layout( location=1 ) in vec3 inColor;
layout( location=2 ) in vec3 inNormal;
layout( location=3 ) in vec3 inInstance;

layout( location=0 ) out vec3 outColor;
layout( location=2 ) out vec3 outNormal;
layout( location=3 ) out vec3 outLightPos;
layout( location=4 ) out vec3 outPos;

vec3 lightPos = vec3( 0.0, 10.0, -4.0 );

void main() {
  vec4 worldPos = pcs.model * vec4( inPosition + inInstance, 1.0 );

  gl_Position = ubo.proj * ubo.view * worldPos;

  outPos = worldPos.xyz;
  outColor = vec3( 1.0, 0.0, 0.0 );
  outNormal = mat3( pcs.model ) * inNormal;
  outLightPos = lightPos - worldPos.xyz;
}
