
// MERL API implementation

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

#define BRDF_SAMPLING_RES_THETA_H 90
#define BRDF_SAMPLING_RES_THETA_D 90
#define BRDF_SAMPLING_RES_PHI_D 360

#define RED_SCALE (1.0 / 1500.0)
#define GREEN_SCALE (1.15 / 1500.0)
#define BLUE_SCALE (1.66 / 1500.0)

// Lookup theta_half index
// This is a non-linear mapping!
// In:  [0 .. pi/2]
// Out: [0 .. 89]
int theta_half_index(float theta_half)
{
    if (theta_half <= 0.0) {
        return 0;
    }

    float theta_half_deg = ((theta_half / (PI / 2.0)) * BRDF_SAMPLING_RES_THETA_H);
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
    int tmp = int(theta_diff / (PI * 0.5) * BRDF_SAMPLING_RES_THETA_D);

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
    // phi_diff -> phi_diff + PI
    if (phi_diff < 0.0) {
        phi_diff += PI;
    }

    // In: phi_diff in [0 .. pi]
    // Out: tmp in [0 .. 179]
    int tmp = int(phi_diff / PI * BRDF_SAMPLING_RES_PHI_D / 2);
    if (tmp < 0) {
        return 0;
    } else if (tmp < BRDF_SAMPLING_RES_PHI_D / 2 - 1) {
        return tmp;
    } else {
        return BRDF_SAMPLING_RES_PHI_D / 2 - 1;
    }
}

// ---------------------------------------------------------------------------------
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
vec3 lookup_brdf_merl(vec3 toLight, vec3 toViewer, vec3 normal, vec3 tangent, vec3 bitangent)
{
    float NoL = dot(normal, toLight);
    float NoV = dot(normal, toViewer);

    if (NoL < 0 || NoV < 0) {
        return vec3(0.);
    }

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

    float r = float(merlBrdfTable[redIndex] * RED_SCALE);
    float g = float(merlBrdfTable[greenIndex] * GREEN_SCALE);
    float b = float(merlBrdfTable[blueIndex] * BLUE_SCALE);

    if (r < 0. || g < 0. || b < 0.) {
        return vec3(0.);
    }

    return vec3(r, g, b);
}
// ---------------------------------------------------------------------------------
