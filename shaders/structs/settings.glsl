
layout(std140, binding = 3) uniform Settings
{
    uniform bool clearcoatEnabled;
    uniform bool directLightEnabled;
    uniform bool IBLEnabled;
    uniform bool realtimeIBL;
    uniform uint diffuseType;
};

