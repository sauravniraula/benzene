#version 450

layout(location=0) in vec3 inPosition;
layout(location=1) in vec3 inColor;


layout(set=0, binding=0) uniform CameraBufferObject {
  mat4 view;
  mat4 projection;
} cbo;


layout(location=0) out vec3 outColor;

void main() {
  gl_Position = cbo.projection * cbo.view * vec4(inPosition, 1.0);
  outColor = vec3(0.5, 0.5, 0.5);
}