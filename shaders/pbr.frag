#version 460 core
// clang-format off
//#defines

{% include "consts.glsl" %}

{% include "structs/pbrVsOut.glsl" %}
{% include "structs/pbrMaterial.glsl" %}
{% include "structs/pbrTextures.glsl" %}
{% include "structs/lighting.glsl" %}
{% include "structs/settings.glsl" %}

{% include "ibl/brdf_sampling.glsl" %}

{% include "tools/tonemap.glsl" %}
{% include "tools/normal_map.glsl" %}

{% include "brdf.glsl" %}

// clang-format on

#line 22
out vec4 FragColor;

// Parameters that stay same for the whole pixel
struct ShadingParams {
    vec4 albedo;

    vec3 viewDir;
    NormalBasis tb;
    float NoV;

    float roughness;
    float metalness;
    vec3 f0;

#ifdef CLEARCOAT
    vec3 clearcoatNormal;
    float clearcoatNoV;
    float clearcoatRoughness;
    float clearcoatIntensity;
#endif
};

#ifdef CLEARCOAT
vec3 clearcoatBrdf(ShadingParams sp, out float fresnel, vec3 halfway, vec3 lightDir, float VoH)
{
    float clearcoatNoH = max(dot(halfway, sp.clearcoatNormal), 0.0);
    float clearcoatNoL = max(dot(lightDir, sp.clearcoatNormal), 0.0);

    // clearcoat BRDF
    float D = distributionGgx(clearcoatNoH, sp.clearcoatRoughness);
    float V = visibilitySmithHeightCorrelatedGgx(sp.NoV, clearcoatNoL, sp.clearcoatRoughness);
    // Coating IOR is 1.5 (f0 == 0.04) - equivalent to polyurethane
    fresnel = fresnelSchlick(DIELECTRIC_FRESNEL, VoH) * sp.clearcoatIntensity;

    return D * V * vec3(fresnel);
}
#endif

#ifdef ANISOTROPY
void baseSpecularAnisotropic(ShadingParams sp, inout vec3 specular, inout vec3 fresnel, float NoH, float NoL,
    float VoH, vec3 halfway, vec3 lightDir)
{
    float D
        = distributionAnisotropicGgx(sp.roughness, NoH, halfway, sp.tb.tangent, sp.tb.bitangent, anisotropy);

    float ToV = dot(sp.tb.tangent, sp.viewDir);
    float BoV = dot(sp.tb.bitangent, sp.viewDir);
    float ToL = dot(sp.tb.tangent, lightDir);
    float BoL = dot(sp.tb.bitangent, lightDir);

    float V
        = visibilitySmithHeightCorrelatedGgxAniso(sp.roughness, sp.NoV, ToV, BoV, ToL, BoL, NoL, anisotropy);

    fresnel = fresnelSchlick(sp.f0, VoH);
    specular = D * V * fresnel;
}
#endif

void baseSpecularIsotropic(
    ShadingParams sp, inout vec3 specular, inout vec3 fresnel, float NoH, float NoL, float VoH)
{
    float D = distributionGgx(NoH, sp.roughness);
    fresnel = fresnelSchlick(sp.f0, VoH);
    float V = visibilitySmithHeightCorrelatedGgx(sp.NoV, NoL, sp.roughness);
    specular = V * D * fresnel;
}

vec3 calculateDirectLighting(ShadingParams sp)
{
    vec3 totalRadiance = vec3(0.);

    for (int i = 0; i < lights; i++) {
        vec3 lightDir = normalize(lightPositions[i].xyz - vsOut.fragPos);
        vec3 halfway = normalize(sp.viewDir + lightDir);
        float VoH = max(dot(halfway, sp.viewDir), 0.0);
        float NoH = max(dot(sp.tb.normal, halfway), 0.0);
        float LoH = max(dot(lightDir, halfway), 0.0);
        float NoL = max(dot(sp.tb.normal, lightDir), 0.0);

        // TODO: should add attenuation...
        vec3 light = lightColors[i].xyz;

        vec3 fresnel;
        vec3 specular;
#ifdef ANISOTROPY
        if (anisotropy != 0.) {
            baseSpecularAnisotropic(sp, specular, fresnel, NoH, NoL, VoH, halfway, lightDir);
        } else {
            baseSpecularIsotropic(sp, specular, fresnel, NoH, NoL, VoH);
        }
#else
        baseSpecularIsotropic(sp, specular, fresnel, NoH, NoL, VoH);
#endif

        // Simple way of setting the strength of the diffuse lobe is (1 - fresnel)
        vec3 kd = vec3(1.0) - fresnel;
        // Metals have no diffuse
        kd *= 1.0 - sp.metalness;

        vec3 diffuse = kd;

        switch (diffuseType) {
        case DIFFUSE_TYPE_LAMBERT:
            diffuse *= diffuseLambert(sp.albedo.xyz);
            break;
        case DIFFUSE_TYPE_FROSTBITE:
            diffuse *= diffuseFrostbite(sp.albedo.xyz, sp.roughness, NoL, LoH, sp.NoV);
            break;
        case DIFFUSE_TYPE_CODWWII:
            diffuse *= diffuseCodWWII(sp.albedo.xyz, sp.roughness, NoL, LoH, NoH, sp.NoV);
            break;
        }

        diffuse /= PI;

#ifdef CLEARCOAT
        vec3 brdf;
        if (clearcoatEnabled) {
            float clearcoatFresnel;
            vec3 clearcoatColor = clearcoatBrdf(sp, clearcoatFresnel, halfway, lightDir, VoH);

            // Energy loss due to the clearcoat layer is given by 1 - clearcoatFresnel
            brdf = (diffuse + specular) * (1. - clearcoatFresnel) + clearcoatColor;
        } else {
            brdf = diffuse + specular;
        }
#else
        vec3 brdf = diffuse + specular;
#endif

        totalRadiance += brdf * light * NoL;
    }

    return totalRadiance;
}

#ifdef CLEARCOAT
vec3 calcClearcoatIBL(ShadingParams sp, inout vec3 baseLayerEnvLight)
{
    vec3 clearcoatReflectDir = reflect(-sp.viewDir, sp.clearcoatNormal);
    float clearcoatFresnel = fresnelSchlick(DIELECTRIC_FRESNEL, sp.clearcoatNoV) * sp.clearcoatIntensity;

    // clang-format off
    const float MAX_REFLECTION_LOD = float({{ consts.ibl.cubemap_roughnes_levels - 1 }});
    // clang-format on

    // Apply clearcoat IBL
    vec3 clearcoatPrefilteredLight
        = textureLod(prefilterMap, clearcoatReflectDir, sp.clearcoatRoughness * MAX_REFLECTION_LOD).rgb;
    vec2 clearcoatDfg = texture(brdfLut, vec2(sp.clearcoatNoV, sp.clearcoatRoughness)).rg;
    vec3 clearcoatIblSpecular
        = clearcoatPrefilteredLight * (clearcoatFresnel * clearcoatDfg.x + clearcoatDfg.y);

    // base layer attenuation for energy compensation
    baseLayerEnvLight *= 1.0 - clearcoatFresnel;

    return clearcoatIblSpecular;
}
#endif

#ifdef ANISOTROPY
// Taken from: Guy and Agopian, “Physically Based Rendering in Filament.”
// Based on
// McAuley: Rendering the World of Far Cry 4.
vec3 anisotropyIblBentReflectDir(ShadingParams sp)
{
    vec3 anisotropicDirection = anisotropy >= 0.0 ? sp.tb.bitangent : sp.tb.tangent;
    vec3 anisotropicTangent = cross(anisotropicDirection, sp.viewDir);
    vec3 anisotropicNormal = cross(anisotropicTangent, anisotropicDirection);
    vec3 bentNormal = normalize(mix(sp.tb.normal, anisotropicNormal, anisotropy));
    vec3 reflectDir = reflect(-sp.viewDir, bentNormal);

    return reflectDir;
}
#endif

vec3 calculateIBL(ShadingParams sp)
{
#ifdef ANISOTROPY
    vec3 reflectDir = anisotropyIblBentReflectDir(sp);
#else
    vec3 reflectDir = reflect(-sp.viewDir, sp.tb.normal);
#endif

    // clang-format off
    const float MAX_REFLECTION_LOD = float({{ consts.ibl.cubemap_roughnes_levels - 1 }});
    // clang-format on
    vec3 prefilteredRadiance = textureLod(prefilterMap, reflectDir, sp.roughness * MAX_REFLECTION_LOD).rgb;
    vec3 irradiance = texture(irradianceMap, sp.tb.normal).rgb;
    vec2 dfg = texture(brdfLut, vec2(sp.NoV, sp.roughness)).rg;

    // Based on Fdez-Agüera, “A Multiple-Scattering Microfacet Model for Real-Time Image Based Lighting.”
    vec3 fresnel
        = sp.f0 + (max(vec3(1. - sp.roughness), sp.f0) - sp.f0) * pow(clamp(1. - sp.NoV, 0., 1.), 5.);

    vec3 baseLayerEnvLight;
    if (energyCompEnabled) {
        vec3 FssEss = fresnel * dfg.x + dfg.y;
        // Multiple scattering
        float Ess = dfg.x + dfg.y;
        float Ems = 1. - Ess;
        vec3 FAvg = sp.f0 + (1. - sp.f0) / 21.;
        vec3 Fms = FssEss * FAvg / (1. - (1. - Ess) * FAvg);
        // Dielectrics
        vec3 Edss = 1. - (FssEss + Fms * Ems);
        vec3 kD = sp.albedo.rgb * Edss;

        baseLayerEnvLight = FssEss * prefilteredRadiance + (Fms * Ems + kD) * irradiance;
    } else {
        // Specular
        vec3 FssEss = fresnel * dfg.x + dfg.y;
        // Diffuse
        vec3 kD = (1.0 - fresnel) * (1. - sp.metalness);

        baseLayerEnvLight = (FssEss * prefilteredRadiance) + (irradiance * sp.albedo.rgb * kD);
    }

#ifdef CLEARCOAT
    if (clearcoatEnabled) {
        vec3 clearcoatIblSpecular = calcClearcoatIBL(sp, baseLayerEnvLight);
        baseLayerEnvLight += clearcoatIblSpecular;
    }
#endif

#ifdef OCCLUSION_MAP
    baseLayerEnvLight *= texture(occlusionTex, vsOut.texCoords).x * occlusionStrength;
#endif

    return baseLayerEnvLight;
}

#ifdef CLEARCOAT
// Formula from:
// https://google.github.io/filament/Filament.html#materialsystem/clearcoatmodel/baselayermodification
// It's derived from Fresnel's formulas
/* void modifyBaseF0(inout vec3 f0, float clearcoatIntensity)
{
    vec3 sqrtF0 = sqrt(f0);
    vec3 numerator = (1. - 5. * sqrtF0);
    vec3 denom = (5. - sqrtF0);

    vec3 newF0 = (numerator * numerator) / (denom * denom);
    // Only modify base f0 if there's actually coating
    f0 = mix(f0, newF0, clearcoatIntensity);
} */
#endif

ShadingParams initShadingParams()
{
    ShadingParams sp;

    // Base color factor is linear RGBA, but base color texture is in sRGB...
    sp.albedo = baseColorFactor;
#ifdef ALBEDO_MAP
    vec4 texalbedo = texture(abledoTex, vsOut.texCoords);
    texalbedo.rgb = pow(texalbedo.rgb, vec3(GAMMA));
    sp.albedo *= texalbedo;
#endif

    sp.viewDir = normalize(camPos.xyz - vsOut.fragPos);

#ifdef NORMAL_MAP
    sp.tb = getNormalFromMap(normalTex, normalScale, sp.viewDir);
#else
    sp.tb.normal = normalize(vsOut.normal);
    sp.tb.tangent = normalize(vsOut.tangent);
    sp.tb.bitangent = normalize(vsOut.bitangent);
#endif

    sp.NoV = max(dot(sp.tb.normal, sp.viewDir), 0.0);

    float linearRoughness = roughnessFactor;
    sp.metalness = metallicFactor;
#ifdef MR_MAP
    linearRoughness *= texture(mrTex, vsOut.texCoords).g;
    sp.metalness *= texture(mrTex, vsOut.texCoords).b;
#endif

    // Disney roughness remapping
    sp.roughness = linearRoughness * linearRoughness;
    // Prevent division by 0
    sp.roughness = clamp(sp.roughness, ROUGHNESS_MIN, 1.0);

    sp.f0 = vec3(DIELECTRIC_FRESNEL);
    sp.f0 = mix(sp.f0, sp.albedo.rgb, sp.metalness);

#ifdef CLEARCOAT
    sp.clearcoatRoughness = clearcoatRoughnessFactor;

#ifdef CLEARCOAT_ROUGHNESS_MAP
    // For some reason the roughness is read from the *green* channel
    sp.clearcoatRoughness *= texture(clearcoatRoughnessTex, vsOut.texCoords).g;
#endif
    sp.clearcoatRoughness = sp.clearcoatRoughness * sp.clearcoatRoughness;
    // Prevent division by 0
    sp.clearcoatRoughness = clamp(sp.clearcoatRoughness, ROUGHNESS_MIN, 1.0);

    sp.clearcoatIntensity = clearcoatIntensityFactor;
#ifdef CLEARCOAT_INTENSITY_MAP
    sp.clearcoatIntensity *= texture(clearcoatIntensityTex, vsOut.texCoords).r;
#endif

#ifdef CLEARCOAT_NORMAL_MAP
    sp.clearcoatNormal = getNormalFromMap(clearcoatNormalTex, clearcoatNormalScale, sp.viewDir).normal;
#else
    // https://github.com/KhronosGroup/glTF/blob/main/extensions/2.0/Khronos/KHR_materials_clearcoat/README.md
    // If clearcoatNormalTexture is not given, no normal mapping is applied to the clear coat layer,
    // even if normal mapping is applied to the base material.
    sp.clearcoatNormal = normalize(vsOut.normal);
#endif
    sp.clearcoatNoV = max(dot(sp.clearcoatNormal, sp.viewDir), 0.0);

#endif

    return sp;
}

void main()
{
    ShadingParams sp = initShadingParams();

    vec3 color = vec3(0.);

    if (IBLEnabled) {
        color += calculateIBL(sp);
    }

    if (directLightEnabled) {
        color += calculateDirectLighting(sp);
    }

#ifdef EMISSIVE_MAP
    vec4 emissive = texture(emissiveTex, vsOut.texCoords);
    emissive.rgb = pow(emissive.rgb, vec3(GAMMA));
    color += emissive.rgb * emissiveFactor.xyz;
#endif

    tonemap(color);

    // gamma correction
    color = pow(color, vec3(1.0 / GAMMA));

    FragColor = vec4(color, sp.albedo.a);
}
