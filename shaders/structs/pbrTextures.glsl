
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

#ifdef CLEARCOAT_NORMAL_MAP
layout(binding = 7) uniform sampler2D clearcoatNormalTex;
#endif

layout(binding = 8) uniform samplerCube irradianceMap;
layout(binding = 9) uniform samplerCube prefilterMap;
layout(binding = 10) uniform sampler2D brdfLut;
layout(binding = 11) uniform samplerCube rawBrdfMap;
layout(binding = 12) uniform samplerCube cubemap;
