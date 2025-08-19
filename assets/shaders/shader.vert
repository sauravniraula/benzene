#version 450

layout(set=0, binding = 0) uniform GlobalUniform {
  mat4 view;
  mat4 projection;
} gu;

layout(location = 0) in vec2 position;
layout(location = 1) in vec3 color;
layout(location = 2) in vec2 tex_coord;

layout(location = 0) out vec3 frag_color;
layout(location = 1) out vec2 frag_tex_coord;

void main() {
  gl_Position = gu.projection * gu.view * vec4(position, 0.0, 1.0);
  frag_color = color;
  frag_tex_coord = tex_coord;
}
