#version 460 core

in VsOut {
    vec2 texCoords;
    vec3 normal;
    vec3 fragPos;
} vsOut;

layout (std140, binding = 4) uniform Material {
    uniform vec4 base_color_factor;
    uniform vec4 emissive_factor;
    uniform float metallic_factor;
    uniform float roughness_factor;
    uniform float normal_scale;
    uniform float occlusion_strength;
};

layout (std140, binding = 5) uniform Lighting {
    uniform vec4 lightPositions[4];
    uniform vec4 lightColors[4];
    uniform vec4 camPos;
    uniform uint lights;
};

layout(binding = 5) uniform samplerCube irradiance_map;

out vec4 FragColor;

const float PI = 3.14159265359;

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

float geometry_ggx(float ndv, float roughness) {
    float a = roughness * roughness;
    float asq = a * a;

    float denom =
        ndv + sqrt(asq + ((1 - asq) * (ndv * ndv)));

    return (2 * ndv) / denom;
}

// Smith
float geometryShadowing(vec3 normal, vec3 viewDir, vec3 lightDir, float roughness) {
    float nv = max(dot(normal, viewDir), 0.0);
    float nl = max(dot(normal, lightDir), 0.0);
    //float ggx2 = geometrySchlick(nv, roughness);
    //float ggx1 = geometrySchlick(nl, roughness);

    float ggx2 = geometry_ggx(nv, roughness);
    float ggx1 = geometry_ggx(nl, roughness);

    return ggx1 * ggx2;
}

vec3 fresnelSchlickRoughness(float cosTheta, vec3 F0, float roughness) {
    return F0 + (max(vec3(1.0 - roughness), F0) - F0) * pow(clamp(1.0 - cosTheta, 0.0, 1.0), 5.0);
}

void main() {
    float gamma = 2.2;

    vec4 g_albedo = vec4(pow(base_color_factor.rgb, vec3(gamma)), base_color_factor.w);

    vec3 f0 = vec3(0.04);
    f0 = mix(f0, base_color_factor.rgb, metallic_factor);

    vec3 viewDir = normalize(camPos.xyz - vsOut.fragPos);
    vec3 normal = normalize(vsOut.normal);

    vec3 totalRadiance = vec3(0.);
    for (int i = 0; i < lights; i++) {
        vec3 lightDir = normalize(lightPositions[i].xyz - vsOut.fragPos);
        vec3 halfway = normalize(viewDir + lightDir);
        float cosTheta = max(dot(halfway, viewDir), 0.);

        // Should add attenuation...
        vec3 radiance = lightColors[i].xyz;

        // Cook-Torrance BRDF
        float normalDistribution = normalDistribution(normal, halfway, roughness_factor + 0.01);
        float geometry = geometryShadowing(normal, viewDir, lightDir, roughness_factor + 0.01);

        vec3 fresnel = fresnelSchlick(f0, cosTheta);

        vec3 numerator = normalDistribution * geometry * fresnel;
        float denominator = 4.0 * max(dot(viewDir, normal), 0.0) * max(dot(lightDir, normal), 0.0);
        // + 0.0001 to prevent divide by zero
        vec3 specular = numerator / (denominator + 0.00001);

        vec3 kS = fresnel;
        vec3 kD = vec3(1.0) - kS;
        kD *= 1.0 - metallic_factor;

        float nl = max(dot(normal, lightDir), 0.0);

        totalRadiance += ((kD * base_color_factor.rgb / PI) + specular) * radiance * nl;
    }

    // ambient
    vec3 kS = fresnelSchlickRoughness(max(dot(normal, viewDir), 0.0), f0, roughness_factor + 0.01);
    vec3 kD = 1.0 - kS;
    vec3 irradiance = texture(irradiance_map, normal).rgb;
    vec3 diffuse    = irradiance * base_color_factor.rgb;
    vec3 ambient    = (kD * diffuse);

    vec3 color = emissive_factor.rgb + ambient.rgb + totalRadiance;

    // HDR tonemapping
    color = color / (color + vec3(1.0));
    color = pow(color, vec3(1.0 / gamma));

    FragColor = vec4(color, 1.0);
}
