#version 450

layout(location = 0) in vec3 inPos;
layout(location = 1) in vec3 inColor;

layout(location = 0) out vec3 vColor;

layout(set = 0, binding = 0) uniform UBO {
    mat4 view_proj;
} ubo;

layout(push_constant) uniform Push {
    mat4 model;
} pc;

void main() {
    gl_Position = ubo.view_proj * pc.model * vec4(inPos, 1.0);
    vColor = inColor;
}

