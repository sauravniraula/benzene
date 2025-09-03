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

layout(set=1, binding=1) uniform DirectionalLightUniform {
  vec4 directions[16];
  vec4 colors[16];
} dlu;

layout(set=1, binding=2) uniform SpotLightUniform {
  vec4 positions[16];
  vec4 directions[16];
  vec4 colors[16];
} slu;

layout(set=2, binding=0) uniform sampler2D texture_sampler;

layout(location = 0) in vec4 in_position_ws;
layout(location = 1) in vec3 in_color;
layout(location = 2) in vec3 in_normal_ws;
layout(location = 3) in vec2 in_uv;

layout(location = 0) out vec4 out_color;

void main() {
  vec3 in_normal_ws_norm = normalize(in_normal_ws);

  vec3 base_color = texture(texture_sampler, in_uv).xyz;

  vec3 accum = vec3(0.0);

  // Point Light
  for (int i = 0; i < 16; ++i) {
    if (plu.points[i].w < 0.5) { continue; }
    vec3 light_pos = plu.points[i].xyz;
    vec3 light_color = plu.colors[i].xyz;
    float light_intensity = plu.colors[i].w;

    vec3 to_light_vector = light_pos - in_position_ws.xyz;
    float diffusion = max(dot(normalize(to_light_vector), in_normal_ws_norm), 0.0);
    float atten = 1.0 / max(dot(to_light_vector, to_light_vector), 0.0001);
    accum += base_color * light_color * light_intensity * diffusion * atten;
  }

  // Directional Light
  for (int i = 0; i < 16; ++i) {
    if (dlu.directions[i].w < 0.5) { continue; }
    vec3 light_color = dlu.colors[i].xyz;
    float light_intensity = dlu.colors[i].w;

    float diffusion = max(dot(-normalize(dlu.directions[i].xyz), in_normal_ws_norm), 0.0);
    accum += base_color * light_color * light_intensity * diffusion;
  }

  // Spot Light
  for (int i = 0; i < 16; ++i) {
    if (slu.positions[i].w < 0.5) { continue; }
    vec3 light_pos = slu.positions[i].xyz;
    vec3 light_color = slu.colors[i].xyz;
    float light_intensity = slu.colors[i].w;

    vec3 to_light_vector = light_pos - in_position_ws.xyz;
    vec3 to_light_vector_norm = normalize(to_light_vector);
    float in_spot = max(dot(-to_light_vector_norm, normalize(slu.directions[i].xyz)), 0.0);
    float diffusion = max(dot(to_light_vector_norm, in_normal_ws_norm), 0.0);
    float atten = 1.0 / max(dot(to_light_vector, to_light_vector), 0.0001);

    // Improved soft cutoff just within this condition block
    if (in_spot <= 0.7) {
      in_spot = 0.0;
    } else {
      in_spot = clamp((in_spot - 0.7) / 0.3, 0.0, 1.0);
      in_spot = in_spot * in_spot;
    }

    accum += base_color * light_color * light_intensity * in_spot * diffusion * atten;
  }

  // Ambient term
  vec3 ambient = gu.ambient_color.xyz * gu.ambient_color.w * base_color;

  out_color = vec4(ambient + accum, 1.0);
  // out_color = vec4(1.0, 1.0, 1.0, 1.0);
}