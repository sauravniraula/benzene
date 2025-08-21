#version 450

layout(set=0, binding = 0) uniform GlobalUniform {
  mat4 view;
  mat4 projection;
  vec3 light_direction;
} gu;

layout(location = 0) in vec3 in_position;
layout(location = 1) in vec3 in_color;
layout(location = 2) in vec3 in_normal;
layout(location = 3) in vec2 in_uv;

layout(location = 0) out vec3 out_color;
layout(location = 1) out vec3 out_normal;
layout(location = 2) out vec2 out_uv;

void main() {
  gl_Position = gu.projection * gu.view * vec4(in_position, 1.0);

  out_color = in_color;
  out_normal = in_normal;  
  out_uv = in_uv;
}
