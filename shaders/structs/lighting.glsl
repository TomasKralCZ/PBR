
layout(std140, binding = {{ consts.buffer_bindings.lighting }}) uniform Lighting
{
    uniform vec4 lightPositions[4];
    uniform vec4 lightColors[4];
    uniform vec4 camPos;
    uniform uint lights;
};
