use eframe::egui;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;
use std::ops::{Add, Mul, Sub};

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

#[derive(Copy, Clone)]
pub struct Vec3d {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

impl Vec3d {
    pub fn new(x: f64, y: f64, z: f64)->Self{
        Self { x, y, z, w: 1.0}
    }

    pub fn dot(&self, rhs: &Vec3d)->f64{
        self.x*rhs.x+self.y*rhs.y+self.z*rhs.z
    }

    pub fn cross(&self, rhs: &Vec3d)->Vec3d{
        Self{
            x: self.y*rhs.z-self.z*rhs.y,
            y: self.z*rhs.x-self.x*rhs.z,
            z: self.x*rhs.y-self.y*rhs.x,
            w: 1.0,
        }
    }

    pub fn normalize(&mut self){
        let den = (self.x*self.x + self.y*self.y + self.z*self.z).sqrt();
        self.x /= den;
        self.y /= den;
        self.z /= den;
    }

    pub fn vector_intersect_plane(plane_p: &Vec3d, plane_n: &Vec3d, line_start: &Vec3d, line_end: &Vec3d)->Vec3d{
        let plane_d = -plane_n.dot(plane_p);
        let ad = line_start.dot(plane_n);
        let bd = line_end.dot(plane_n);
        let t = (-plane_d - ad) / (bd - ad);
        *line_start + (*line_end - *line_start)*t
    }
}

impl Add for Vec3d {
    type Output = Vec3d;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
            w: 1.0,
        }
    }
}

impl Sub for Vec3d {
    type Output = Vec3d;
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
            w: 1.0,
        }
    }
}

impl Mul<f64> for Vec3d {
    type Output = Vec3d;
    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            x: self.x*rhs,
            y: self.y*rhs,
            z: self.z*rhs,
            w: 1.0,
        }
    }
}

#[derive(Clone)]
pub struct Tri {
    pub p: Vec<Vec3d>,
    pub shade: u8,
}

impl Tri {
    pub fn new(p: Vec<Vec3d>)->Self{
        Self { p, shade:255 }
    }

    pub fn get_normal(&self) -> Vec3d{
        let (mut normal, line1, line2);

        line1 = self.p[1] - self.p[0];
        line2 = self.p[2] - self.p[0];

        normal = line1.cross(&line2);
        normal.normalize();

        normal
    }

    pub fn triangle_clip_against_plane(&self, plane_p: &Vec3d, plane_n: &Vec3d)->Vec<Tri>{
		// Return signed shortest distance from point to plane, plane normal must be normalised
		let dist = |p: &Vec3d| {
			plane_n.x * p.x + plane_n.y * p.y + plane_n.z * p.z - plane_n.dot(plane_p)
		};

        let mut ret: Vec<Tri> = Vec::with_capacity(2);

		// Create two temporary storage arrays to classify points either side of plane
		// If distance sign is positive, point lies on "inside" of plane
		let mut inside_points = Vec::with_capacity(3);  let n_inside_point_count;
		let mut outside_points = Vec::with_capacity(3); let n_outside_point_count;

		// Get signed distance of each point in triangle to plane
		let d0 = dist(&self.p[0]);
		let d1 = dist(&self.p[1]);
		let d2 = dist(&self.p[2]);

        // println!("{} {} {}",d0,d1,d2);

		if d0 >= 0.0 { inside_points.push(self.p[0]); }
		else { outside_points.push(self.p[0]); }
		if d1 >= 0.0 { inside_points.push(self.p[1]); }
		else { outside_points.push(self.p[1]); }
		if d2 >= 0.0 { inside_points.push(self.p[2]); }
		else { outside_points.push(self.p[2]); }

		// Now classify triangle points, and break the input triangle into 
		// smaller output triangles if required. There are four possible
		// outcomes...
        n_inside_point_count = inside_points.len();
        n_outside_point_count = outside_points.len();

		if n_inside_point_count == 0
		{
			// All points lie on the outside of plane, so clip whole triangle
			// It ceases to exist

			return ret; // No returned triangles are valid
		}

		if n_inside_point_count == 3
		{
			// All points lie on the inside of plane, so do nothing
			// and allow the triangle to simply pass through
			ret.push(self.clone());

			return ret; // Just the one returned original triangle is valid
		}

		if n_inside_point_count == 1 && n_outside_point_count == 2
		{
			// Triangle should be clipped. As two points lie outside
			// the plane, the triangle simply becomes a smaller triangle

			// Copy appearance info to new triangle
			ret.push(self.clone());

			// The inside point is valid, so keep that...
			ret[0].p[0] = inside_points[0];

			// but the two new points are at the locations where the 
			// original sides of the triangle (lines) intersect with the plane
			ret[0].p[1] = Vec3d::vector_intersect_plane(&plane_p, &plane_n, &inside_points[0], &outside_points[0]);
			ret[0].p[2] = Vec3d::vector_intersect_plane(&plane_p, &plane_n, &inside_points[0], &outside_points[1]);

			return ret; // Return the newly formed single triangle
		}

		if n_inside_point_count == 2 && n_outside_point_count == 1
		{
			// Triangle should be clipped. As two points lie inside the plane,
			// the clipped triangle becomes a "quad". Fortunately, we can
			// represent a quad with two new triangles

			// Copy appearance info to new triangles
            ret.push(self.clone());
            ret.push(self.clone());

			// The first triangle consists of the two inside points and a new
			// point determined by the location where one side of the triangle
			// intersects with the plane
			ret[0].p[0] = inside_points[0];
			ret[0].p[1] = inside_points[1];
			ret[0].p[2] = Vec3d::vector_intersect_plane(&plane_p, &plane_n, &inside_points[0], &outside_points[0]);

			// The second triangle is composed of one of he inside points, a
			// new point determined by the intersection of the other side of the 
			// triangle and the plane, and the newly created point above
			ret[1].p[0] = inside_points[1];
			ret[1].p[1] = ret[0].p[2];
			ret[1].p[2] = Vec3d::vector_intersect_plane(&plane_p, &plane_n, &inside_points[1], &outside_points[0]);

			return ret; // Return two newly formed triangles which form a quad
		}
        return ret;
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

    // Only for rot and trans matrices
    pub fn quick_inverse(&self)->Self{
            let mut matrix = Matrix4x4::new(vec![vec![0.0; 4]; 4]);
            matrix.m[0][0] = self.m[0][0]; matrix.m[0][1] = self.m[1][0]; matrix.m[0][2] = self.m[2][0]; matrix.m[0][3] = 0.0;
            matrix.m[1][0] = self.m[0][1]; matrix.m[1][1] = self.m[1][1]; matrix.m[1][2] = self.m[2][1]; matrix.m[1][3] = 0.0;
            matrix.m[2][0] = self.m[0][2]; matrix.m[2][1] = self.m[1][2]; matrix.m[2][2] = self.m[2][2]; matrix.m[2][3] = 0.0;
            matrix.m[3][0] = -(self.m[3][0] * matrix.m[0][0] + self.m[3][1] * matrix.m[1][0] + self.m[3][2] * matrix.m[2][0]);
            matrix.m[3][1] = -(self.m[3][0] * matrix.m[0][1] + self.m[3][1] * matrix.m[1][1] + self.m[3][2] * matrix.m[2][1]);
            matrix.m[3][2] = -(self.m[3][0] * matrix.m[0][2] + self.m[3][1] * matrix.m[1][2] + self.m[3][2] * matrix.m[2][2]);
            matrix.m[3][3] = 1.0;
            return matrix;
    }

    pub fn make_rotation_x(&mut self, ftheta: f64){
        self.m[0][0] = 1.0;
		self.m[1][1] = ftheta.cos();
		self.m[1][2] = ftheta.sin();
		self.m[2][1] = -ftheta.sin();
		self.m[2][2] = ftheta.cos();
		self.m[3][3] = 1.0;
    }

    pub fn make_rotation_y(&mut self, ftheta: f64){
        self.m[0][0] = ftheta.cos();
		self.m[0][2] = ftheta.sin();
		self.m[2][0] = -ftheta.sin();
		self.m[1][1] = 1.0;
		self.m[2][2] = ftheta.cos();
		self.m[3][3] = 1.0;
    }

    pub fn make_rotation_z(&mut self, ftheta: f64){
        self.m[0][0] = ftheta.cos();
		self.m[0][1] = ftheta.sin();
		self.m[1][0] = -ftheta.sin();
		self.m[1][1] = ftheta.cos();
		self.m[2][2] = 1.0;
		self.m[3][3] = 1.0;
    }

    pub fn make_projection(&mut self, f_fov_degrees: f64, f_aspect_ratio: f64, f_near: f64, f_far: f64){
        let f_fov_rad = 1.0 / ((f_fov_degrees * 0.5 / 180.0 * 3.14159) as f64).tan();
        self.m[0][0] = f_aspect_ratio * f_fov_rad;
		self.m[1][1] = f_fov_rad;
		self.m[2][2] = f_far / (f_far - f_near);
		self.m[3][2] = (-f_far * f_near) / (f_far - f_near);
		self.m[2][3] = 1.0;
		self.m[3][3] = 0.0;
    }

    pub fn make_translation(&mut self, x: f64, y: f64, z: f64){
        self.m[0][0] = 1.0;
		self.m[1][1] = 1.0;
		self.m[2][2] = 1.0;
		self.m[3][3] = 1.0;
		self.m[3][0] = x;
		self.m[3][1] = y;
		self.m[3][2] = z;
    }

    pub fn make_point_at(&mut self, pos: &Vec3d, target: &Vec3d, up: &Vec3d){
        // Calculate new forward direction
		let mut new_forward = *target - *pos;
		new_forward.normalize();

		// Calculate new Up direction
		let a = new_forward*up.dot(&new_forward);
		let mut new_up = *up - a;
		new_up.normalize();

		// New Right direction is easy, its just cross product
		let new_right = new_up.cross(&new_forward);

		// Construct Dimensioning and Translation Matrix	
		self.m[0][0] = new_right.x;	    self.m[0][1] = new_right.y;	    self.m[0][2] = new_right.z;	    self.m[0][3] = 0.0;
		self.m[1][0] = new_up.x;		self.m[1][1] = new_up.y;		self.m[1][2] = new_up.z;		self.m[1][3] = 0.0;
		self.m[2][0] = new_forward.x;	self.m[2][1] = new_forward.y;	self.m[2][2] = new_forward.z;	self.m[2][3] = 0.0;
		self.m[3][0] = pos.x;			self.m[3][1] = pos.y;			self.m[3][2] = pos.z;			self.m[3][3] = 1.0;
    }
}