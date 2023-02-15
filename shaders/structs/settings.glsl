// clang-format off
layout(std140, binding = {{ consts.buffer_bindings.settings }}) uniform Settings
// clang-format on
{
    uniform bool clearcoatEnabled;
    uniform bool directLightEnabled;
    uniform bool IBLEnabled;
    uniform uint diffuseType;
    uniform bool energyCompEnabled;
};

