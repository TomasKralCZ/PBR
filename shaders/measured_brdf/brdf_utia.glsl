
// Based on Jiri Filip's code

const float STEP_T = 15.;
const float STEP_P = 7.5;
const int NTI = 6;
const int NTV = 6;
const int NPI = int(360. / STEP_P);
const int NPV = int(360. / STEP_P);
const int PLANES = 3;

vec3 filip_lookup_brdf(float theta_i, float phi_i, float theta_v, float phi_v)
{
    vec3 RGB = vec3(0.);

    float PI2 = PI * 0.5;
    if (theta_i > PI2 || theta_v > PI2) {
        return RGB;
    }

    float r2d = 180. / PI;
    theta_i *= r2d;
    theta_v *= r2d;
    phi_i *= r2d;
    phi_v *= r2d;
    if (phi_i >= 360.)
        phi_i = 0.;
    if (phi_v >= 360.)
        phi_v = 0.;

    int iti[2], itv[2], ipi[2], ipv[2];
    iti[0] = int(floor(theta_i / STEP_T));
    iti[1] = iti[0] + 1;
    if (iti[0] > NTI - 2) {
        iti[0] = NTI - 2;
        iti[1] = NTI - 1;
    }
    itv[0] = int(floor(theta_v / STEP_T));
    itv[1] = itv[0] + 1;
    if (itv[0] > NTV - 2) {
        itv[0] = NTV - 2;
        itv[1] = NTV - 1;
    }

    ipi[0] = int(floor(phi_i / STEP_P));
    ipi[1] = ipi[0] + 1;
    ipv[0] = int(floor(phi_v / STEP_P));
    ipv[1] = ipv[0] + 1;

    float sum = 0.;
    float wti[2], wtv[2], wpi[2], wpv[2];
    wti[1] = theta_i - float(STEP_T * iti[0]);
    wti[0] = float(STEP_T * iti[1]) - theta_i;
    sum = wti[0] + wti[1];
    wti[0] /= sum;
    wti[1] /= sum;
    wtv[1] = theta_v - float(STEP_T * itv[0]);
    wtv[0] = float(STEP_T * itv[1]) - theta_v;
    sum = wtv[0] + wtv[1];
    wtv[0] /= sum;
    wtv[1] /= sum;

    wpi[1] = phi_i - float(STEP_P * ipi[0]);
    wpi[0] = float(STEP_P * ipi[1]) - phi_i;
    sum = wpi[0] + wpi[1];
    wpi[0] /= sum;
    wpi[1] /= sum;
    wpv[1] = phi_v - float(STEP_P * ipv[0]);
    wpv[0] = float(STEP_P * ipv[1]) - phi_v;
    sum = wpv[0] + wpv[1];
    wpv[0] /= sum;
    wpv[1] /= sum;

    if (ipi[1] == NPI)
        ipi[1] = 0;
    if (ipv[1] == NPV)
        ipv[1] = 0;

    int nc = NPV * NTV;
    int nr = NPI * NTI;
    for (int isp = 0; isp < PLANES; isp++) {
        RGB[isp] = 0.;
        for (int i = 0; i < 2; i++) {
            for (int j = 0; j < 2; j++) {
                for (int k = 0; k < 2; k++) {
                    for (int l = 0; l < 2; l++) {
                        RGB[isp] += float(utiaBrdfTable[isp * nr * nc + nc * (NPI * iti[i] + ipi[k])
                                        + NPV * itv[j] + ipv[l]])
                            * wti[i] * wtv[j] * wpi[k] * wpv[l];
                    }
                }
            }
        }
    }

    return RGB;
}

vec3 lookup_brdf_utia(vec3 toLight, vec3 toViewer, vec3 normal, vec3 tangent, vec3 bitangent)
{
    float NoL = dot(normal, toLight);
    float NoV = dot(normal, toViewer);

    if (NoL < 0 || NoV < 0) {
        return vec3(0.);
    }

    float theta_in = acos(NoL);
    float theta_out = acos(NoV);

    vec3 projected_to_light = normalize(toLight - (clamp(NoL, 0.1, 0.9) * normal));
    vec3 projected_to_viewer = normalize(toViewer - (NoV * normal));

    float phi_in = acos(dot(normalize(tangent), projected_to_light));
    float phi_out = acos(dot(normalize(tangent), projected_to_viewer));

    vec3 rgb = filip_lookup_brdf(theta_in, phi_in, theta_out, phi_out);

    if (rgb.r < 0 || rgb.g < 0 || rgb.b < 0) {
        return vec3(0.);
    }

    return rgb;
}
