#version 460 core

// clang-format off
layout(location = {{ consts.vertex_attrib_indices.position }}) in vec3 inPos;
layout(location = {{ consts.vertex_attrib_indices.normals }}) in vec3 inNormal;
layout(location = {{ consts.vertex_attrib_indices.texcoords }}) in vec2 inTexcoords;
layout(location = {{ consts.vertex_attrib_indices.tangent }}) in vec4 inTangent;

layout(std140, binding = {{ consts.buffer_bindings.transforms }}) uniform Transforms
// clang-format on
{
    mat4 projection;
    mat4 view;
    mat4 model;
};

out VsOut
{
    vec2 texCoords;
    vec3 fragPos;
    vec3 normal;
    vec3 tangent;
    vec3 bitangent;
}
vsOut;

void main()
{
    gl_Position = projection * view * model * vec4(inPos, 1.0);

    vsOut.texCoords = inTexcoords;
    vsOut.fragPos = vec3(model * vec4(inPos, 1.0));

    mat3 normalMat = mat3(transpose(inverse(model)));

    // TODO: is normalization needed ?
    vsOut.normal = normalize(normalMat * inNormal);
    vsOut.tangent = normalize(normalMat * inTangent.w * inTangent.xyz);

    // modified Gram-Schmidt process
    vsOut.tangent = normalize(vsOut.tangent - dot(vsOut.tangent, vsOut.normal) * vsOut.normal);

    // Correct handedness with tangent.w
    vsOut.bitangent = normalize(inTangent.w * (cross(inNormal, inTangent.xyz)));
}
