
// clang-format off
#ifdef MERL_BRDF
layout(std430, binding = {{ consts.buffer_bindings.brdf_merl }}) buffer merlBrdfData { double merlBrdfTable[]; };
#endif

#ifdef UTIA_BRDF
layout(std430, binding = {{ consts.buffer_bindings.brdf_utia }}) buffer utiaBrdfData { double utiaBrdfTable[]; };
#endif
// clang-format on
