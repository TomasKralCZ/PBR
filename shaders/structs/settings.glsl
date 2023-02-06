
layout(std140, binding = 3) uniform Settings
{
    uniform bool clearcoatEnabled;
    uniform bool directLightEnabled;
    uniform bool IBLEnabled;
    uniform uint diffuseType;
};

