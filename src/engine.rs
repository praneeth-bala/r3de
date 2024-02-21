use eframe::egui;
use std::sync::{ Arc, Mutex };
use std::sync::atomic::Ordering;
use std::time::Instant;
use std::cmp::Ordering as cmpOrdering;

use crate::objs::{ GUIState, DisplayBuffers, Matrix4x4, Mesh, Tri, Vec3d };

pub struct Engine {
    state: Arc<Mutex<GUIState>>,
    buffers: DisplayBuffers,
    meshes: Vec<Mesh>,
    mat_proj: Matrix4x4,
    begin_time: Instant,
    v_camera: Vec3d,
}

impl Engine{

    pub fn new(state: Arc<Mutex<GUIState>>, buffers_copy: &DisplayBuffers) -> Self{
        let buffers = DisplayBuffers::from(buffers_copy);

        let mut meshes = Vec::with_capacity(1);
        let mut sample = Vec::with_capacity(12);
        sample.push(Tri::new(vec![Vec3d::new(0.0, 0.0, 0.0),Vec3d::new(0.0, 1.0, 0.0), Vec3d::new(1.0, 1.0, 0.0)]));
        sample.push(Tri::new(vec![Vec3d::new(0.0, 0.0, 0.0),Vec3d::new(1.0, 1.0, 0.0), Vec3d::new(1.0, 0.0, 0.0)]));
        sample.push(Tri::new(vec![Vec3d::new(1.0, 0.0, 0.0),Vec3d::new(1.0, 1.0, 0.0), Vec3d::new(1.0, 1.0, 1.0)]));
        sample.push(Tri::new(vec![Vec3d::new(1.0, 0.0, 0.0),Vec3d::new(1.0, 1.0, 1.0), Vec3d::new(1.0, 0.0, 1.0)]));
        sample.push(Tri::new(vec![Vec3d::new(1.0, 0.0, 1.0),Vec3d::new(1.0, 1.0, 1.0), Vec3d::new(0.0, 1.0, 1.0)]));
        sample.push(Tri::new(vec![Vec3d::new(1.0, 0.0, 1.0),Vec3d::new(0.0, 1.0, 1.0), Vec3d::new(0.0, 0.0, 1.0)]));
        sample.push(Tri::new(vec![Vec3d::new(0.0, 0.0, 1.0),Vec3d::new(0.0, 1.0, 1.0), Vec3d::new(0.0, 1.0, 0.0)]));
        sample.push(Tri::new(vec![Vec3d::new(0.0, 0.0, 1.0),Vec3d::new(0.0, 1.0, 0.0), Vec3d::new(0.0, 0.0, 0.0)]));
        sample.push(Tri::new(vec![Vec3d::new(0.0, 1.0, 0.0),Vec3d::new(0.0, 1.0, 1.0), Vec3d::new(1.0, 1.0, 1.0)]));
        sample.push(Tri::new(vec![Vec3d::new(0.0, 1.0, 0.0),Vec3d::new(1.0, 1.0, 1.0), Vec3d::new(1.0, 1.0, 0.0)]));
        sample.push(Tri::new(vec![Vec3d::new(1.0, 0.0, 1.0),Vec3d::new(0.0, 0.0, 1.0), Vec3d::new(0.0, 0.0, 0.0)]));
        sample.push(Tri::new(vec![Vec3d::new(1.0, 0.0, 1.0),Vec3d::new(0.0, 0.0, 0.0), Vec3d::new(1.0, 0.0, 0.0)]));
        meshes.push(Mesh::new(sample));
        
        // for m in meshes.iter_mut() {
        //     for t in &mut m.tris {
        //         t.p[0].z += 3.0;
        //         t.p[1].z += 3.0;
        //         t.p[2].z += 3.0;
        //     }
        // }

        // Projection Matrix
		let f_near = 0.1;
		let f_far = 1000.0;
		let f_fov = 90.0;
		let f_aspect_ratio = buffers.buf_size[0] as f64 / buffers.buf_size[1] as f64;
		let f_fov_rad = 1.0 / ((f_fov * 0.5 / 180.0 * 3.14159) as f64).tan();

        let mut mat_proj = Matrix4x4::new(vec![vec![0.0; 4]; 4]);
		mat_proj.m[0][0] = f_aspect_ratio * f_fov_rad;
		mat_proj.m[1][1] = f_fov_rad;
		mat_proj.m[2][2] = f_far / (f_far - f_near);
		mat_proj.m[3][2] = (-f_far * f_near) / (f_far - f_near);
		mat_proj.m[2][3] = 1.0;
		mat_proj.m[3][3] = 0.0;

        Self { 
            state,
            buffers,
            meshes,
            mat_proj,
            begin_time: Instant::now(),
            v_camera: Vec3d::new(0.0, 0.0, 0.0),
        }
    }

    fn put_pixel(&self, x: i64, y: i64, color: &egui::Color32, pixels: &mut Vec<egui::Color32>){
        let hlen = self.buffers.buf_size[0];
        let vlen = self.buffers.buf_size[1];
        if x<hlen as i64 && y<vlen as i64 {
            pixels[(y as usize)*hlen+(x as usize)] = *color;
        }
    }

    fn draw_line(&self, x1: i64, y1: i64, x2: i64, y2: i64, color: &egui::Color32, pixels: &mut Vec<egui::Color32>){
        let (mut x, mut y, dx, dy, dx1, dy1, mut px, mut py, xe, ye);
        dx = x2 - x1; dy = y2 - y1;
        dx1 = dx.abs(); dy1 = dy.abs();
        px = 2 * dy1 - dx1;	py = 2 * dx1 - dy1;
        if dy1 <= dx1
        {
            if dx >= 0
                { x = x1; y = y1; xe = x2; }
            else
                { x = x2; y = y2; xe = x1;}
    
            // Draw(x, y, c, col);
            self.put_pixel(x, y, color, pixels);
            loop
            {
                if x>=xe {
                    break;
                }
                x = x + 1;
                if px<0 {
                    px = px + 2 * dy1;
                }
                else
                {
                    if (dx<0 && dy<0) || (dx>0 && dy>0){
                        y = y + 1;
                    }
                    else {
                        y = y - 1;
                    }
                    px = px + 2 * (dy1 - dx1);
                }
                // Draw(x, y, c, col);
                self.put_pixel(x, y, color, pixels);
            }
        }
        else
        {
            if dy >= 0
                { x = x1; y = y1; ye = y2; }
            else
                { x = x2; y = y2; ye = y1; }
    
            // Draw(x, y, c, col);
            self.put_pixel(x, y, color, pixels);
            loop
            {
                if y>=ye {
                    break;
                }
                y = y + 1;
                if py <= 0 {
                    py = py + 2 * dx1;
                }
                else
                {
                    if (dx<0 && dy<0) || (dx>0 && dy>0) {
                        x = x + 1;
                    }
                    else {
                        x = x - 1;
                    }
                    py = py + 2 * (dx1 - dy1);
                }
                // Draw(x, y, c, col);
                self.put_pixel(x, y, color, pixels);
            }
        }
    }

    fn draw_hline(&self, mut sx: i64, mut ex: i64, ny: i64, color: &egui::Color32, pixels: &mut Vec<egui::Color32>){
        if sx > ex {
            let t = sx;
            sx = ex;
            ex = t;
        }
        for i in sx..=ex {
            self.put_pixel(i, ny, color, pixels);
        }
    }

    fn draw_triangle(&self, x1: i64, y1: i64, x2: i64, y2: i64, x3: i64, y3: i64, color: &egui::Color32, pixels: &mut Vec<egui::Color32>){
        self.draw_line(x1, y1, x2, y2, color, pixels);
        self.draw_line(x2, y2, x3, y3, color, pixels);
        self.draw_line(x1, y1, x3, y3, color, pixels);
    }

    fn fill_bottom_flat_triangle(&self, x1: i64, y1: i64, x2: i64, y2: i64, x3: i64, y3: i64, color: &egui::Color32, pixels: &mut Vec<egui::Color32>) {
        let invslope1 = ((x2 - x1) as f64) / ((y2 - y1) as f64);
        let invslope2 = ((x3 - x1) as f64) / ((y3 - y1) as f64);
        
        let mut curx1 = x1 as f64;
        let mut curx2 = x1 as f64;

        let mut scanline_y = y1;
        loop {
            if scanline_y > y2 {
                break;
            }
            self.draw_hline(curx1 as i64, curx2 as i64, scanline_y, color, pixels);
            curx1 += invslope1;
            curx2 += invslope2;
            scanline_y += 1;
        }
    }

    fn fill_top_flat_triangle(&self, x1: i64, y1: i64, x2: i64, y2: i64, x3: i64, y3: i64, color: &egui::Color32, pixels: &mut Vec<egui::Color32>){
        let invslope1 = ((x3 - x1) as f64) / ((y3 - y1) as f64);
        let invslope2 = ((x3 - x2) as f64) / ((y3 - y2) as f64);
        
        let mut curx1 = x3 as f64;
        let mut curx2 = x3 as f64;

        let mut scanline_y = y3;
        loop {
            if scanline_y <= y1 {
                break;
            }
            self.draw_hline(curx1 as i64, curx2 as i64, scanline_y, color, pixels);
            curx1 -= invslope1;
            curx2 -= invslope2;
            scanline_y -= 1;
        }
    }

    fn fill_triangle(&self, mut x1: i64, mut y1: i64, mut x2: i64, mut y2: i64, mut x3: i64, mut y3: i64, color: &egui::Color32, pixels: &mut Vec<egui::Color32>) {
        /* at first sort the three vertices by y-coordinate ascending so v1 is the topmost vertice */
        let mut temp_arr = [[y1, x1], [y2, x2], [y3, x3]];
        temp_arr.sort_by(|a, b| {
            if a[0] >= b[0] {
                cmpOrdering::Greater
            }
            else {
                cmpOrdering::Less
            }
        });
        (x1, y1, x2, y2, x3, y3) = (temp_arr[0][1], temp_arr[0][0], temp_arr[1][1], temp_arr[1][0], temp_arr[2][1], temp_arr[2][0]);
        if y2 == y3 {
            self.fill_bottom_flat_triangle(x1, y1, x2, y2, x3, y3, color, pixels);
        }
        else if y1 == y2 {
            self.fill_top_flat_triangle(x1, y1, x2, y2, x3, y3, color, pixels);
        }
        else {
            let x4 = (x1 as f64 + ((y2 - y1) as f64 / (y3 - y1) as f64) * ((x3 - x1) as f64)) as i64;
            let y4 = y2;
            self.fill_bottom_flat_triangle(x1, y1, x2, y2, x4, y4, color, pixels);
            self.fill_top_flat_triangle(x2, y2, x4, y4, x3, y3, color, pixels);
        }
        
    }

    fn render(&mut self, inp_buffer_index: usize){
        let mut pixels = self.buffers.bufs[inp_buffer_index].lock().unwrap();
        pixels.clear();
        pixels.resize(self.buffers.buf_size[0]*self.buffers.buf_size[1], egui::Color32::from_rgba_premultiplied(0, 0, 0, 255,));
        
        let mut ftheta: f64 = self.begin_time.elapsed().as_secs_f64();
        // ftheta = 10.0;
        let mut mat_rot_z = Matrix4x4::new(vec![vec![0.0; 4]; 4]);
        mat_rot_z.m[0][0] = ftheta.cos();
		mat_rot_z.m[0][1] = ftheta.sin();
		mat_rot_z.m[1][0] = -ftheta.sin();
		mat_rot_z.m[1][1] = ftheta.cos();
		mat_rot_z.m[2][2] = 1.0;
		mat_rot_z.m[3][3] = 1.0;

        ftheta *= 0.5;
        let mut mat_rot_x = Matrix4x4::new(vec![vec![0.0; 4]; 4]);
        mat_rot_x.m[0][0] = 1.0;
		mat_rot_x.m[1][1] = ftheta.cos();
		mat_rot_x.m[1][2] = ftheta.sin();
		mat_rot_x.m[2][1] = -ftheta.sin();
		mat_rot_x.m[2][2] = ftheta.cos();
		mat_rot_x.m[3][3] = 1.0;

        let mut mat_trans = Matrix4x4::new(vec![vec![0.0; 4]; 4]);
        mat_trans.m[0][0] = 1.0;
		mat_trans.m[1][1] = 1.0;
        mat_trans.m[2][2] = 1.0;
        mat_trans.m[3][2] = 3.0;
        mat_trans.m[3][3] = 1.0;

        for m in self.meshes.iter() {
            for t in &m.tris {
                let tri_rotated = mat_rot_z.mul_mat_tri(&t);
                let tri_rotated = mat_rot_x.mul_mat_tri(&tri_rotated);
                let tri_translated = mat_trans.mul_mat_tri(&tri_rotated);
                // Use Cross-Product to get surface normal
                let (mut normal, mut line1, mut line2) = (Vec3d::new(0.0, 0.0, 0.0), Vec3d::new(0.0, 0.0, 0.0), Vec3d::new(0.0, 0.0, 0.0));
                line1.x = tri_translated.p[1].x - tri_translated.p[0].x;
                line1.y = tri_translated.p[1].y - tri_translated.p[0].y;
                line1.z = tri_translated.p[1].z - tri_translated.p[0].z;

                line2.x = tri_translated.p[2].x - tri_translated.p[0].x;
                line2.y = tri_translated.p[2].y - tri_translated.p[0].y;
                line2.z = tri_translated.p[2].z - tri_translated.p[0].z;

                normal.x = line1.y * line2.z - line1.z * line2.y;
                normal.y = line1.z * line2.x - line1.x * line2.z;
                normal.z = line1.x * line2.y - line1.y * line2.x;

                let den = (normal.x*normal.x + normal.y*normal.y + normal.z*normal.z).sqrt();
                normal.x /= den;
                normal.y /= den;
                normal.z /= den;

                if  normal.x * (tri_translated.p[0].x - self.v_camera.x) + 
                    normal.y * (tri_translated.p[0].y - self.v_camera.y) +
                    normal.z * (tri_translated.p[0].z - self.v_camera.z) < 0.0 {

                    let mut tri_projected = self.mat_proj.mul_mat_tri(&tri_translated);
                    for i in 0..3 {
                        tri_projected.p[i].x += 1.0;
                        tri_projected.p[i].y += 1.0;
                        tri_projected.p[i].x *= 0.5*(self.buffers.buf_size[0] as f64);
                        tri_projected.p[i].y *= 0.5*(self.buffers.buf_size[1] as f64);
                    }
                    self.fill_triangle(tri_projected.p[0].x as i64, tri_projected.p[0].y as i64, tri_projected.p[1].x as i64, tri_projected.p[1].y as i64, tri_projected.p[2].x as i64, tri_projected.p[2].y as i64, &egui::Color32::from_rgba_premultiplied(255, 255, 255, 255,), &mut pixels);
                    self.draw_triangle(tri_projected.p[0].x as i64, tri_projected.p[0].y as i64, tri_projected.p[1].x as i64, tri_projected.p[1].y as i64, tri_projected.p[2].x as i64, tri_projected.p[2].y as i64, &egui::Color32::from_rgba_premultiplied(0, 0, 0, 255,), &mut pixels);
                }
            }
        }
        drop(pixels);
    }
    
    pub fn lo(&mut self) {
        loop {
            let trip_state_lock = self.buffers.trip_state.lock().unwrap();
            let inp_buffer_index = trip_state_lock[1];
            drop(trip_state_lock);
            
            self.render(inp_buffer_index as usize);

            let mut trip_state_lock = self.buffers.trip_state.lock().unwrap();
            (trip_state_lock[0], trip_state_lock[1]) = (trip_state_lock[1], trip_state_lock[0]);
            self.buffers.stale.store(false, Ordering::Relaxed);
            drop(trip_state_lock);

            let state_lock = self.state.lock().unwrap();
            let ctx = &state_lock.ctx;
            match ctx {
                Some(x) => x.request_repaint(),
                None => panic!("error in Option<>"),
            }
        }
    }
}