use glam::{Vec2, Vec3};

use super::{Primitive, Vertex};

impl Primitive {
    /// Inspired by http://foundationsofgameenginedev.com/FGED2-sample.pdf
    pub(super) fn calculate_tangents(&mut self, vertex_buf: &mut Vec<Vertex>, index_buf: &[u32]) {
        // Tangents are already stored in the vertex buffer
        let mut bitagents: Vec<[f32; 3]> = vec![[0.; 3]; vertex_buf.len()];
        let mut counts: Vec<u32> = vec![0; vertex_buf.len()];

        for indices in index_buf.chunks_exact(3) {
            let i0 = indices[0] as usize;
            let i1 = indices[1] as usize;
            let i2 = indices[2] as usize;

            let p0 = Vec3::from(vertex_buf[i0].pos);
            let p1 = Vec3::from(vertex_buf[i1].pos);
            let p2 = Vec3::from(vertex_buf[i2].pos);

            let uv0 = Vec2::from(vertex_buf[i0].texcoords);
            let uv1 = Vec2::from(vertex_buf[i1].texcoords);
            let uv2 = Vec2::from(vertex_buf[i2].texcoords);

            // Vectors from p0 to p1 and p2
            let e1 = p1 - p0;
            let e2 = p2 - p0;

            // This is a bit tricky, but it makes sense...
            // Just find a linear combination of tangent and bitangent vectors that result in vectors e1 and e2.
            // Deltas and e1 / e2 are basically the same thing...
            let dx1 = uv1.x - uv0.x;
            let dy1 = uv1.y - uv0.y;
            let dx2 = uv2.x - uv0.x;
            let dy2 = uv2.y - uv0.y;

            let r = 1. / (dx1 * dy2 - dx2 * dy1);
            // 2x2 by 2x3 matrix multiplication
            let mut t = (e1 * dy2 - e2 * dy1) * r;
            let mut b = (e2 * dx1 - e1 * dx2) * r;

            // FIXME: tangents nan and infinite... maybe degenerate triangles ?
            if !t.is_finite() {
                eprintln!("t not finite");
                t = Vec3::new(1., 1., 1.);
            }

            if !b.is_finite() {
                eprintln!("b not finite");
                b = Vec3::new(1., 1., 1.);
            }

            vertex_buf[i0].tangent[0] += t[0];
            vertex_buf[i0].tangent[1] += t[1];
            vertex_buf[i0].tangent[2] += t[2];

            vertex_buf[i1].tangent[0] += t[0];
            vertex_buf[i1].tangent[1] += t[1];
            vertex_buf[i1].tangent[2] += t[2];

            vertex_buf[i2].tangent[0] += t[0];
            vertex_buf[i2].tangent[1] += t[1];
            vertex_buf[i2].tangent[2] += t[2];

            bitagents[i0][0] += b[0];
            bitagents[i0][1] += b[1];
            bitagents[i0][2] += b[2];

            bitagents[i1][0] += b[0];
            bitagents[i1][1] += b[1];
            bitagents[i1][2] += b[2];

            bitagents[i2][0] += b[0];
            bitagents[i2][1] += b[1];
            bitagents[i2][2] += b[2];

            counts[i0] += 1;
            counts[i1] += 1;
            counts[i2] += 1;
        }

        for (i, v) in vertex_buf.iter_mut().enumerate() {
            let n = Vec3::from(v.normal);
            let mut t = Vec3::from_slice(&v.tangent[0..4]);
            let b = Vec3::from(bitagents[i]);

            // Average the tangents... not great but it probably works
            if counts[i] != 0 {
                t *= 1. / counts[i] as f32;
            }

            t = t.normalize();

            let det = t.cross(b).dot(n);
            let handedness = if det > 0. { 1. } else { -1. };

            v.tangent[0..3].copy_from_slice(&t.to_array());
            v.tangent[3] = handedness;
        }
    }
}
