
vec3 fresnelSchlick(vec3 f0, float VoH) { return f0 + (1. - f0) * pow(clamp(1. - VoH, 0.0, 1.0), 5.); }

float fresnelSchlick(float f0, float VoH) { return f0 + (1. - f0) * pow(clamp(1. - VoH, 0.0, 1.0), 5.); }

// roughness is perceptual roughness (roughness squared)
float distributionGgx(float NoH, float roughness)
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

float geometrySmithGgx(float NoV, float NoL, float roughness)
{
    float ggx2 = geometryGgx(NoV, roughness);
    float ggx1 = geometryGgx(NoL, roughness);

    return ggx1 * ggx2;
}

float geometrySmithHeightCorrelatedGgx(float NoV, float NoL, float roughness)
{
    float asq = roughness * roughness;
    float NoVsq = NoV * NoV;
    float NoLsq = NoL * NoL;

    float denoml = sqrt(1 + asq * ((1. / NoLsq) - 1.));
    float denomv = sqrt(1 + asq * ((1. / NoVsq) - 1.));

    return 2 / (denoml + denomv);
}

float visibilitySmithHeightCorrelatedGgx(float NoV, float NoL, float roughness)
{
    float asq = roughness * roughness;
    float NoVsq = NoV * NoV;
    float NoLsq = NoL * NoL;

    float denoml = NoL * sqrt(asq + NoVsq * (1. - asq));
    float denomv = NoV * sqrt(asq + NoLsq * (1. - asq));

    // Protect against division by zero
    return 0.5 / (denoml + denomv + 0.00001);
}

#ifdef ANISOTROPY
// Burley, “Physically-Based Shading at Disney.”
float distributionAnisotropicGgx(
    float roughness, float NoH, vec3 halfway, vec3 tangent, vec3 bitangent, float anisotropy)
{
    // Remapping from: Kulla and Conty, “Revisiting Physically Based Shading at Imageworks.”
    float tRoughness = max(roughness * (1. + anisotropy), ROUGHNESS_MIN);
    float bRoughness = max(roughness * (1. - anisotropy), ROUGHNESS_MIN);

    float ToH = dot(tangent, halfway);
    float BoH = dot(bitangent, halfway);

    float denom
        = ((ToH * ToH) / (tRoughness * tRoughness)) + ((BoH * BoH) / (bRoughness * bRoughness)) + (NoH * NoH);

    denom = denom * denom;

    return 1. / (PI * tRoughness * bRoughness * denom);
}

// Taken from: Guy and Agopian, “Physically Based Rendering in Filament.”
float visibilitySmithHeightCorrelatedGgxAniso(
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

const uint DIFFUSE_TYPE_LAMBERT = 0;
const uint DIFFUSE_TYPE_FROSTBITE = 1;
const uint DIFFUSE_TYPE_CODWWII = 2;

// For diffuse BRDFs, divide by PI is done later

vec3 diffuseLambert(vec3 albedo) { return albedo; }

float disneyFresnelSchlick(float fd90, float NoX) { return 1. + (fd90 - 1.) * pow(1. - NoX, 5.); }

// Moving Frostbite to Physically Based Rendering 3.0
vec3 diffuseFrostbite(vec3 albedo, float roughness, float NoL, float LoH, float NoV)
{
    float energyBias = mix(0., 0.5, roughness);
    float energyFactor = mix(1., 1. / 1.51, roughness);
    float fd90 = energyBias + 2. * LoH * LoH * roughness;

    float lightScatter = disneyFresnelSchlick(fd90, NoL);
    float viewScatter = disneyFresnelSchlick(fd90, NoV);

    return albedo * lightScatter * viewScatter * energyFactor;
}

// Material advances in Call of Duty: WWII
vec3 diffuseCodWWII(vec3 albedo, float roughness, float NoL, float LoH, float NoH, float NoV)
{
    float f0 = LoH + pow(1. - LoH, 5.);
    float f1 = (1. - 0.75 * pow(1. - NoL, 5.)) * (1. - 0.75 * pow(1. - NoV, 5.));

    // convert roughness to gloss
    // they use gloss with a special parametrization instead of roughness
    float asq = roughness * roughness;
    float g = log2((2. / asq) - 1.) / 18.;

    float t = clamp(2.2 * g - 0.5, 0., 1.);
    float fd = f0 + (f1 - f0) * t;

    float gsq = g * g;
    float expo = -sqrt(NoH) * max(73.2 * g - 21.2, 8.9);
    float fb = (34.5 * gsq - 59 * g + 24.5) * LoH * pow(2., expo);

    return albedo * (fd + fb);
}
