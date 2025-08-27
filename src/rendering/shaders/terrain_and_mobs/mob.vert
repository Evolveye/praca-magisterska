// layout( location=0 ) in vec3 inColor;
// layout( location=1 ) in vec3 inNormal;
// layout( location=2 ) in mat4 inTransform;

// void main() {
//   vec3 translation = vec3( inTransform[ 3 ].x, inTransform[ 3 ].y, inTransform[ 3 ].z );
//   mat3 rotation = mat3( inTransform );
//   vec3 rotatedPosition = rotation * translation;
//   vec4 worldPos = vec4( rotatedPosition, 1.0 );

//   gl_Position = ubo.proj * ubo.view * worldPos;

//   outPos = worldPos.xyz;
//   outPosModel = rotatedPosition;
//   outColor = inColor;
//   outNormal = rotation * inNormal;
//   outLightPos = lightPos - worldPos.xyz;
// }

#version 450

layout( binding=0 ) uniform UniformBufferObject {
  mat4 view;
  mat4 proj;
} ubo;

layout( push_constant ) uniform PushConstants {
  mat4 model;
} pcs;

layout( location=0 ) in vec3 inPos;
layout( location=1 ) in vec3 inColor;
layout( location=2 ) in mat4 inTransform;

layout( location=0 ) out vec3 outColor;
layout( location=1 ) out vec3 outNormal;
layout( location=2 ) out vec3 outLightPos;
layout( location=3 ) out vec3 outPos;
layout( location=4 ) out vec3 outPosModel;

vec3 lightPos = vec3( 10.0, 20.0, 10.0 );
// vec3 inColor = vec3( 1.0, 0.0, 0.0 );
vec3 inNormal = vec3( 1.0, 1.0, 1.0 );
vec3 inPosInstance = vec3( 1.0, 1.0, 1.0 );
mat3 rotation = mat3( 1.0 );

void main() {
  // mat3 rotation = mat3( inTransform );
  vec3 rotatedPosition = inPos;
  vec3 translation = vec3( inTransform[ 3 ].x, inTransform[ 3 ].y, inTransform[ 3 ].z );
  // vec4 worldPos = vec4( rotatedPosition + translation, 1.0 );
  vec4 worldPos = vec4( inPos, 1.0 );

  gl_Position = ubo.proj * ubo.view * worldPos;

  outPos = worldPos.xyz;
  outPosModel = rotatedPosition;
  outColor = inColor;
  outNormal = rotatedPosition * inNormal;
  outLightPos = lightPos - worldPos.xyz;
}
