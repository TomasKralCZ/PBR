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

    vec3 normal = normalize(normalMat * inNormal);
    vec3 tangent = normalize(normalMat * inTangent.xyz);
    // "The bitangent vectors MUST be computed by taking the cross product of the normal
    // and tangent XYZ vectors and multiplying it against the W component of the tangent"
    vec3 bitangent = normalize(inTangent.w * (cross(normal, tangent)));

    vsOut.normal = normal;
    vsOut.tangent = tangent;
    vsOut.bitangent = bitangent;
}
