#version 450

layout(set=0, binding=0) uniform GlobalUniform {
  mat4 view;
  mat4 projection;
  vec3 light_direction;
} gu;

layout(set=1, binding=0) uniform sampler2D texture_sampler;

layout(location = 0) in vec3 in_color;
layout(location = 1) in vec3 in_normal;
layout(location = 2) in vec2 in_uv;

layout(location = 0) out vec4 out_color;

void main() {
  float diffuse = min(max(dot(-normalize(gu.light_direction), normalize(in_normal)), 0.0) + 0.02, 1.0);

  vec3 sampled_color = texture(texture_sampler, in_uv).xyz;
  out_color = vec4(sampled_color * diffuse, 1.0);
}