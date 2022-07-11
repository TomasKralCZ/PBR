#version 460 core

in VsOut {
    vec2 texCoords;
    vec3 normal;
    vec3 fragPos;
} vsOut;

uniform vec3 lightColor;

out vec4 FragColor;

void main() {
    FragColor = vec4(lightColor, 1.0);
}
