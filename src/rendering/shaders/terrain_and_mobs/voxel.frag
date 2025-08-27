#version 450

layout( push_constant ) uniform PushConstants {
  layout( offset=64 ) float opacity;
} pcs;

layout( location=0 ) in vec3 inColor;
layout( location=1 ) in vec3 inNormal;
layout( location=2 ) in vec3 inLightPos;
layout( location=3 ) in vec3 inPos;
layout( location=4 ) in vec3 inPosModel;

layout( location=0 ) out vec4 outColor;

void main() {
  // outColor = vec4( inColor, 1.0 );
	// vec3 N = normalize( inNormal );
	// vec3 lightDir = normalize( inLightPos - inPos );
	// float diffuse = max( dot( N, lightDir ), 0.0 );
	vec3 ambient = vec3( 0.02 ) * inColor;

  // Ambient Occlusion — bazujące na lokalnej pozycji względem środka instancji
	float dist = length( inPosModel );           // odległość od środka sześcianu
	float maxDist = length( vec3( 0.5 ) );       // maksymalna odległość do narożnika
	float ao = 1.0 - smoothstep( 0.0, maxDist, dist ); // centrum jaśniejsze, narożniki ciemniejsze
	ao = mix( 0.2, 1.0, ao );                  // siła AO (ciemność krawędzi)

	vec3 finalColor = inColor * ao;
	// vec3 finalColor = (ambient + diffuse) * inColor * ao;
	outColor = vec4( finalColor, pcs.opacity );
}
