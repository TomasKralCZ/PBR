
#ifdef MERL_BRDF
layout(std430, binding = 10) buffer merlBrdfData { double merlBrdfTable[]; };
#endif

#ifdef MIT_BRDF
layout(std430, binding = 11) buffer mitBrdfData { float mitBrdfTable[]; };
#endif

#ifdef UTIA_BRDF
layout(std430, binding = 12) buffer utiaBrdfData { double utiaBrdfTable[]; };
#endif
