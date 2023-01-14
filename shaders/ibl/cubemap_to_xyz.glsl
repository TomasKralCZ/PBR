
vec3 directionFromCubemapUv(uvec3 gid, float cubemapSize)
{
    // Map from (0, size) to (0, 1)
    vec2 uv = vec2(gid.xy + 0.5) / cubemapSize;
    // Map from (0, 1) to (-1, 1)
    uv = (uv * 2.0) - 1.0;

    // For coordinates, check out https://www.khronos.org/opengl/wiki/Cubemap_Texture
    switch (gid.z) {
    case 0:
        return normalize(vec3(1.0, -uv.y, -uv.x));
    case 1:
        return normalize(vec3(-1.0, -uv.y, uv.x));
    case 2:
        return normalize(vec3(uv.x, 1.0, uv.y));
    case 3:
        return normalize(vec3(uv.x, -1.0, -uv.y));
    case 4:
        return normalize(vec3(uv.x, -uv.y, 1.0));
    default:
        return normalize(vec3(-uv.x, -uv.y, -1.0));
    }
}
