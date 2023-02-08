
// clang-format off
#ifdef ALBEDO_MAP
layout(binding = {{consts.texture_ports.albedo}}) uniform sampler2D abledoTex;
#endif

#ifdef MR_MAP
layout(binding = {{consts.texture_ports.mr}}) uniform sampler2D mrTex;
#endif

#ifdef NORMAL_MAP
layout(binding = {{consts.texture_ports.normal}}) uniform sampler2D normalTex;
#endif

#ifdef OCCLUSION_MAP
layout(binding = {{consts.texture_ports.occlusion}}) uniform sampler2D occlusionTex;
#endif

#ifdef EMISSIVE_MAP
layout(binding = {{consts.texture_ports.emissive}}) uniform sampler2D emissiveTex;
#endif

#ifdef CLEARCOAT_INTENSITY_MAP
layout(binding = {{consts.texture_ports.clearcoat_intensity}}) uniform sampler2D clearcoatIntensityTex;
#endif

#ifdef CLEARCOAT_ROUGHNESS_MAP
layout(binding = {{consts.texture_ports.clearcoat_roughness}}) uniform sampler2D clearcoatRoughnessTex;
#endif

#ifdef CLEARCOAT_NORMAL_MAP
layout(binding = {{consts.texture_ports.clearcoat_normal}}) uniform sampler2D clearcoatNormalTex;
#endif

layout(binding = {{consts.texture_ports.irradiance}}) uniform samplerCube irradianceMap;
layout(binding = {{consts.texture_ports.prefilter}}) uniform samplerCube prefilterMap;
layout(binding = {{consts.texture_ports.brdf}}) uniform sampler2D brdfLut;
layout(binding = {{consts.texture_ports.raw_brdf}}) uniform samplerCube rawBrdfMap;
// clang-format on
