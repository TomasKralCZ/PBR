#version 420 core

layout(location = 0) in vec3 inPos;
layout(location = 1) in vec3 inNormal;
layout(location = 2) in vec2 inTexcoords;

layout (std140, binding = 1) uniform Transforms {
    mat4 projection;
    mat4 view;
    mat4 model;
};

out VsOut {
    vec2 texCoords;
    vec3 normal;
    vec3 fragPos;
} vsOut;

void main() {
    gl_Position = projection * view * model * vec4(inPos, 1.0);

    vsOut.texCoords = inTexcoords;
    vsOut.normal = mat3(transpose(inverse(model))) * inNormal;
    vsOut.fragPos = vec3(model * vec4(inPos, 1.0));
}
