#version 460 core
layout (location = 0) in vec3 aPos;

//uniform mat4 projection;
//uniform mat4 view;

layout (std140, binding = 1) uniform Transforms {
    mat4 projection;
    mat4 view;
    mat4 model;
};

out vec3 localPos;

void main()
{
    localPos = aPos;

    mat4 rotView = mat4(mat3(view)); // remove translation from the view matrix
    vec4 clipPos = projection * rotView * vec4(localPos, 1.0);

    gl_Position = clipPos.xyww;
}
