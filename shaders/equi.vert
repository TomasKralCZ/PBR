#version 460 core
layout (location = 0) in vec3 aPos;

out vec3 localPos;

layout(location = 2) uniform mat4 view;
layout(location = 3) uniform mat4 projection;

void main()
{
    localPos = aPos;
    gl_Position =  projection * view * vec4(localPos, 1.0);
}
