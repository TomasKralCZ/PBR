
#define TONEMAP_REINHARD 0
#define TONEMAP_UNCHARTED 1

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

void tonemap(inout vec3 color)
{
#if TONEMAP_OPERATOR == TONEMAP_REINHARD
    reinhard(color);
#elif TONEMAP_OPERATOR == TONEMAP_UNCHARTED
    uncharted2Filmic(color);
#endif
}
