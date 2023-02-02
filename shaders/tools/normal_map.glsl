
struct NormalBasis {
    vec3 normal;
    vec3 tangent;
    vec3 bitangent;
};  

#if defined(NORMAL_MAP) || defined(CLEARCOAT_NORMAL_MAP)
NormalBasis getNormalFromMap(sampler2D tex, float scaleNormal, vec3 viewDir)
{
    // https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#_material_normaltextureinfo_scale
    vec3 tangentNormal
        = normalize((texture(tex, vsOut.texCoords).xyz) * 2.0 - 1.0) * vec3(scaleNormal, scaleNormal, 1.0);

    vec3 tangent = normalize(vsOut.tangent);
    vec3 bitangent = normalize(vsOut.bitangent);

    mat3 tbn = mat3(tangent, bitangent, normalize(vsOut.normal));

    vec3 adjustedNormal = normalize(tbn * tangentNormal);

    NormalBasis tb;
    tb.normal = adjustedNormal;

    // Math from Real-Time rendering
    // tangent and bitangent vectors need to be adjusted for anisotropic BRDFs and for measured BRDFs
    tb.tangent = tangent - dot(tangent, tb.normal) * tb.normal;
    tb.bitangent = bitangent - dot(bitangent, tb.normal) * tb.normal;

    return tb;
}
#endif
