
// clang-format off
layout(std140, binding = {{ consts.buffer_bindings.pbr_material }}) uniform PbrMaterial
// clang-format on
{
    uniform vec4 baseColorFactor;
    uniform vec4 emissiveFactor;
    uniform float metallicFactor;
    uniform float roughnessFactor;
    uniform float normalScale;
    uniform float occlusionStrength;

    uniform float clearcoatIntensityFactor;
    uniform float clearcoatRoughnessFactor;
    uniform float clearcoatNormalScale;

    uniform float anisotropy;
};
