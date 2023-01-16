
vec3 fresnelSchlick(vec3 f0, float VoH) { return f0 + (1. - f0) * pow(clamp(1. - VoH, 0.0, 1.0), 5.); }

float fresnelSchlick(float f0, float VoH) { return f0 + (1. - f0) * pow(clamp(1. - VoH, 0.0, 1.0), 5.); }

vec3 fresnelSchlickRoughness(float VoH, vec3 f0, float roughness)
{
    return f0 + (max(vec3(1.0 - roughness), f0) - f0) * pow(clamp(1.0 - VoH, 0.0, 1.0), 5.0);
}

// GGX / Trowbridge-Reitz
// roughness is perceptual roughness (roughness squared)
float ggxDistribution(float NoH, float roughness)
{
    float asq = roughness * roughness;
    float denom = (NoH * NoH) * (asq - 1.) + 1.;

    return (asq) / (PI * denom * denom);
}

float geometryGgx(float NoV, float roughness)
{
    float asq = roughness * roughness;

    float denom = NoV + sqrt(asq + ((1 - asq) * (NoV * NoV)));

    return (2 * NoV) / denom;
}

// Smith
float smithGeometryShadowing(float NoV, float NoL, float roughness)
{
    float ggx2 = geometryGgx(NoV, roughness);
    float ggx1 = geometryGgx(NoL, roughness);

    return ggx1 * ggx2;
}

#ifdef ANISOTROPY
// Burley, “Physically-Based Shading at Disney.”
float anisotropicGgxDistribution(
    float roughness, float NoH, vec3 halfway, vec3 tangent, vec3 bitangent, float anisotropy)
{
    // Remapping from: Kulla and Conty, “Revisiting Physically Based Shading at Imageworks.”
    float tRoughness = max(roughness * (1.0 + anisotropy), ROUGHNESS_MIN);
    float bRoughness = max(roughness * (1.0 - anisotropy), ROUGHNESS_MIN);

    float ToH = dot(tangent, halfway);
    float BoH = dot(bitangent, halfway);

    float denom
        = ((ToH * ToH) / (tRoughness * tRoughness)) + ((BoH * BoH) / (bRoughness * bRoughness)) + (NoH * NoH);

    denom = denom * denom;

    return (1.0 / (PI * tRoughness * bRoughness)) * (1.0 / denom);
}

// Taken from: Guy and Agopian, “Physically Based Rendering in Filament.”
float anisotropicVSmithGgxCorrelated(
    float roughness, float NoV, float ToV, float BoV, float ToL, float BoL, float NoL, float anisotropy)
{
    // Remapping from: Kulla and Conty, “Revisiting Physically Based Shading at Imageworks.”
    float tRoughness = max(roughness * (1.0 + anisotropy), ROUGHNESS_MIN);
    float bRoughness = max(roughness * (1.0 - anisotropy), ROUGHNESS_MIN);

    float lambdaV = NoL * length(vec3(tRoughness * ToV, bRoughness * BoV, NoV));
    float lambdaL = NoV * length(vec3(tRoughness * ToL, bRoughness * BoL, NoL));

    float v = 0.5 / (lambdaV + lambdaL);
    return v;
}
#endif
