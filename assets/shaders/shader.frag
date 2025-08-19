#version 450

layout(set = 0, binding = 1) uniform sampler2D texSampler;

layout(location = 0) in vec3 frag_color;
layout(location = 1) in vec2 frag_tex_coord;

layout(location = 0) out vec4 out_color;

void main() {
  out_color = vec4(frag_color, 1.0);
  out_color = texture(texSampler, frag_tex_coord);
}