
#define TONEMAP_REINHARD 0
#define TONEMAP_ASEC 1
#define TONEMAP_UNCHARTED 2

#define TONEMAP_OPERATOR TONEMAP_UNCHARTED

// This code is based on: https://64.github.io/tonemapping/

void reinhard(inout vec3 color) { color = color / (color + 1.0); }

vec3 uncharted2TonemapPartial(vec3 x)
{
    float A = 0.15f;
    float B = 0.50f;
    float C = 0.10f;
    float D = 0.20f;
    float E = 0.02f;
    float F = 0.30f;
    return ((x * (A * x + C * B) + D * E) / (x * (A * x + B) + D * F)) - E / F;
}

// https://gdcvault.com/play/1012351/Uncharted-2-HDR
void uncharted2Filmic(inout vec3 v)
{
    float exposureBias = 2.0f;
    vec3 curr = uncharted2TonemapPartial(v * exposureBias);

    vec3 w = vec3(11.2f);
    vec3 whiteScale = vec3(1.0f) / uncharted2TonemapPartial(w);

    v = curr * whiteScale;
}

// sRGB => XYZ => D65_2_D60 => AP1 => RRT_SAT
const mat3 ACESInputMat
    = { { 0.59719, 0.35458, 0.04823 }, { 0.07600, 0.90834, 0.01566 }, { 0.02840, 0.13383, 0.83777 } };

// ODT_SAT => XYZ => D60_2_D65 => sRGB
const mat3 ACESOutputMat
    = { { 1.60475, -0.53108, -0.07367 }, { -0.10208, 1.10813, -0.00605 }, { -0.00327, -0.07276, 1.07602 } };

vec3 RRTAndODTFit(vec3 v)
{
    vec3 a = v * (v + 0.0245786f) - 0.000090537f;
    vec3 b = v * (0.983729f * v + 0.4329510f) + 0.238081f;
    return a / b;
}

void ACESFitted(inout vec3 color)
{
    // color = ACESInputMat * color;
    color = color * ACESInputMat;

    // Apply RRT and ODT
    color = RRTAndODTFit(color);

    // color = ACESOutputMat * color;
    color = color * ACESOutputMat;

    // Clamp to [0, 1]
    color = clamp(color, 0.0, 1.0);
}

void tonemap(inout vec3 color)
{
#if TONEMAP_OPERATOR == TONEMAP_REINHARD
    reinhard(color);
#elif TONEMAP_OPERATOR == TONEMAP_UNCHARTED
    uncharted2Filmic(color);
#elif TONEMAP_OPERATOR == TONEMAP_ASEC
    ACESFitted(color);
#endif
}
