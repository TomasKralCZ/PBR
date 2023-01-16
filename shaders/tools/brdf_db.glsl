
// Copyright 2005 Mitsubishi Electric Research Laboratories All Rights Reserved.

// Permission to use, copy and modify this software and its documentation without
// fee for educational, research and non-profit purposes, is hereby granted, provided
// that the above copyright notice and the following three paragraphs appear in all copies.

// To request permission to incorporate this software into commercial products contact:
// Vice President of Marketing and Business Development;
// Mitsubishi Electric Research Laboratories (MERL), 201 Broadway, Cambridge, MA 02139 or
// <license@merl.com>.

// IN NO EVENT SHALL MERL BE LIABLE TO ANY PARTY FOR DIRECT, INDIRECT, SPECIAL, INCIDENTAL,
// OR CONSEQUENTIAL DAMAGES, INCLUDING LOST PROFITS, ARISING OUT OF THE USE OF THIS SOFTWARE AND
// ITS DOCUMENTATION, EVEN IF MERL HAS BEEN ADVISED OF THE POSSIBILITY OF SUCH DAMAGES.

// MERL SPECIFICALLY DISCLAIMS ANY WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
// WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE.  THE SOFTWARE PROVIDED
// HEREUNDER IS ON AN "AS IS" BASIS, AND MERL HAS NO OBLIGATIONS TO PROVIDE MAINTENANCE, SUPPORT,
// UPDATES, ENHANCEMENTS OR MODIFICATIONS.

// The original code was C++, this a very close GLSL translation done by me

layout(std430, binding = 10) buffer BrdfData { double brdfTable[]; };

#define BRDF_SAMPLING_RES_THETA_H 90
#define BRDF_SAMPLING_RES_THETA_D 90
#define BRDF_SAMPLING_RES_PHI_D 360

#define RED_SCALE (1.0 / 1500.0)
#define GREEN_SCALE (1.15 / 1500.0)
#define BLUE_SCALE (1.66 / 1500.0)
#define D_PI 3.1415926535897932384626433832795

// rotate vector along one axis
void rotate_vector(vec3 vector, vec3 axis, float angle, out vec3 res)
{
    float temp;
    float cos_ang = cos(angle);
    float sin_ang = sin(angle);

    res[0] = vector[0] * cos_ang;
    res[1] = vector[1] * cos_ang;
    res[2] = vector[2] * cos_ang;

    temp = axis[0] * vector[0] + axis[1] * vector[1] + axis[2] * vector[2];
    temp = temp * (1.0 - cos_ang);

    res[0] += axis[0] * temp;
    res[1] += axis[1] * temp;
    res[2] += axis[2] * temp;

    vec3 cross_vec = cross(axis, vector);

    res[0] += cross_vec[0] * sin_ang;
    res[1] += cross_vec[1] * sin_ang;
    res[2] += cross_vec[2] * sin_ang;
}

// convert standard coordinates to half vector/difference vector coordinates (Rusinkiewicz’s coordinate
// system)
void std_coords_to_half_diff_coords(float theta_in, float phi, float theta_out, float phi_out,
    out float theta_half, out float phi_half, out float theta_diff, out float phi_diff)
{
    // compute in vector
    float in_vec_z = cos(theta_in);
    float proj_in_vec = sin(theta_in);
    float in_vec_x = proj_in_vec * cos(phi);
    float in_vec_y = proj_in_vec * sin(phi);
    vec3 inDir = vec3(in_vec_x, in_vec_y, in_vec_z);
    normalize(inDir);

    // compute out vector
    float out_vec_z = cos(theta_out);
    float proj_out_vec = sin(theta_out);
    float out_vec_x = proj_out_vec * cos(phi_out);
    float out_vec_y = proj_out_vec * sin(phi_out);
    vec3 outDir = vec3(out_vec_x, out_vec_y, out_vec_z);
    normalize(outDir);

    // compute halfway vector
    float half_x = (in_vec_x + out_vec_x) / 2.0;
    float half_y = (in_vec_y + out_vec_y) / 2.0;
    float half_z = (in_vec_z + out_vec_z) / 2.0;
    vec3 halfway = vec3(half_x, half_y, half_z);
    normalize(halfway);

    // compute  theta_half, fi_half
    theta_half = acos(halfway[2]);
    phi_half = atan(halfway[1], halfway[0]);

    vec3 bi_normal = vec3(0.0, 1.0, 0.0);
    vec3 normal = vec3(0.0, 0.0, 1.0);
    vec3 temp = vec3(0.0);
    vec3 diff = vec3(0.0);

    // compute diff vector
    rotate_vector(inDir, normal, -phi_half, temp);
    rotate_vector(temp, bi_normal, -theta_half, diff);

    // compute  theta_diff, fi_diff
    theta_diff = acos(diff[2]);
    phi_diff = atan(diff[1], diff[0]);
}

// Lookup theta_half index
// This is a non-linear mapping!
// In:  [0 .. pi/2]
// Out: [0 .. 89]
int theta_half_index(float theta_half)
{
    if (theta_half <= 0.0) {
        return 0;
    }

    float theta_half_deg = ((theta_half / (D_PI / 2.0)) * BRDF_SAMPLING_RES_THETA_H);
    float temp = theta_half_deg * BRDF_SAMPLING_RES_THETA_H;
    temp = sqrt(temp);
    int ret_val = int(temp);

    if (ret_val < 0) {
        ret_val = 0;
    }

    if (ret_val >= BRDF_SAMPLING_RES_THETA_H) {
        ret_val = BRDF_SAMPLING_RES_THETA_H - 1;
    }

    return ret_val;
}

// Lookup theta_diff index
// In:  [0 .. pi/2]
// Out: [0 .. 89]
int theta_diff_index(float theta_diff)
{
    int tmp = int(theta_diff / (D_PI * 0.5) * BRDF_SAMPLING_RES_THETA_D);

    if (tmp < 0) {
        return 0;
    } else if (tmp < BRDF_SAMPLING_RES_THETA_D - 1) {
        return tmp;
    } else {
        return BRDF_SAMPLING_RES_THETA_D - 1;
    }
}

// Lookup phi_diff index
int phi_diff_index(float phi_diff)
{
    // Because of reciprocity, the BRDF is unchanged under
    // phi_diff -> phi_diff + D_PI
    if (phi_diff < 0.0) {
        phi_diff += D_PI;
    }

    // In: phi_diff in [0 .. pi]
    // Out: tmp in [0 .. 179]
    int tmp = int(phi_diff / D_PI * BRDF_SAMPLING_RES_PHI_D / 2);
    if (tmp < 0) {
        return 0;
    } else if (tmp < BRDF_SAMPLING_RES_PHI_D / 2 - 1) {
        return tmp;
    } else {
        return BRDF_SAMPLING_RES_PHI_D / 2 - 1;
    }
}

vec3 lookup_brdf(float theta_in, float phi_in, float theta_out, float phi_out)
{
    // Convert to halfangle / difference angle coordinates
    float theta_half, phi_half, theta_diff, phi_diff;
    std_coords_to_half_diff_coords(
        theta_in, phi_in, theta_out, phi_out, theta_half, phi_half, theta_diff, phi_diff);

    // Find index.
    // Note that phi_half is ignored, since isotropic BRDFs are assumed
    int ind = phi_diff_index(phi_diff) + theta_diff_index(theta_diff) * BRDF_SAMPLING_RES_PHI_D / 2
        + theta_half_index(theta_half) * BRDF_SAMPLING_RES_PHI_D / 2 * BRDF_SAMPLING_RES_THETA_D;

    double red = brdfTable[ind] * RED_SCALE;
    double green
        = brdfTable[ind + BRDF_SAMPLING_RES_THETA_H * BRDF_SAMPLING_RES_THETA_D * BRDF_SAMPLING_RES_PHI_D / 2]
        * GREEN_SCALE;
    double blue
        = brdfTable[ind + BRDF_SAMPLING_RES_THETA_H * BRDF_SAMPLING_RES_THETA_D * BRDF_SAMPLING_RES_PHI_D]
        * BLUE_SCALE;

    if (red < 0.0 || green < 0.0 || blue < 0.0) {
        // error
        return vec3(0, theta_out, phi_out);
    }

    return vec3(float(red), float(green), float(blue));
}

// Taken from the brdf-explorer

/*
Copyright Disney Enterprises, Inc. All rights reserved.

This license governs use of the accompanying software. If you use the software, you
accept this license. If you do not accept the license, do not use the software.

1. Definitions
The terms "reproduce," "reproduction," "derivative works," and "distribution" have
the same meaning here as under U.S. copyright law. A "contribution" is the original
software, or any additions or changes to the software. A "contributor" is any person
that distributes its contribution under this license. "Licensed patents" are a
contributor's patent claims that read directly on its contribution.

2. Grant of Rights
(A) Copyright Grant- Subject to the terms of this license, including the license
conditions and limitations in section 3, each contributor grants you a non-exclusive,
worldwide, royalty-free copyright license to reproduce its contribution, prepare
derivative works of its contribution, and distribute its contribution or any derivative
works that you create.
(B) Patent Grant- Subject to the terms of this license, including the license
conditions and limitations in section 3, each contributor grants you a non-exclusive,
worldwide, royalty-free license under its licensed patents to make, have made,
use, sell, offer for sale, import, and/or otherwise dispose of its contribution in the
software or derivative works of the contribution in the software.

3. Conditions and Limitations
(A) No Trademark License- This license does not grant you rights to use any
contributors' name, logo, or trademarks.
(B) If you bring a patent claim against any contributor over patents that you claim
are infringed by the software, your patent license from such contributor to the
software ends automatically.
(C) If you distribute any portion of the software, you must retain all copyright,
patent, trademark, and attribution notices that are present in the software.
(D) If you distribute any portion of the software in source code form, you may do
so only under this license by including a complete copy of this license with your
distribution. If you distribute any portion of the software in compiled or object code
form, you may only do so under a license that complies with this license.
(E) The software is licensed "as-is." You bear the risk of using it. The contributors
give no express warranties, guarantees or conditions. You may have additional
consumer rights under your local laws which this license cannot change.
To the extent permitted under your local laws, the contributors exclude the
implied warranties of merchantability, fitness for a particular purpose and non-
infringement.
*/
vec3 lookup_brdf(vec3 toLight, vec3 toViewer, vec3 normal, vec3 tangent, vec3 bitangent)
{
    vec3 H = normalize(toLight + toViewer);
    float theta_H = acos(clamp(dot(normal, H), 0, 1));
    float theta_diff = acos(clamp(dot(H, toLight), 0, 1));
    float phi_diff = 0;

    if (theta_diff < 1e-3) {
        // phi_diff indeterminate, use phi_half instead
        phi_diff = atan(clamp(-dot(toLight, bitangent), -1, 1), clamp(dot(toLight, tangent), -1, 1));
    } else if (theta_H > 1e-3) {
        // use Gram-Schmidt orthonormalization to find diff basis vectors
        vec3 u = -normalize(normal - dot(normal, H) * H);
        vec3 v = cross(H, u);
        phi_diff = atan(clamp(dot(toLight, v), -1, 1), clamp(dot(toLight, u), -1, 1));
    } else {
        theta_H = 0;
    }

    // Find index.
    // Note that phi_half is ignored, since isotropic BRDFs are assumed
    int ind = phi_diff_index(phi_diff) + theta_diff_index(theta_diff) * BRDF_SAMPLING_RES_PHI_D / 2
        + theta_half_index(theta_H) * BRDF_SAMPLING_RES_PHI_D / 2 * BRDF_SAMPLING_RES_THETA_D;

    int redIndex = ind;
    int greenIndex
        = ind + BRDF_SAMPLING_RES_THETA_H * BRDF_SAMPLING_RES_THETA_D * BRDF_SAMPLING_RES_PHI_D / 2;
    int blueIndex = ind + BRDF_SAMPLING_RES_THETA_H * BRDF_SAMPLING_RES_THETA_D * BRDF_SAMPLING_RES_PHI_D;

    return vec3(brdfTable[redIndex] * RED_SCALE, brdfTable[greenIndex] * GREEN_SCALE,
        brdfTable[blueIndex] * BLUE_SCALE);
}
