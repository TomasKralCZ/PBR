
#if defined(NORMAL_MAP) || defined(CLEARCOAT_NORMAL_MAP)
vec3 getNormalFromMap(sampler2D tex, float scaleNormal, vec3 viewDir)
{
    // https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#_material_normaltextureinfo_scale
    vec3 tangentNormal
        = normalize((texture(tex, vsOut.texCoords).xyz) * 2.0 - 1.0) * vec3(scaleNormal, scaleNormal, 1.0);

    mat3 tbn = mat3(normalize(vsOut.tangent), normalize(vsOut.bitangent), normalize(vsOut.normal));

    return normalize(tbn * tangentNormal);
}
#endif
