#version 460

layout(set=0, binding=0) uniform GlobalUniform {
  mat4 view;
  mat4 projection;
  vec4 point_light;
} gu;

layout(set=1, binding=0) uniform sampler2D texture_sampler;

layout(location = 0) in vec4 in_position_ws;
layout(location = 1) in vec3 in_color;
layout(location = 2) in vec3 in_normal_ws;
layout(location = 3) in vec2 in_uv;

layout(location = 0) out vec4 out_color;

void main() {
  vec3 dir_to_light = (gu.point_light - in_position_ws).xyz;
  
  float diff = max(dot(normalize(dir_to_light), normalize(in_normal_ws)), 0.0);
  float atten = 1 / dot(dir_to_light, dir_to_light);

  vec3 color = in_color * diff * atten;

  out_color = vec4(color, 1.0);
}