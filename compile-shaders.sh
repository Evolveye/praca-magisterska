function compile {
  glslc ./src/rendering/shaders/$1/shader.vert -o ./src/rendering/shaders/$1/vert.spv
  glslc ./src/rendering/shaders/$1/shader.frag -o ./src/rendering/shaders/$1/frag.spv
}

compile 'instances-textured-lighted'
