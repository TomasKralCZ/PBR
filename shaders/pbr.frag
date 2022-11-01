#version 460 core

//@DEFINES@

in VsOut
{
    vec2 texCoords;
    vec3 normal;
    vec3 fragPos;
}
vsOut;

layout(std140, binding = 1) uniform PbrMaterial
{
    uniform vec4 base_color_factor;
    uniform vec4 emissive_factor;
    uniform float metallic_factor;
    uniform float roughness_factor;
    uniform float normal_scale;
    uniform float occlusion_strength;

    uniform float clearcoatIntensityFactor;
    uniform float clearcoatRoughnessFactor;
};

#ifdef ALBEDO_MAP
layout(binding = 0) uniform sampler2D abledoTex;
#endif
#ifdef MR_MAP
layout(binding = 1) uniform sampler2D mrTex;
#endif
#ifdef NORMAL_MAP
layout(binding = 2) uniform sampler2D normalTex;
#endif
#ifdef OCCLUSION_MAP
layout(binding = 3) uniform sampler2D occlusionTex;
#endif
#ifdef EMISSIVE_MAP
layout(binding = 4) uniform sampler2D emissiveTex;
#endif

#ifdef CLEARCOAT_INTENSITY_MAP
layout(binding = 5) uniform sampler2D clearcoatIntensityTex;
#endif
#ifdef CLEARCOAT_ROUGHNESS_MAP
layout(binding = 6) uniform sampler2D clearcoatRoughnessTex;
#endif

layout(std140, binding = 2) uniform Lighting
{
    uniform vec4 lightPositions[4];
    uniform vec4 lightColors[4];
    uniform vec4 camPos;
    uniform uint lights;
};

layout(binding = 7) uniform samplerCube irradianceMap;
layout(binding = 8) uniform samplerCube prefilterMap;
layout(binding = 9) uniform sampler2D brdfLut;

out vec4 FragColor;

const float PI = 3.14159265359;
const float ROUGHNESS_MIN = 0.0001;

const float GAMMA = 2.2;

const float DIELECTRIC_FRESNEL = 0.04;

// Parameters that stay same for the whole pixel
struct ShadingParams {
    vec4 albedo;

    vec3 viewDir;
    vec3 normal;
    // viewDir * normal (dot product)
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

// TODO: better normal mapping
#ifdef NORMAL_MAP
vec3 getNormalFromMap()
{
    vec3 tangentNormal = (normal_scale * texture(normalTex, vsOut.texCoords).xyz) * 2.0 - 1.0;

    vec3 Q1 = dFdx(vsOut.fragPos);
    vec3 Q2 = dFdy(vsOut.fragPos);
    vec2 st1 = dFdx(vsOut.texCoords);
    vec2 st2 = dFdy(vsOut.texCoords);

    vec3 N = normalize(vsOut.normal);
    vec3 T = normalize(Q1 * st2.t - Q2 * st1.t);
    vec3 B = -normalize(cross(N, T));
    mat3 TBN = mat3(T, B, N);

    return normalize(TBN * tangentNormal);
}
#endif

vec3 fresnelSchlick(vec3 f0, float VoH)
{
    return f0 + (1. - f0) * pow(clamp(1. - VoH, 0.0, 1.0), 5.);
}

float fresnelSchlick(float f0, float VoH)
{
    return f0 + (1. - f0) * pow(clamp(1. - VoH, 0.0, 1.0), 5.);
}

vec3 fresnelSchlickRoughness(float VoH, vec3 f0, float roughness)
{
    return f0 + (max(vec3(1.0 - roughness), f0) - f0) * pow(clamp(1.0 - VoH, 0.0, 1.0), 5.0);
}

// GGX / Trowbridge-Reitz
float normalDistribution(float NoH, float roughness)
{
    float asq = roughness * roughness;
    float denom = (NoH * NoH) * (asq - 1.) + 1.;

    return (asq) / (PI * denom * denom);
}

float geometry_ggx(float ndv, float roughness)
{
    float asq = roughness * roughness;

    float denom = ndv + sqrt(asq + ((1 - asq) * (ndv * ndv)));

    return (2 * ndv) / denom;
}

// Smith
float geometryShadowing(float NoV, float NoL, float roughness)
{
    float ggx2 = geometry_ggx(NoV, roughness);
    float ggx1 = geometry_ggx(NoL, roughness);

    return ggx1 * ggx2;
}

#ifdef CLEARCOAT
vec3 clearcoatBrdf(ShadingParams sp, out float fresnel, vec3 halfway,
    vec3 lightDir, float VoH)
{
    float clearcoatNoH = max(dot(halfway, sp.clearcoatNormal), 0.0);
    float clearcoatNoL = max(dot(lightDir, sp.clearcoatNormal), 0.0);

    // clearcoat BRDF
    float normalDistribution = normalDistribution(clearcoatNoH, sp.clearcoatRoughness);
    float geometry = geometryShadowing(sp.clearcoatNoV, clearcoatNoL, sp.clearcoatRoughness);
    // Coating IOR is 1.5 (f0 == 0.04) - equivalent to polyurethane
    fresnel = fresnelSchlick(DIELECTRIC_FRESNEL, VoH) * sp.clearcoatIntensity;

    vec3 numerator = normalDistribution * geometry * vec3(fresnel);
    float denominator = 4.0 * sp.clearcoatNoV * clearcoatNoL;
    // + 0.0001 to prevent divide by zero
    vec3 specular = numerator / (denominator + 0.0001);

    return specular;
}
#endif

vec3 calculateDirectLighting(ShadingParams sp)
{
    vec3 totalRadiance = vec3(0.);

    for (int i = 0; i < lights; i++) {
        vec3 lightDir = normalize(lightPositions[i].xyz - vsOut.fragPos);
        vec3 halfway = normalize(sp.viewDir + lightDir);
        float VoH = max(dot(halfway, sp.viewDir), 0.);
        float NoH = max(dot(sp.normal, halfway), 0.0);
        float NoL = max(dot(sp.normal, lightDir), 0.0);

        // TODO: should add attenuation...
        vec3 radiance = lightColors[i].xyz;

        // Cook-Torrance BRDF
        float normalDistribution = normalDistribution(NoH, sp.roughness);
        float geometry = geometryShadowing(sp.NoV, NoL, sp.roughness);
        vec3 fresnel = fresnelSchlick(sp.f0, VoH);

#ifdef CLEARCOAT
        float clearcoatFresnel;
        vec3 clearcoatColor = clearcoatBrdf(sp, clearcoatFresnel, halfway, lightDir, VoH);
#endif

        vec3 numerator = normalDistribution * geometry * fresnel;
        float denominator = 4.0 * sp.NoV * NoL;
        // + 0.0001 to prevent divide by zero
        vec3 specular = numerator / (denominator + 0.0001);

        // diffuse is 1 - fresnel (the amount of reflected light)
        vec3 kd = vec3(1.0) - fresnel;
        // metals have no diffuse, attenuate for in-between materials
        kd *= 1.0 - sp.metalness;

        // Lambertian diffuse
        vec3 diffuse = kd * sp.albedo.rgb / PI;

#ifdef CLEARCOAT
        // Energy loss due to the clearcoat layer is given by 1 - clearcoatFresnel
        vec3 brdf = (diffuse + specular) * (1. - clearcoatFresnel) + clearcoatColor;
#else
        vec3 brdf = diffuse + specular;
#endif

        totalRadiance += brdf * radiance * NoL;
    }

    return totalRadiance;
}

vec3 calculateIBL(ShadingParams sp)
{
    vec3 F = fresnelSchlickRoughness(sp.NoV, sp.f0, sp.roughness);
    vec3 kD = 1.0 - F;
    kD *= 1.0 - sp.metalness;

    vec3 irradiance = texture(irradianceMap, sp.normal).rgb;
    vec3 iblDiffuse = irradiance * sp.albedo.rgb;

    vec3 R = reflect(-sp.viewDir, sp.normal);

    const float MAX_REFLECTION_LOD = 6.0;
    vec3 prefilteredLight = textureLod(prefilterMap, R, sp.roughness * MAX_REFLECTION_LOD).rgb;
    vec2 dfg = texture(brdfLut, vec2(sp.NoV, sp.roughness)).rg;
    vec3 iblSpecular = prefilteredLight * (F * dfg.x + dfg.y);

#ifdef CLEARCOAT
    // https://google.github.io/filament/Filament.html#lighting/imagebasedlights/clearcoat
    float clearcoatFresnel = fresnelSchlick(DIELECTRIC_FRESNEL, sp.NoV) * sp.clearcoatIntensity;

    // Apply clearcoat IBL
    vec3 clearcoatPrefilteredLight = textureLod(prefilterMap, R, sp.clearcoatRoughness * MAX_REFLECTION_LOD).rgb;
    vec2 clearcoatDfg = texture(brdfLut, vec2(sp.NoV, sp.clearcoatRoughness)).rg;
    vec3 clearcoatIblSpecular = clearcoatPrefilteredLight * (clearcoatFresnel * clearcoatDfg.x + clearcoatDfg.y);

    // base layer attenuation for energy compensation
    iblDiffuse *= 1.0 - clearcoatFresnel;
    iblSpecular *= 1.0 - clearcoatFresnel;

    vec3 ambient = (kD * iblDiffuse + iblSpecular) + clearcoatIblSpecular;
#else
    vec3 ambient = (kD * iblDiffuse + iblSpecular);
#endif

#ifdef OCCLUSION_MAP
    ambient *= texture(occlusionTex, vsOut.texCoords).x * occlusion_strength;
#endif

    return ambient;
}

void initShadingParams(inout ShadingParams sp)
{
    sp.albedo = base_color_factor;
#ifdef ALBEDO_MAP
    sp.albedo *= texture(abledoTex, vsOut.texCoords);
#endif

    sp.albedo.rgb = pow(sp.albedo.rgb, vec3(GAMMA));

    sp.viewDir = normalize(camPos.xyz - vsOut.fragPos);

#ifdef NORMAL_MAP
    sp.normal = getNormalFromMap();
#else
    sp.normal = normalize(vsOut.normal);
#endif

    sp.NoV = max(dot(sp.normal, sp.viewDir), 0.0);

    // Disney roughness remapping
    float perceptualRoughness = roughness_factor;
    sp.metalness = metallic_factor;
#ifdef MR_MAP
    perceptualRoughness *= texture(mrTex, vsOut.texCoords).g;
    sp.metalness *= texture(mrTex, vsOut.texCoords).b;
#endif

    // Prevent division by 0
    sp.roughness = perceptualRoughness * perceptualRoughness;
    sp.roughness = clamp(sp.roughness, ROUGHNESS_MIN, 1.0);

    sp.f0 = vec3(DIELECTRIC_FRESNEL);
    sp.f0 = mix(sp.f0, sp.albedo.rgb, sp.metalness);

#ifdef CLEARCOAT
    sp.clearcoatRoughness = clearcoatRoughnessFactor;

#ifdef CLEARCOAT_ROUGHNESS_MAP
    sp.clearcoatRoughness *= texture(clearcoatRoughnessTex, vsOut.texCoords).x;
#endif
    // Disney remapping
    sp.clearcoatRoughness = sp.clearcoatRoughness * sp.clearcoatRoughness;
    // Prevent division by 0
    sp.clearcoatRoughness = clamp(sp.clearcoatRoughness, ROUGHNESS_MIN, 1.0);

    sp.clearcoatIntensity = clearcoatIntensityFactor;
#ifdef CLEARCOAT_INTENSITY_MAP
    sp.clearcoatIntensity *= texture(clearcoatIntensityTex, vsOut.texCoords).x;
#endif

    // TODO: separate clearcoat normal map
    sp.clearcoatNormal = sp.normal;
    sp.clearcoatNoV = max(dot(sp.clearcoatNormal, sp.viewDir), 0.0);
#endif
}

void main()
{
    ShadingParams sp;
    initShadingParams(sp);

    vec3 directLight = calculateDirectLighting(sp);
    vec3 environmentLight = calculateIBL(sp);

    vec3 color = environmentLight + directLight;

#ifdef EMISSIVE_MAP
    vec4 emissive = texture(emissiveTex, vsOut.texCoords);
    emissive.rgb = pow(emissive.rgb, vec3(GAMMA));
    color += emissive.rgb * emissive_factor.xyz;
#endif

    // Reinhard tonemapping
    color = color / (color + vec3(1.0));
    // gamma correction
    color = pow(color, vec3(1.0 / GAMMA));

    FragColor = vec4(color, 1.0);
}
