#version 460 core

// {% include "tools/tonemap.glsl" %}

out vec4 FragColor;

in vec3 localPos;

layout(binding = 0) uniform samplerCube environmentMap;

void main()
{
    vec3 envColor = texture(environmentMap, localPos).rgb;

    tonemap(envColor);
    envColor = pow(envColor, vec3(1.0 / 2.2));

    FragColor = vec4(envColor, 1.0);
}
