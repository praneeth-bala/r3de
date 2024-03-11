use eframe::egui;
use std::sync::{ Arc, Mutex };
use std::sync::atomic::Ordering;
use std::time::Instant;
use std::cmp::Ordering as cmpOrdering;
use std::fs::File;
use std::io::{BufRead, BufReader};

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

        let meshes = Vec::with_capacity(1);

        // Projection Matrix
		let f_near = 0.1;
		let f_far = 1000.0;
		let f_fov = 90.0;
		let f_aspect_ratio = buffers.buf_size[0] as f64 / buffers.buf_size[1] as f64;

        let mut mat_proj = Matrix4x4::new(vec![vec![0.0; 4]; 4]);
        mat_proj.make_projection(f_fov, f_aspect_ratio, f_near, f_far);

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

    fn load_from_object_file(&mut self, fpath: String){
        let file = File::open(fpath).expect("fopen error");
        let mut reader = BufReader::new(file);

        let mut vec_cache = Vec::with_capacity(0);
        let mut sample = Vec::with_capacity(0);
        
        let mut line = String::new();
        loop {
            let bytes_read = reader.read_line(&mut line).expect("read error");
            if bytes_read == 0 {
                break;
            }
            let chars_as_vec = line.chars().collect::<Vec<char>>();
            if chars_as_vec[0]=='v' {
                let trimmed: Vec<&str> = line.trim().split_whitespace().collect();
                vec_cache.push(Vec3d::new(trimmed[1].parse::<f64>().unwrap(), trimmed[2].parse::<f64>().unwrap(), trimmed[3].parse::<f64>().unwrap()));    
            }
            else if chars_as_vec[0]=='f'{
                let trimmed: Vec<&str> = line.trim().split_whitespace().collect();
                sample.push(Tri::new(vec![vec_cache[trimmed[1].parse::<usize>().unwrap()-1], vec_cache[trimmed[2].parse::<usize>().unwrap()-1], vec_cache[trimmed[3].parse::<usize>().unwrap()-1]]));
            }
            line.clear();
        }
        self.meshes.push(Mesh::new(sample));
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

    fn to_screen_space(&self, tri: &mut Tri){
        for i in 0..3 {
            tri.p[i].x += 1.0;
            tri.p[i].y += 1.0;
            // if tri.p[i].x<0.0 {tri.p[i].x = 0.0}
            // if tri.p[i].y<0.0 {tri.p[i].y = 0.0}
            // if tri.p[i].x>2.0 {tri.p[i].x = 2.0}
            // if tri.p[i].y>2.0 {tri.p[i].y = 2.0}
            tri.p[i].x *= 0.5*(self.buffers.buf_size[0] as f64);
            tri.p[i].y *= 0.5*(self.buffers.buf_size[1] as f64);
        }
    }

    fn render(&mut self, inp_buffer_index: usize){
        let mut pixels = self.buffers.bufs[inp_buffer_index].lock().unwrap();
        pixels.clear();
        pixels.resize(self.buffers.buf_size[0]*self.buffers.buf_size[1], egui::Color32::from_rgba_premultiplied(0, 0, 0, 255,));
        
        let mut ftheta: f64 = self.begin_time.elapsed().as_secs_f64();
        // ftheta = 10.0;
        let mut mat_rot_z = Matrix4x4::new(vec![vec![0.0; 4]; 4]);
        mat_rot_z.make_rotation_z(ftheta);

        ftheta *= 0.5;
        let mut mat_rot_x = Matrix4x4::new(vec![vec![0.0; 4]; 4]);
        mat_rot_x.make_rotation_x(ftheta);

        let mut mat_trans = Matrix4x4::new(vec![vec![0.0; 4]; 4]);
        mat_trans.make_translation(0.0, 0.0, 8.0);

        let light = Vec3d::new(0.0, 0.0, -1.0);

        for m in self.meshes.iter() {
            let mut triangles_to_raster = Vec::with_capacity(m.tris.len());
            for t in &m.tris {
                let tri_rotated = mat_rot_z.mul_mat_tri(&t);
                let tri_rotated = mat_rot_x.mul_mat_tri(&tri_rotated);
                let tri_translated = mat_trans.mul_mat_tri(&tri_rotated);

                let triangles_to_project = tri_translated.triangle_clip_against_plane(&Vec3d::new( 0.0, 0.0, 1.0 ), &Vec3d::new( 0.0, 0.0, 1.0 ));

                for tri_translated in triangles_to_project {
                    // Use Cross-Product to get surface normal
                    let normal = tri_translated.get_normal();

                    if  normal.dot(&(tri_translated.p[0] - self.v_camera)) < 0.0 {

                        let mut tri_projected = self.mat_proj.mul_mat_tri(&tri_translated);
                        self.to_screen_space(&mut tri_projected);
                        
                        let shade = ((normal.x*light.x + normal.y*light.y + normal.z*light.z)*(255.0)) as u8;
                        tri_projected.shade = shade;

                        triangles_to_raster.push(tri_projected);
                    }
                }                
            }
            triangles_to_raster.sort_by(|a, b| {
                let za = (a.p[0].z+a.p[1].z+a.p[2].z)/3.0;
                let zb = (b.p[0].z+b.p[1].z+b.p[2].z)/3.0;
                if za<zb {
                    cmpOrdering::Greater
                }
                else {
                    cmpOrdering::Less
                }
            });
            for tri_projected in triangles_to_raster.iter() {
                self.fill_triangle(tri_projected.p[0].x as i64, tri_projected.p[0].y as i64, tri_projected.p[1].x as i64, tri_projected.p[1].y as i64, tri_projected.p[2].x as i64, tri_projected.p[2].y as i64, &egui::Color32::from_rgba_premultiplied(tri_projected.shade, tri_projected.shade, tri_projected.shade, 255), &mut pixels);
                self.draw_triangle(tri_projected.p[0].x as i64, tri_projected.p[0].y as i64, tri_projected.p[1].x as i64, tri_projected.p[1].y as i64, tri_projected.p[2].x as i64, tri_projected.p[2].y as i64, &egui::Color32::from_rgba_premultiplied(0, 0, 0, 255,), &mut pixels);
            }
        }
        drop(pixels);
    }
    
    pub fn lo(&mut self) {

        self.load_from_object_file("./teapot.obj".to_string());

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