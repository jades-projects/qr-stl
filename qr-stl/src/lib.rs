mod result;

use qrcode_generator::QrCodeEcc;
use std::convert::TryInto;
use std::io;
use std::ops::{Add, Deref, Mul, Neg, Sub};
use stl_io::{Normal, Triangle};

pub use crate::result::Result;

const NO_NORMAL: Normal = [0., 0., 0.];

type Matrix = [Vec<bool>];

trait MatPeek {
    fn mat_peek(&self, x: isize, y: isize) -> bool;
}

impl MatPeek for [Vec<bool>] {
    /// Gets the pixel value at a given position in the matrix. Defaults to false (no pixel).
    fn mat_peek(&self, x: isize, y: isize) -> bool {
        let x: std::result::Result<usize, _> = x.try_into();
        let y: std::result::Result<usize, _> = y.try_into();

        match (x, y) {
            (Ok(x), Ok(y)) => self
                .get(y) //
                .and_then(|row| row.get(x))
                .copied(),
            (_, _) => None,
        }
        .unwrap_or(false)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Vec3([f32; 3]);

impl Deref for Vec3 {
    type Target = [f32; 3];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Add<Vec3> for Vec3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self([self[0] + rhs[0], self[1] + rhs[1], self[2] + rhs[2]])
    }
}

impl Sub<Vec3> for Vec3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        self + -rhs
    }
}

impl Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self {
        Self([-self[0], -self[1], -self[2]])
    }
}

#[derive(Clone, Copy, Debug)]
struct Scale(f32);

impl Mul<Vec3> for Scale {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Vec3 {
        Vec3([rhs[0] * self.0, rhs[1] * self.0, rhs[2] * self.0])
    }
}

impl Mul<Triangle> for Scale {
    type Output = Triangle;

    fn mul(self, Triangle { normal, vertices }: Triangle) -> Triangle {
        Triangle {
            normal,
            vertices: [
                *(self * Vec3(vertices[0])),
                *(self * Vec3(vertices[1])),
                *(self * Vec3(vertices[2])),
            ],
        }
    }
}

fn rect(p1: Vec3, p2: Vec3) -> [Triangle; 2] {
    let tri1 = Triangle {
        normal: NO_NORMAL,
        vertices: [
            *p2,
            *p1,
            [p1[0].max(p2[0]), p1[1].min(p2[1]), p1[2].max(p2[2])],
        ],
    };
    let tri2 = Triangle {
        normal: NO_NORMAL,
        vertices: [
            *p1,
            *p2,
            [p1[0].min(p2[0]), p1[1].max(p2[1]), p1[2].min(p2[2])],
        ],
    };
    [tri1, tri2]
}

pub fn matrix_to_triangles(matrix: &Matrix, scale: f32) -> Vec<Triangle> {
    let thickness = scale * 0.25;
    let scale = Scale(scale);

    let mut tris = Vec::new();
    for (my, row) in matrix.iter().enumerate() {
        for (mx, &val) in row.iter().enumerate() {
            let x = mx as f32;
            let y = my as f32;

            if val {
                // if our pixel is black, emit a raised tile and edge detect around its position to emit sides

                // emit raised tile
                tris.extend_from_slice(&rect(
                    scale * Vec3([x, y, thickness]),
                    scale * Vec3([x + 1., y + 1., thickness]),
                ));

                let dirs = [
                    // left
                    (
                        (mx as isize - 1, my as isize), //
                        ((x, y), (x, y + 1.)),
                        false,
                    ),
                    // down
                    (
                        (mx as isize, my as isize + 1),
                        ((x + 1., y + 1.), (x, y + 1.)),
                        true,
                    ),
                    // // right
                    (
                        (mx as isize + 1, my as isize),
                        ((x + 1., y), (x + 1., y + 1.)),
                        true,
                    ),
                    // up
                    (
                        (mx as isize, my as isize - 1), //
                        ((x + 1., y), (x, y)),
                        false,
                    ),
                ];

                for &((peekx, peeky), (gen_start, gen_end), inv) in &dirs {
                    // peeked tile is white (so a transition)
                    if !matrix.mat_peek(peekx, peeky) {
                        let p1 = scale * Vec3([gen_start.0, gen_start.1, 0.]);
                        let p2 = scale * Vec3([gen_end.0, gen_end.1, thickness]);

                        let (p1, p2) = if inv { (p2, p1) } else { (p1, p2) };
                        tris.extend_from_slice(&rect(p1, p2));
                    }
                }
            } else {
                // if our pixel is white, emit a floor tile
                tris.extend_from_slice(&rect(
                    scale * Vec3([x, y, 0.]),
                    scale * Vec3([x + 1., y + 1., 0.]),
                ));
            }
        }
    }

    tris
}

fn offset_tris(tris: &mut [Triangle], offset: Vec3) {
    for tri in tris {
        tri.vertices.iter_mut().for_each(|vert| {
            *vert = [
                vert[0] + offset[0],
                vert[1] + offset[1],
                vert[2] + offset[2],
            ]
        });
    }
}

pub struct MeshOptions {
    pub pixel_size: f32,
    pub base_size: f32,
    pub base_height: f32,
}

pub fn qr_to_triangles(input: &[u8], opts: &MeshOptions) -> Result<Vec<Triangle>> {
    let mat = qrcode_generator::to_matrix(input, QrCodeEcc::Low)?;
    let mut tris = matrix_to_triangles(&mat, opts.pixel_size);

    let total = mat.len() as f32 * opts.pixel_size + 2. * opts.base_size;
    let base = opts.base_size;
    let height = opts.base_height;
    offset_tris(&mut tris, Vec3([opts.base_size, opts.base_size, height]));

    // bottom
    tris.extend_from_slice(&rect(Vec3([total, total, 0.]), Vec3([0., 0., 0.])));

    // surround
    tris.extend_from_slice(&rect(
        Vec3([0., 0., height]),
        Vec3([total - base, base, height]),
    ));
    tris.extend_from_slice(&rect(
        Vec3([0., base, height]), //
        Vec3([base, total, height]),
    ));
    tris.extend_from_slice(&rect(
        Vec3([base, total - base, height]), //
        Vec3([total - base, total, height]),
    ));
    tris.extend_from_slice(&rect(
        Vec3([total - base, 0., height]), //
        Vec3([total, total, height]),
    ));

    // sides
    tris.extend_from_slice(&rect(Vec3([total, 0., 0.]), Vec3([0., 0., height])));
    tris.extend_from_slice(&rect(Vec3([0., 0., 0.]), Vec3([0., total, height])));
    tris.extend_from_slice(&rect(Vec3([total, total, height]), Vec3([total, 0., 0.])));
    tris.extend_from_slice(&rect(Vec3([0., total, height]), Vec3([total, total, 0.])));

    Ok(tris)
}

pub fn save_stl<W: io::Write>(tris: &[Triangle], writer: &mut W) -> Result<()> {
    Ok(stl_io::write_stl(writer, tris.iter())?)
}
