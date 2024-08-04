#version 450

layout( set=1, binding=0 ) uniform sampler2D texSampler;

layout( push_constant ) uniform PushConstants {
  layout( offset=64 ) float opacity;
} pcs;

layout( location=0 ) in vec3 inColor;
layout( location=1 ) in vec2 inTexCoord;
layout( location=2 ) in vec3 inNormal;
layout( location=3 ) in vec3 inLightPos;
layout( location=4 ) in vec3 inPos;

layout( location=0 ) out vec4 outColor;

void main() {
	vec3 N = normalize( inNormal );
	vec3 lightDir = normalize( inLightPos - inPos );
	float diffuse = max( dot( N, lightDir ), 0.0 );
	vec3 ambient = vec3( 0.02 ) * inColor;

	vec3 finalColor = (ambient + diffuse) * inColor;
	outColor = vec4( finalColor, pcs.opacity );
	outColor *= texture( texSampler, inTexCoord );
}
