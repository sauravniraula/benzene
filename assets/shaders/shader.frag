#version 450

layout(set=0, binding = 0) uniform GlobalUniform {
  mat4 view;
  mat4 projection;
  vec3 light_direction;
} gu;

layout(location = 0) in vec3 in_color;
layout(location = 1) in vec3 in_normal;


layout(location = 0) out vec4 out_color;

void main() {
  float diffuse = min(max(dot(-normalize(gu.light_direction), normalize(in_normal)), 0.0) + 0.02, 1.0);

  out_color = vec4(in_color * diffuse, 1.0);
}