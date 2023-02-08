
#ifdef MERL_BRDF
layout(std430, binding = 10) buffer merlBrdfData { double merlBrdfTable[]; };
#endif

#ifdef UTIA_BRDF
layout(std430, binding = 11) buffer utiaBrdfData { double utiaBrdfTable[]; };
#endif
