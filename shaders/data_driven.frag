#version 460 core

//#defines

//#import shaders/consts.glsl
//#import shaders/tools/tonemap.glsl
//#import shaders/tools/brdf_db.glsl

in VsOut
{
    vec2 texCoords;
    vec3 fragPos;
    vec3 normal;
    vec3 tangent;
    vec3 bitangent;
}
vsOut;

out vec4 FragColor;

// Only normal / occlusion is used for this shader, the rest are irrelevant
layout(std140, binding = 1) uniform PbrMaterial
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

#ifdef NORMAL_MAP
layout(binding = 2) uniform sampler2D normalTex;
#endif
#ifdef OCCLUSION_MAP
layout(binding = 3) uniform sampler2D occlusionTex;
#endif

layout(binding = 8) uniform samplerCube irradianceMap;
layout(binding = 9) uniform samplerCube prefilterMap;
layout(binding = 10) uniform sampler2D brdfLut;
layout(binding = 11) uniform samplerCube rawBrdfMap;

layout(std140, binding = 2) uniform Lighting
{
    uniform vec4 lightPositions[4];
    uniform vec4 lightColors[4];
    uniform vec4 camPos;
    uniform uint lights;
};

layout(std140, binding = 3) uniform Settings
{
    uniform bool clearcoatEnabled;
    uniform bool directLightEnabled;
    uniform bool IBLEnabled;
};

// Parameters that stay same for the whole pixel
struct ShadingParams {
    vec3 viewDir;
    vec3 normal;
    float NoV;
};

#if defined(NORMAL_MAP)
vec3 getNormalFromMap(sampler2D tex, float scaleNormal, vec3 viewDir)
{
    // https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#_material_normaltextureinfo_scale
    vec3 tangentNormal
        = normalize((texture(tex, vsOut.texCoords).xyz) * 2.0 - 1.0) * vec3(scaleNormal, scaleNormal, 1.0);

    mat3 tbn = mat3(normalize(vsOut.tangent), normalize(vsOut.bitangent), normalize(vsOut.normal));

    return normalize(tbn * tangentNormal);
}
#endif

vec3 calculateDirectLighting(ShadingParams sp)
{
    vec3 totalRadiance = vec3(0.);

    for (int i = 0; i < lights; i++) {
        // TODO: should add attenuation...
        vec3 radiance = lightColors[i].xyz;

        vec3 lightDir = normalize(lightPositions[i].xyz - vsOut.fragPos);
        vec3 halfway = normalize(sp.viewDir + lightDir);

        float NoL = dot(sp.normal, lightDir);

        if (NoL > 0.0 && sp.NoV > 0.0) {
            // TODO(high): fix angles calculation
            float thetaIn = acos(NoL);
            float thetaOut = acos(sp.NoV);

            vec3 projectedLightDir = normalize(lightDir - (dot(sp.normal, lightDir) * sp.normal));
            vec3 projectedViewDir = normalize(sp.viewDir - (dot(sp.normal, sp.viewDir) * sp.normal));

            float phiIn = acos(clamp(dot(normalize(vsOut.tangent), projectedLightDir), -1.0, 1.0));
            float phiOut = acos(clamp(dot(normalize(vsOut.tangent), projectedViewDir), -1.0, 1.0));

            // vec3 brdf1 = lookup_brdf(thetaIn, phiIn, thetaOut, phiOut);
            vec3 brdf2 = lookup_brdf(lightDir, sp.viewDir, sp.normal, vsOut.tangent, vsOut.bitangent);

            totalRadiance += radiance * brdf2;
        }
    }

    return totalRadiance;
}

vec3 calculateIBL(ShadingParams sp)
{
    // vec3 irradiance = texture(irradianceMap, sp.normal).rgb;

    vec3 reflectDir = reflect(-sp.viewDir, sp.normal);
    return texture(rawBrdfMap, reflectDir).rgb;
}

void initShadingParams(inout ShadingParams sp)
{
    sp.viewDir = normalize(camPos.xyz - vsOut.fragPos);

#ifdef NORMAL_MAP
    sp.normal = getNormalFromMap(normalTex, normalScale, sp.viewDir);
#else
    sp.normal = normalize(vsOut.normal);
#endif

    sp.NoV = dot(sp.normal, sp.viewDir);
}

void main()
{
    ShadingParams sp;
    initShadingParams(sp);

    vec3 color = vec3(0.);

    if (IBLEnabled) {
        color += calculateIBL(sp);
    }

    if (directLightEnabled) {
        color += calculateDirectLighting(sp);
    }

#ifdef OCCLUSION_MAP
    color *= texture(occlusionTex, vsOut.texCoords).x * occlusionStrength;
#endif

    tonemap(color);

    // gamma correction
    color = pow(color, vec3(1.0 / GAMMA));

    FragColor = vec4(color, 1.0);
}
