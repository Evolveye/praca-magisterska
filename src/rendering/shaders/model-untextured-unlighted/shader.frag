#version 450

layout( push_constant ) uniform PushConstants {
  layout( offset=64 ) float opacity;
} pcs;

layout( location=0 ) in vec3 fragColor;

layout( location=0 ) out vec4 outColor;

void main() {
  outColor = vec4( fragColor, pcs.opacity );
}
