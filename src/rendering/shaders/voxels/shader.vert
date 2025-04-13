#version 450

layout( binding=0 ) uniform UniformBufferObject {
  mat4 view;
  mat4 proj;
} ubo;

layout( push_constant ) uniform PushConstants {
  mat4 model;
} pcs;

layout( location=0 ) in vec3 inPos;
layout( location=1 ) in vec3 inNormal;
layout( location=2 ) in vec3 inPosInstance;
layout( location=3 ) in vec3 inColor;

layout( location=0 ) out vec3 outColor;
layout( location=1 ) out vec3 outNormal;
layout( location=2 ) out vec3 outLightPos;
layout( location=3 ) out vec3 outPos;
layout( location=4 ) out vec3 outPosModel;

vec3 lightPos = vec3( 10.0, 20.0, 10.0 );

void main() {
  vec4 worldPos = vec4( inPos + inPosInstance, 1.0 );

  gl_Position = ubo.proj * ubo.view * worldPos;

  outPos = worldPos.xyz;
  outPosModel = inPos;
  outColor = inColor;
  outNormal = inPos * inNormal;
  outLightPos = lightPos - worldPos.xyz;
}
