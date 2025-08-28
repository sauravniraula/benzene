#version 460

layout(set=0, binding=0) uniform GlobalUniform {
  mat4 view;
  mat4 projection;
  vec4 ambient_color;
} gu;

layout(set=1, binding=0) uniform PointLightUniform {
  vec4 points[16]; // xyz position, w used flag (1.0 used, 0.0 unused)
  vec4 colors[16]; // rgb color, w intensity
} plu;

layout(set=2, binding=0) uniform sampler2D texture_sampler;

layout(location = 0) in vec4 in_position_ws;
layout(location = 1) in vec3 in_color;
layout(location = 2) in vec3 in_normal_ws;
layout(location = 3) in vec2 in_uv;

layout(location = 0) out vec4 out_color;

void main() {
  // vec3 base_color = in_color;
  vec3 base_color = texture(texture_sampler, in_uv).xyz;

  vec3 accum = vec3(0.0);

  for (int i = 0; i < 16; ++i) {
    if (plu.points[i].w < 0.5) { continue; }
    vec3 light_pos = plu.points[i].xyz;
    vec3 light_color = plu.colors[i].xyz;
    float light_alpha = plu.colors[i].w;

    vec3 dir_to_light = (vec4(light_pos, 1.0) - in_position_ws).xyz;
    float diffusion = max(dot(normalize(dir_to_light), normalize(in_normal_ws)), 0.0);
    float atten = 1.0 / max(dot(dir_to_light, dir_to_light), 0.0001);
    accum += base_color * light_color * light_alpha * diffusion * atten;
  }

  // Ambient term
  vec3 ambient = gu.ambient_color.xyz * gu.ambient_color.w * base_color;

  out_color = vec4(ambient + accum, 1.0);
}