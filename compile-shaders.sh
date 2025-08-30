function compile_terrain_with_mobs {
  glslc ./src/rendering/shaders/$1/voxel.vert -o ./src/rendering/shaders/$1/voxel.vert.spv
  glslc ./src/rendering/shaders/$1/voxel.frag -o ./src/rendering/shaders/$1/voxel.frag.spv
  glslc ./src/rendering/shaders/$1/mob.vert -o ./src/rendering/shaders/$1/mob.vert.spv
  glslc ./src/rendering/shaders/$1/mob.frag -o ./src/rendering/shaders/$1/mob.frag.spv
}

function compile {
  glslc ./src/rendering/shaders/$1/shader.vert -o ./src/rendering/shaders/$1/vert.spv
  glslc ./src/rendering/shaders/$1/shader.frag -o ./src/rendering/shaders/$1/frag.spv
}

# compile 'instances-textured-lighted'
# compile 'instances-untextured-unlighted'
# compile 'model-untextured-lighted'
# compile 'voxels'
# compile 'voxel_sides'
compile_terrain_with_mobs 'terrain_and_mobs'
