#version 460 core

//#defines

// {% include "consts.glsl" %}

// {% include "structs/pbrVsOut.glsl" %}
// {% include "structs/pbrMaterial.glsl" %}
// {% include "structs/pbrTextures.glsl" %}
// {% include "structs/lighting.glsl" %}
// {% include "structs/settings.glsl" %}
// {% include "structs/brdf_bufs.glsl" %}

// {% include "tools/tonemap.glsl" %}
// {% include "tools/normal_map.glsl" %}

#ifdef MERL_BRDF
// {% include "measured_brdf/brdf_merl.glsl" %}
#endif

#ifdef MIT_BRDF
// {% include "measured_brdf/brdf_mit.glsl" %}
#endif

#ifdef UTIA_BRDF
// {% include "measured_brdf/brdf_utia.glsl" %}
#endif

out vec4 FragColor;

// Parameters that stay same for the whole pixel
struct ShadingParams {
    vec3 viewDir;
    vec3 normal;
    float NoV;
};

vec3 calculateDirectLighting(ShadingParams sp)
{
    vec3 totalRadiance = vec3(0.);

    for (int i = 0; i < lights; i++) {
        // TODO: should add attenuation...
        vec3 radiance = lightColors[i].xyz;

        vec3 lightDir = normalize(lightPositions[i].xyz - vsOut.fragPos);

        // FIXME: check NoV > 0 and NoL > 0

#ifdef MERL_BRDF
        vec3 brdf = lookup_brdf_merl(lightDir, sp.viewDir, sp.normal, vsOut.tangent, vsOut.bitangent);
#endif

#ifdef MIT_BRDF
        vec3 brdf = lookup_brdf_mit(lightDir, sp.viewDir, sp.normal, vsOut.tangent, vsOut.bitangent);
#endif

#ifdef UTIA_BRDF
        vec3 brdf = lookup_brdf_utia(lightDir, sp.viewDir, sp.normal, vsOut.tangent, vsOut.bitangent);
#endif

        totalRadiance += radiance * brdf;
    }

    return totalRadiance;
}

vec3 calculateIBL(ShadingParams sp)
{
    vec3 reflectDir = reflect(-sp.viewDir, sp.normal);
    return texture(rawBrdfMap, reflectDir).rgb;
}

ShadingParams initShadingParams()
{
    ShadingParams sp;

    sp.viewDir = normalize(camPos.xyz - vsOut.fragPos);

#ifdef NORMAL_MAP
    sp.normal = getNormalFromMap(normalTex, normalScale, sp.viewDir);
#else
    sp.normal = normalize(vsOut.normal);
#endif

    sp.NoV = dot(sp.normal, sp.viewDir);

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

#ifdef OCCLUSION_MAP
    color *= texture(occlusionTex, vsOut.texCoords).x * occlusionStrength;
#endif

    tonemap(color);

    // gamma correction
    color = pow(color, vec3(1.0 / GAMMA));

    FragColor = vec4(color, 1.0);
}
