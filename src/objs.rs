use eframe::egui;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;
pub struct GUIState {
    pub ctx: Option<egui::Context>,
}

impl GUIState {
    pub fn new() -> Self {
        Self {
            ctx: None,
        }
    }
}

pub struct DisplayBuffers {
    pub buf_size: [usize; 2],
    pub bufs: Vec<Arc<Mutex<Vec<egui::Color32>>>>,
    // Ready, In-Progress, Present
    pub trip_state: Arc<Mutex<[i64; 3]>>,
    pub stale: Arc<AtomicBool>,
}

impl DisplayBuffers{
    pub fn new(buf_size: [usize; 2]) -> Self{

        let mut vec = Vec::with_capacity(buf_size[0]*buf_size[1]);
        for _ in 0..buf_size[0]*buf_size[1] {
            vec.push(egui::Color32::from_rgba_premultiplied(0, 0, 0, 255,));
        }
        let mut bufs = Vec::with_capacity(3);
        for _ in 0..3 {
            bufs.push(Arc::new(Mutex::new(vec.to_vec())));
        }


        let trip_state = Arc::new(Mutex::new([0,1,2]));

        let stale = Arc::new(AtomicBool::new(true));

        Self { 
            buf_size,
            bufs,
            trip_state,
            stale,
        }
    }

    pub fn from(buffers_copy: &DisplayBuffers) -> Self{
        let mut bufs = Vec::with_capacity(3);
        for i in 0..3 {
            bufs.push(buffers_copy.bufs[i].clone());
        }
        Self { buf_size: buffers_copy.buf_size, bufs, trip_state: buffers_copy.trip_state.clone(), stale: buffers_copy.stale.clone() }
    }
}

pub struct Vec3d {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3d {
    pub fn new(x: f64, y: f64, z: f64)->Self{
        Self { x, y, z,}
    }
}
pub struct Tri {
    pub p: Vec<Vec3d>,
}

impl Tri {
    pub fn new(p: Vec<Vec3d>)->Self{
        Self { p, }
    }
}
pub struct Mesh {
    pub tris: Vec<Tri>,
}

impl Mesh {
    pub fn new(tris: Vec<Tri>)->Self{
        Self { tris, }
    }
}

pub struct Matrix4x4 {
    pub m: Vec<Vec<f64>>,
}

impl Matrix4x4 {
    pub fn new(m: Vec<Vec<f64>>)->Self{
        Self { m, }
    }

    pub fn mul_mat_vec(&self, i: &Vec3d)->Vec3d{
        let mut x = i.x * self.m[0][0] + i.y * self.m[1][0] + i.z * self.m[2][0] + self.m[3][0];
		let mut y = i.x * self.m[0][1] + i.y * self.m[1][1] + i.z * self.m[2][1] + self.m[3][1];
		let mut z = i.x * self.m[0][2] + i.y * self.m[1][2] + i.z * self.m[2][2] + self.m[3][2];
		let w = i.x * self.m[0][3] + i.y * self.m[1][3] + i.z * self.m[2][3] + self.m[3][3];

		if w != 0.0 {
			x /= w;
            y /= w;
            z /= w;
		}

        Vec3d::new(x, y, z)
    }

    pub fn mul_mat_tri(&self, t: &Tri)->Tri{
        let mut tri_projected = Vec::with_capacity(3);
        tri_projected.push(self.mul_mat_vec(&t.p[0]));
        tri_projected.push(self.mul_mat_vec(&t.p[1]));
        tri_projected.push(self.mul_mat_vec(&t.p[2]));
        Tri::new(tri_projected)
    }
}