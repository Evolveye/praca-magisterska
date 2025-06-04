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
layout( location=4 ) in uint inDirection;

layout( location=0 ) out vec3 outColor;
layout( location=1 ) out vec3 outNormal;
layout( location=2 ) out vec3 outLightPos;
layout( location=3 ) out vec3 outPos;
layout( location=4 ) out vec3 outPosModel;

vec3 lightPos = vec3( 10.0, 20.0, 10.0 );

mat3 getRotationMatrix( uint dir ) {
    if (dir == 1u) { // LEFT (-X)
        return mat3(
             0, 1, 0,
            -1, 0, 0,
             0, 0, 1
        );
    } else if (dir == 2u) { // RIGHT (+X)
        return mat3(
             0, -1, 0,
             1,  0, 0,
             0,  0, 1
        );
    } else if (dir == 4u) { // BOTTOM
        return mat3(
             1,  0,  0,
             0, -1,  0,
             0,  0, -1
        );
    } else if (dir == 5u) { // FRONT (+Z)
        return mat3(
             1,  0,  0,
             0,  0,  1,
             0, -1,  0
        );

    } else if (dir == 6u) { // BACK (-Z)
        return mat3(
             1,  0,  0,
             0,  0, -1,
             0,  1,  0
        );
    }

    return mat3(1.0); // TOP
}

void main() {
  mat3 rotation = getRotationMatrix( inDirection );
  vec3 rotatedPosition = rotation * inPos;
  vec4 worldPos = vec4( rotatedPosition + inPosInstance, 1.0 );

  gl_Position = ubo.proj * ubo.view * worldPos;

  outPos = worldPos.xyz;
  outPosModel = rotatedPosition;
  outColor = inColor;
  outNormal = rotatedPosition * inNormal;
  outLightPos = lightPos - worldPos.xyz;
}
