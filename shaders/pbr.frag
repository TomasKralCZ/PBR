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

layout(binding = 7) uniform samplerCube irradiance_map;
layout(binding = 8) uniform samplerCube prefilter_map;
layout(binding = 9) uniform sampler2D brdf_lut;

out vec4 FragColor;

const float PI = 3.14159265359;
const float ROUGHNESS_MIN = 0.0001;

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

vec3 fresnelSchlick(vec3 f0, float cosTheta)
{
    return f0 + (1. - f0) * pow(clamp(1. - cosTheta, 0.0, 1.0), 5.);
}

float fresnelSchlick(float f0, float cosTheta)
{
    return f0 + (1. - f0) * pow(clamp(1. - cosTheta, 0.0, 1.0), 5.);
}

// GGX / Trowbridge-Reitz
float normalDistribution(vec3 normal, vec3 halfway, float roughness)
{
    float a = roughness * roughness;
    float asq = a * a;

    float nh = max(dot(normal, halfway), 0.);
    float denom = (nh * nh) * (asq - 1.) + 1.;

    return (asq) / (PI * denom * denom);
}

vec3 fresnelSchlickRoughness(float cosTheta, vec3 F0, float roughness)
{
    return F0 + (max(vec3(1.0 - roughness), F0) - F0) * pow(clamp(1.0 - cosTheta, 0.0, 1.0), 5.0);
}

float geometry_ggx(float ndv, float roughness)
{
    float a = roughness * roughness;
    float asq = a * a;

    float denom = ndv + sqrt(asq + ((1 - asq) * (ndv * ndv)));

    return (2 * ndv) / denom;
}

// Smith
float geometryShadowing(vec3 normal, vec3 viewDir, vec3 lightDir,
    float roughness)
{
    float nv = max(dot(normal, viewDir), 0.0);
    float nl = max(dot(normal, lightDir), 0.0);

    float ggx2 = geometry_ggx(nv, roughness);
    float ggx1 = geometry_ggx(nl, roughness);

    return ggx1 * ggx2;
}

#ifdef CLEARCOAT
vec3 clearcoatBrdf(out float clearcoatFresnel, vec3 normal, vec3 halfway,
    vec3 viewDir, vec3 lightDir, float cosTheta)
{
    float clearcoatRoughness = clearcoatIntensityFactor;

#ifdef CLEARCOAT_ROUGHNESS_MAP
    clearcoatRoughness *= texture(clearcoatRoughnessTex, vsOut.texCoords).x;
#endif

    clearcoatRoughness = clearcoatIntensityFactor * clearcoatRoughnessFactor;
    clearcoatRoughness = clamp(clearcoatRoughness, ROUGHNESS_MIN, 1.0);

    float clearcoatIntensity = clearcoatIntensityFactor;

#ifdef CLEARCOAT_INTENSITY_MAP
    clearcoatIntensity *= texture(clearcoatIntensityTex, vsOut.texCoords).x;
#endif

    // clearcoat BRDF
    float clearcoatDistribution = normalDistribution(normal, halfway, clearcoatRoughness);
    float clearcoatGeometry = geometryShadowing(normal, viewDir, lightDir, clearcoatRoughness);
    // Coating IOR is 1.5 (f0 == 0.04) - equivalent to polyurethane
    clearcoatFresnel = fresnelSchlick(0.04, cosTheta) * clearcoatIntensity;

    vec3 numerator = clearcoatDistribution * clearcoatGeometry * vec3(clearcoatFresnel);
    float denominator = 4.0 * max(dot(viewDir, normal), 0.0) * max(dot(lightDir, normal), 0.0);
    // + 0.0001 to prevent divide by zero
    vec3 specular = numerator / (denominator + 0.0001);

    return specular;
}
#endif

void main()
{
    float gamma = 2.2;

    vec4 albedo = base_color_factor;
#ifdef ALBEDO_MAP
    albedo *= texture(abledoTex, vsOut.texCoords);
#endif

    albedo.rgb = pow(albedo.rgb, vec3(gamma));

    float roughness = roughness_factor;
    float metalness = metallic_factor;
#ifdef MR_MAP
    roughness *= texture(mrTex, vsOut.texCoords).g;
    metalness *= texture(mrTex, vsOut.texCoords).b;
#endif

    roughness = clamp(roughness, ROUGHNESS_MIN, 1.0);

    vec3 f0 = vec3(0.04);
    f0 = mix(f0, albedo.rgb, metalness);

    vec3 viewDir = normalize(camPos.xyz - vsOut.fragPos);
#ifdef NORMAL_MAP
    vec3 normal = getNormalFromMap();
#else
    vec3 normal = normalize(vsOut.normal);
#endif

    vec3 totalRadiance = vec3(0.);
    for (int i = 0; i < lights; i++) {
        vec3 lightDir = normalize(lightPositions[i].xyz - vsOut.fragPos);
        vec3 halfway = normalize(viewDir + lightDir);
        float cosTheta = max(dot(halfway, viewDir), 0.);

        // TODO: should add attenuation...
        vec3 radiance = lightColors[i].xyz;

        // Cook-Torrance BRDF
        float normalDistribution = normalDistribution(normal, halfway, roughness);
        float geometry = geometryShadowing(normal, viewDir, lightDir, roughness);
        vec3 fresnel = fresnelSchlick(f0, cosTheta);

#ifdef CLEARCOAT
        float clearcoatFresnel;
        vec3 clearcoatColor = clearcoatBrdf(clearcoatFresnel, normal, halfway, viewDir, lightDir, cosTheta);
#endif

        vec3 numerator = normalDistribution * geometry * fresnel;
        float denominator = 4.0 * max(dot(viewDir, normal), 0.0) * max(dot(lightDir, normal), 0.0);
        // + 0.0001 to prevent divide by zero
        vec3 specular = numerator / (denominator + 0.0001);

        vec3 kD = vec3(1.0) - fresnel;
        kD *= 1.0 - metalness;

        vec3 diffuse = kD * albedo.rgb / PI;
        float nl = max(dot(normal, lightDir), 0.0);

#ifdef CLEARCOAT
        // Energy loss due to the clearcoat layer is given by 1 - clearcoatFresnel
        vec3 brdf = (diffuse + specular) * (1. - clearcoatFresnel) + clearcoatColor;
#else
        vec3 brdf = diffuse + specular;
#endif

        totalRadiance += brdf * radiance * nl;
    }

    // environment lighting
    vec3 F = fresnelSchlickRoughness(max(dot(normal, viewDir), 0.0), f0, roughness);
    vec3 kS = F;
    vec3 kD = 1.0 - kS;
    kD *= 1.0 - metalness;

    vec3 irradiance = texture(irradiance_map, normal).rgb;
    vec3 diffuse = irradiance * albedo.rgb;

    vec3 R = reflect(-viewDir, normal);

    const float MAX_REFLECTION_LOD = 6.0;
    vec3 prefiltered_color = textureLod(prefilter_map, R, roughness * MAX_REFLECTION_LOD).rgb;
    vec2 env_brdf = texture(brdf_lut, vec2(max(dot(normal, viewDir), 0.0), roughness)).rg;
    vec3 specular = prefiltered_color * (F * env_brdf.x + env_brdf.y);

    vec3 ambient = (kD * diffuse + specular);

#ifdef OCCLUSION_MAP
    ambient *= texture(occlusionTex, vsOut.texCoords).x * occlusion_strength;
#endif

    vec3 color = ambient + totalRadiance;

#ifdef EMISSIVE_MAP
    vec4 emissive = texture(emissiveTex, vsOut.texCoords);
    emissive.rgb = pow(emissive.rgb, vec3(gamma));
    color += emissive.rgb * emissive_factor.xyz;
#endif

    // HDR tonemapping
    color = color / (color + vec3(1.0));
    color = pow(color, vec3(1.0 / gamma));

    FragColor = vec4(color, 1.0);
}
