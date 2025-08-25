#version 460

layout(set=0, binding = 0) uniform GlobalUniform {
  mat4 view;
  mat4 projection;
  vec4 ambient_color;
} gu;

layout (push_constant) uniform constants {
  mat4 transform;
} pc;

layout(location = 0) in vec3 in_position_ms;
layout(location = 1) in vec3 in_color;
layout(location = 2) in vec3 in_normal_ms;
layout(location = 3) in vec2 in_uv;

layout(location = 0) out vec4 out_position_ws;
layout(location = 1) out vec3 out_color;
layout(location = 2) out vec3 out_normal_ws;
layout(location = 3) out vec2 out_uv;

void main() {
  vec4 pos_ws = pc.transform * vec4(in_position_ms, 1.0);
  vec3 normal_ws = (pc.transform * vec4(in_normal_ms, 0.0)).xyz;

  out_position_ws = pos_ws;
  out_color = in_color;
  out_normal_ws = normal_ws;
  out_uv = in_uv;

  gl_Position = gu.projection * gu.view * pos_ws;
}
