#version 420 core

in VsOut {
    vec2 texCoords;
    vec3 normal;
    vec3 fragPos;
} vsOut;

layout (std140, binding = 5) uniform Lighting {
    uniform vec4 lightPositions[4];
    uniform vec4 lightColors[4];
    uniform vec4 camPos;
    uniform uint lights;
};

layout(binding = 0) uniform sampler2D abledoTex;
layout(binding = 1) uniform sampler2D mrTex;
layout(binding = 2) uniform sampler2D normalTex;
layout(binding = 3) uniform sampler2D occlusionTex;
layout(binding = 4) uniform sampler2D emissiveTex;

out vec4 FragColor;

const float PI = 3.14159265359;

// ----------------------------------------------------------------------------
// Easy trick to get tangent-normals to world-space to keep PBR code simplified.
// Don't worry if you don't get what's going on; you generally want to do normal
// mapping the usual way for performance anways; I do plan make a note of this
// technique somewhere later in the normal mapping tutorial.
vec3 getNormalFromMap()
{
    vec3 tangentNormal = texture(normalTex, vsOut.texCoords).xyz * 2.0 - 1.0;

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

vec3 fresnelSchlick(vec3 f0, float cosTheta) {
    return f0 + (1. - f0) * pow(clamp(1. - cosTheta, 0.0, 1.0), 5.);
}

// GGX / Trowbridge-Reitz
float normalDistribution(vec3 normal, vec3 halfway, float roughness) {
    float a = roughness * roughness;
    float asq = a * a;

    float nh = max(dot(normal, halfway), 0.);
    float denom = (nh * nh) * (asq - 1.) + 1.;

    return (asq) / (PI * denom * denom);
}

// Schlick-Beckmann in UE4 (http://graphicrants.blogspot.com/2013/08/specular-brdf-reference.html)
float geometrySchlick(float dotProd, float roughness) {
    float r = (roughness + 1.);
    float k = (r * r) / 8.;

    return dotProd / (dotProd * (1. - k) + k);
}

// Smith
float geometryShadowing(vec3 normal, vec3 viewDir, vec3 lightDir, float roughness) {
    float nv = max(dot(normal, viewDir), 0.0);
    float nl = max(dot(normal, lightDir), 0.0);
    float ggx2 = geometrySchlick(nv, roughness);
    float ggx1 = geometrySchlick(nl, roughness);

    return ggx1 * ggx2;
}

void main() {
    float gamma = 2.2;

    // Texture sampling
    vec4 albedo = texture(abledoTex, vsOut.texCoords);
    albedo.rgb = pow(albedo.rgb, vec3(gamma));

    vec4 emissive = texture(emissiveTex, vsOut.texCoords) * 0.2;
    emissive.rgb = pow(emissive.rgb, vec3(gamma));

    float roughness = texture(mrTex, vsOut.texCoords).g;
    float metalness = texture(mrTex, vsOut.texCoords).b;

    // Per-fragment
    vec3 f0 = vec3(0.04);
    f0 = mix(f0, albedo.rgb, metalness);

    vec3 viewDir = normalize(camPos.xyz - vsOut.fragPos);
    //vec3 normal = normalize(vsOut.normal);
    vec3 normal = getNormalFromMap();

    // Per-light radiance
    vec3 totalRadiance = vec3(0.);
    for (int i = 0; i < lights; i++) {
        vec3 lightDir = normalize(lightPositions[i].xyz - vsOut.fragPos);
        vec3 halfway = normalize(viewDir + lightDir);
        float cosTheta = max(dot(halfway, viewDir), 0.);

        // Should add attenuation...
        vec3 radiance = lightColors[i].xyz;

        // Cook-Torrance BRDF
        float normalDistribution = normalDistribution(normal, halfway, roughness);
        float geometry = geometryShadowing(normal, viewDir, lightDir, roughness);
        vec3 fresnel = fresnelSchlick(f0, cosTheta);

        vec3 numerator = normalDistribution * geometry * fresnel;
        float denominator = 4.0 * max(dot(viewDir, normal), 0.0) * max(dot(lightDir, normal), 0.0);
        // + 0.0001 to prevent divide by zero
        vec3 specular = numerator / (denominator + 0.0001);

        vec3 kS = fresnel;
        // for energy conservation, the diffuse and specular light can't
        // be above 1.0 (unless the surface emits light); to preserve this
        // relationship the diffuse component (kD) should equal 1.0 - kS.
        vec3 kD = vec3(1.0) - kS;
        // multiply kD by the inverse metalness such that only non-metals
        // have diffuse lighting, or a linear blend if partly metal (pure metals
        // have no diffuse light).
        kD *= 1.0 - metalness;

        // scale light by NdotL
        float nl = max(dot(normal, lightDir), 0.0);

        // add to outgoing radiance Lo
        totalRadiance += ((kD * albedo.rgb / PI) + specular) * radiance * nl;
    }

    // ambient
    vec4 ambient = albedo * 0.05 * texture(occlusionTex, vsOut.texCoords).x;

    vec3 color = emissive.rgb + ambient.rgb + totalRadiance;

    // HDR tonemapping
    color = color / (color + vec3(1.0));
    color = pow(color, vec3(1.0 / gamma));

    FragColor = vec4(color, 1.0);
}
