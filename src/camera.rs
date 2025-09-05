use crate::math::Vec3;

pub struct OrbitCam {
   pub target: Vec3,
   pub yaw: f32,
   pub pitch: f32,
   pub dist: f32,
   pub fov_deg: f32,
}
impl OrbitCam {
   pub fn eye(&self) -> Vec3 {
      let cp=self.pitch.cos(); let sp=self.pitch.sin();
      let sy=self.yaw.sin(); let cy=self.yaw.cos();
      let x=self.dist*cp*sy; let y=self.dist*sp; let z=self.dist*cp*cy;
      self.target.add(Vec3::new(x,y,z))
   }
   pub fn basis(&self) -> (Vec3, Vec3, Vec3) {
      let eye = self.eye();
      let fwd = self.target.sub(eye).normalize();
      let up  = Vec3::new(0.0,1.0,0.0);
      let right = fwd.cross(up).normalize();
      let up2 = right.cross(fwd).normalize();
      (fwd, right, up2)
   }
}
