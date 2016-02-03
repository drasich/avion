use object;
use geometry;
use mesh;
use resource;
use std::f64::EPSILON;
use component::mesh_render;

use vec::{Vec3, Quat};

pub struct IntersectionRay
{
    pub hit : bool,
    pub inside : bool,
    pub position : Vec3,
    pub normal : Vec3
}

impl IntersectionRay
{
    pub fn new() -> IntersectionRay
    {
        IntersectionRay {
            hit : false,
            inside : false,
            position : Vec3::zero(),
            normal : Vec3::zero()
        }
    }
}

pub fn ray_object(ray : &geometry::Ray, o : &object::Object) -> IntersectionRay
{
    let out = IntersectionRay::new();

    match o.mesh_render {
        None => out,
        Some(ref mr) => {
            let wp = o.world_position();
            let wq = o.world_orientation();
            let ws = o.world_scale();

            //TODO
            //let ir_box = ray_box(ray, .... 
            let m = &mr.mesh;
            ray_mesh(ray, &*m.read().unwrap(), &wp, &wq, &ws)
        }
    }
}

// test for box first, and then mesh.
pub fn ray_mesh(
    ray : &geometry::Ray,
    m : &mesh::Mesh,
    position : &Vec3,
    rotation : &Quat,
    scale : &Vec3
    ) -> IntersectionRay
{
    if let Some(ref b) = m.aabox {
        let ir_box = intersection_ray_box(ray, b, position, rotation, scale);
        if !ir_box.hit {
            return IntersectionRay::new();
        }
    }

    let mut out = IntersectionRay::new();
    let r = geometry::Repere::new(*position, *rotation);

    let start = r.world_to_local(&ray.start);
    let direction = r.world_to_local(&(ray.direction + ray.start)) - start;

    let newray = geometry::Ray::new(start, direction);

    let vertices = match m.buffer_f32_get("position") {
        None => return out,
        Some(v) => v
    };

    fn get_vertex(v : &[f32], index: usize) -> Vec3
    {
        Vec3::new(
            v[index*3] as f64,
            v[index*3 + 1] as f64,
            v[index*3 + 2] as f64
            )
    }

    match m.buffer_u32_get("faces"){
        None => return out,
        Some(ref b) => {
            //for i in range_step(0, b.data.len(), 3) {
            for i in (0..b.data.len()).step_by(3) {
                let index = b.data[i] as usize;
                let v0 = get_vertex(&vertices.data, index) * *scale;
                //let v0 = get_vertex(&vertices.data, index).mul(*scale);
                let index = b.data[i+1] as usize;
                let v1 = get_vertex(&vertices.data, index) * *scale;
                //let v1 = get_vertex(&vertices.data, index).mul(*scale);
                let index = b.data[i+2] as usize;
                let v2 = get_vertex(&vertices.data, index)* *scale;
                //let v2 = get_vertex(&vertices.data, index).mul(*scale);

                let tri = geometry::Triangle::new(v0,v1,v2);

                out = ray_triangle(&newray, &tri, 1.0);
                if out.hit {
                    out.position = r.local_to_world(&out.position);
                    out.normal = rotation.rotate_vec3(&out.normal);
                    return out;
                }
            }
        }
    }

    out
}

pub fn ray_triangle(r : &geometry::Ray, t : &geometry::Triangle, min : f64) -> IntersectionRay
{
    let mut out = IntersectionRay::new();

    let e1 = t.v1 - t.v0;
    let e2 = t.v2 - t.v1;

    let n = e1 ^ e2;

    let dot = n.dot(&r.direction);
    let d = n.dot(&t.v0);
    let tt = d - n.dot(&r.start);

    if !(tt/dot <= min) {
        return out;
    }

    let tt = tt/dot;
    let p = r.start + (r.direction * tt);

    let a0;
    let a1;
    let a2;
    let b0;
    let b1;
    let b2;

    if n.x.abs() > n.y.abs() {
        if n.x.abs() > n.z.abs() {
            a0 = p.y - t.v0.y;
            a1 = t.v1.y - t.v0.y;
            a2 = t.v2.y - t.v0.y;

            b0 = p.z - t.v0.z;
            b1 = t.v1.z - t.v0.z;
            b2 = t.v2.z - t.v0.z;
        } else {
            a0 = p.x - t.v0.x;
            a1 = t.v1.x - t.v0.x;
            a2 = t.v2.x - t.v0.x;

            b0 = p.y - t.v0.y;
            b1 = t.v1.y - t.v0.y;
            b2 = t.v2.y - t.v0.y;
        }
    } else {
        if n.y.abs() > n.z.abs() {
            a0 = p.x - t.v0.x;
            a1 = t.v1.x - t.v0.x;
            a2 = t.v2.x - t.v0.x;

            b0 = p.z - t.v0.z;
            b1 = t.v1.z - t.v0.z;
            b2 = t.v2.z - t.v0.z;
        } else {
            a0 = p.x - t.v0.x;
            a1 = t.v1.x - t.v0.x;
            a2 = t.v2.x - t.v0.x;

            b0 = p.y - t.v0.y;
            b1 = t.v1.y - t.v0.y;
            b2 = t.v2.y - t.v0.y;
        }
    }

    let temp = a1*b2 - b1*a2;

    if !(temp != 0f64) {
        return out;
    }

    let temp = 1.0 /temp;
    let alpha = (a0 * b2 - b0 * a2) * temp;
    if !(alpha >= 0.0) {
        return out;
    }

    let beta = (a1 * b0 - b1 * a0) * temp;
    if !(beta >= 0.0) {
        return out;
    }

    let gamma = 1.0 - alpha - beta;
    if !(gamma >= 0.0) {
        return out;
    }

    out.hit = true;
    out.position = p;

    out
}

pub fn intersection_ray_aabox(ray : &geometry::Ray, abox : &geometry::AABox) -> IntersectionRay
{
  let mut out = IntersectionRay::new();
  let mut xt;
  let mut xn = 0f64;

  if ray.start.x < abox.min.x {
      xt = abox.min.x - ray.start.x;
      if xt > ray.direction.x {
          out.inside = false;
          return out;
      }
      xt /= ray.direction.x;
      out.inside = false;
      xn = -1f64;
  } else if ray.start.x > abox.max.x {
      xt = abox.max.x - ray.start.x;
      if xt < ray.direction.x {
          out.inside = false;
          return out;
      }
      xt /= ray.direction.x;
      out.inside = false;
      xn = 1f64;
  } else {
      xt = -1f64;
  }

  let mut yt;
  let mut yn = 0f64;
  if ray.start.y < abox.min.y {
      yt = abox.min.y - ray.start.y;
      if yt > ray.direction.y {
          out.inside = false;
          return out;
      }
      yt /= ray.direction.y;
      out.inside = false;
      yn = -1f64;
  } else if ray.start.y > abox.max.y {
      yt = abox.max.y - ray.start.y;
      if yt < ray.direction.y {
          out.inside = false;
          return out;
      }
      yt /= ray.direction.y;
      out.inside = false;
      yn = 1f64;
  } else {
      yt = -1f64;
  }

  let mut zt;
  let mut zn = 0f64;
  if ray.start.z < abox.min.z {
      zt = abox.min.z - ray.start.z;
      if zt > ray.direction.z {
          out.inside = false;
          return out;
      }
      zt /= ray.direction.z;
      out.inside = false;
      zn = -1f64;
  } else if ray.start.z > abox.max.z {
      zt = abox.max.z - ray.start.z;
      if zt < ray.direction.z {
          out.inside = false;
          return out;
      }
      zt /= ray.direction.z;
      out.inside = false;
      zn = 1f64;
  } else {
      zt = -1f64;
  }

  if out.inside {
      out.hit = true;
      return out;
  }

  enum Plane {
      YZ,
      XZ,
      XY
  }

  let mut plane = Plane::YZ;
  let mut t = xt;
  if yt > t {
      plane = Plane::XZ;
      t = yt;
  }
  if zt > t {
      plane = Plane::XY;
      t = zt;
  }

  //printf("position %f, %f, %f \n", ir.position.x, ir.position.y, ir.position.z);

  match plane {
      Plane::YZ => { // yz plane
          let y = ray.start.y + ray.direction.y*t;
          if y < abox.min.y - EPSILON || y > abox.max.y + EPSILON { 
              out.inside = false;
              return out; 
          }
          let z = ray.start.z + ray.direction.z*t;
          if z < abox.min.z - EPSILON || z > abox.max.z + EPSILON {
              out.inside = false;
              return out;
          }
          out.normal.x = xn;
      },
      Plane::XZ => { //xz plane
          let x = ray.start.x + ray.direction.x*t;
          if x < abox.min.x - EPSILON || x > abox.max.x + EPSILON { 
              out.inside = false; return out; }
          let z = ray.start.z + ray.direction.z*t;
          if z < abox.min.z - EPSILON || z > abox.max.z + EPSILON {
              out.inside = false; return out; }

          out.normal.y = yn;
      },
      Plane::XY => {
          let x = ray.start.x + ray.direction.x*t;
          if x < abox.min.x - EPSILON || x > abox.max.x + EPSILON {
              out.inside = false; return out; }
          let y = ray.start.y + ray.direction.y*t;
          if y < abox.min.y - EPSILON || y > abox.max.y + EPSILON { 
              out.inside = false; return out; }

          out.normal.y = zn;
      },
  }

  out.position = ray.start + (ray.direction *t);
  out.hit = true;
 
  return out;
}

pub fn intersection_ray_box(
    ray : &geometry::Ray,
    abox : &geometry::AABox,
    position : &Vec3,
    rotation : &Quat,
    scale : &Vec3)
-> IntersectionRay
{
  let r = geometry::Repere::new(*position, *rotation);
  //transform the ray in box/object coord
  let start = r.world_to_local(&ray.start);
  let direction = r.world_to_local(&(ray.direction + ray.start)) - start;
  let newray = geometry::Ray::new(start, direction);

  let box_rep = geometry::AABox::new(
      abox.min * *scale,
      abox.max * *scale);

  let mut ir = intersection_ray_aabox(&newray, &box_rep);

  //transform back
  ir.position = r.local_to_world(&ir.position);
  ir.normal = rotation.rotate_vec3(&ir.normal);

  return ir;
}

pub fn intersection_ray_plane(ray : &geometry::Ray, plane : &geometry::Plane) -> IntersectionRay
{
  let dn = ray.direction.dot(&plane.normal);
  let mut out = IntersectionRay::new();

  if dn != 0f64 {
      let d = plane.normal.dot(&plane.point);
      let p0n = ray.start.dot(&plane.normal);
      let t = (d - p0n) / dn;
      out.hit = true;
      out.position = ray.start + (ray.direction * t);
  }

  return out;
}

pub fn is_position_in_plane(p : &geometry::Plane, v : Vec3) -> bool
{
  /*
  printf("testing plane \n");
  printf(" point : %f, %f, %f\n", p.point.x, p.point.y, p.point.z);
  printf(" normal : %f, %f, %f\n", p.normal.x, p.normal.y, p.normal.z);
  printf(" with v : %f, %f, %f\n", v.x, v.y, v.z);
  */
  let pos = v - p.point;
  //printf(" pos is : %f, %f, %f\n", pos.x, pos.y, pos.z);
  let dot = pos.dot(&p.normal);
  //printf("dot is : %f \n", dot);
  return dot >= 0f64;
}


pub fn is_position_in_planes(planes : &[geometry::Plane], v : Vec3) -> bool
{
    for p in planes {
        if !is_position_in_plane(p, v) {
            return false;
        }
    }

  return true;
}

pub fn planes_is_box_in_allow_false_positives(planes : &[geometry::Plane], b : &geometry::OBox) -> bool
{
    let mut out;
    let mut inn;

    // for each plane do ...
    //for(i=0; i < nb_planes; ++i) {
    for plane in planes {

        out=0;
        inn=0;
        // for each corner of the box do ...
        // get out of the cycle as soon as a box has corners
        // both inside and out of the frustum
        //for (k = 0; k < 8 && (inn==0 || out==0); k++) {
        for pos in &b.v {

            // is the corner on the good or bad side of the plane
            if is_position_in_plane(plane, *pos) {
                inn = inn + 1;
            }
            else {
                out = out + 1;
            }

            if !(inn==0 || out==0) {
                break;
            }
        }
        //if all corners are out
        if inn == 0 {
            return false;
        }
    }

    return true;
}






pub fn is_object_in_planes(planes : &[geometry::Plane], o : &object::Object) 
    -> bool
{
    let m = match o.mesh_render {
        None => 
            return is_position_in_planes(planes, o.position),
        Some(ref mr) => {
            mr.mesh.read().unwrap()
        }
    };

    //first test the box and then test the object/mesh
    if let Some(ref aa) = m.aabox {
        let b = aa.to_obox(o.position, o.orientation.as_quat(), o.scale);
        if !planes_is_box_in_allow_false_positives(planes, &b) {
            return false;
        }
    }

    let mut new_planes = planes.clone().to_vec();

    let r = geometry::Repere { origin : o.position, rotation : o.orientation.as_quat()};
    let iq = r.rotation.conj();

    for p in &mut new_planes {
        let point = p.point;
        p.point =  r.world_to_local(&point);
        p.normal = r.world_to_local(&(p.normal + point));
        p.normal = p.normal - p.point;
    }


    let vertices = match m.buffer_f32_get("position") {
        None => return is_position_in_planes(planes, o.position),
        Some(v) => v
    };

    let faces = match m.buffer_u32_get("faces") {
        None => {
            println!("TODO handle when there is no indices");
            return is_position_in_planes(planes, o.position);
        },
        Some(f) => f
    };

    fn get_vertex(v : &[f32], index: usize) -> Vec3
    {
        Vec3::new(
            v[index*3] as f64,
            v[index*3 + 1] as f64,
            v[index*3 + 2] as f64
            )
    }

    //for i in range_step(0, faces.data.len(), 3) {
    for i in (0..faces.data.len()).step_by(3) {
        let index = faces.data[i] as usize;
        let v0 = get_vertex(&vertices.data, index) * o.scale;
        let index = faces.data[i+1] as usize;
        let v1 = get_vertex(&vertices.data, index) * o.scale;
        let index = faces.data[i+2] as usize;
        let v2 = get_vertex(&vertices.data, index) * o.scale;

        let tri = geometry::Triangle::new(v0,v1,v2);

        if planes_is_in_triangle(&new_planes, &tri) {
            return true;
        }
    }

    return false;
}

pub fn planes_is_in_triangle(planes : &[geometry::Plane], t : &geometry::Triangle) -> bool
{
  let mut point_is_in = true;
  //for (i = 0; i< nb_planes; i++) {
  for p in planes {
    if !is_position_in_plane(p, t.v0) {
      point_is_in = false;
      break;
    }
  }

  if point_is_in {
      return true;
  }

  point_is_in = true;
  //for (i = 0; i< nb_planes; i++) {
  for p in planes {
    if !is_position_in_plane(p, t.v1) {
      point_is_in = false;
      break;
    }
  }

  if point_is_in {
      return true;
  }

  point_is_in = true;
  for p in planes {
    if !is_position_in_plane(p, t.v2) {
      point_is_in = false;
      break;
    }
  }

  if point_is_in {
      return true;
  }

  //TODO stuff here when points are not inside the plan
  // like axis separating test

  //for (i = 0; i< nb_planes; i++) {
  let mut i = 0;
  for p in planes {
    let ipt = _intersection_plane_triangle(p, t);
    //test everything but the current plane
    if ipt.intersect {
      
      let otherplane = if i % 2 == 0 {i+1} else {i-1};
      //near, far, up, down, right, left

      if _check_inter(planes, i, otherplane, &ipt.segment) {
          return true;
      }
    }

    i = i + 1;
  }

  return false;
}

pub struct IntersectionPlaneTriangle {
  intersect : bool,
  segment : geometry::Segment
}



fn _intersection_plane_triangle(p : &geometry::Plane, t : &geometry::Triangle) -> IntersectionPlaneTriangle
{
  let mut ipt = IntersectionPlaneTriangle {
      intersect : false, 
      segment : geometry::Segment{ p0 : Vec3::zero(), p1 : Vec3::zero()} };

  let b0 = is_position_in_plane(p, t.v0);
  let b1 = is_position_in_plane(p, t.v1);
  let b2 = is_position_in_plane(p, t.v2);

  let mut r0 = geometry::Ray::new(Vec3::zero(), Vec3::zero());
  let mut r1 = geometry::Ray::new(Vec3::zero(), Vec3::zero());

  if b0 != b1 {
      ipt.intersect = true;
      if b0 != b2 {
          //bo is alone
          r0.start = t.v0;
          r0.direction = t.v1 - t.v0;
          r1.start = t.v0;
          r1.direction = t.v2 - t.v0;
    }
    else {
      //b1 is alone
      r0.start = t.v1;
      r0.direction = t.v0 - t.v1;
      r1.start = t.v1;
      r1.direction = t.v2 - t.v1;
    }
  }
  else if b0 != b2 {
    ipt.intersect = true;
    //b2 is alone
    r0.start = t.v2;
    r0.direction = t.v0 - t.v2;
    r1.start = t.v2;
    r1.direction = t.v1 - t.v2;
  }

  if ipt.intersect {
    let ir0 = intersection_ray_plane(&r0, p);
    let ir1 = intersection_ray_plane(&r1, p);
    ipt.segment.p0 = ir0.position;
    ipt.segment.p1 = ir1.position;
  }

  return ipt;
}


fn _check_inter(
    planes : &[geometry::Plane],
    notthisplane : usize,
    notthisplaneeither : usize,
    s : &geometry::Segment) -> bool
{
  for (i,p) in planes.iter().enumerate() {
    if i == notthisplane || i == notthisplaneeither {
        continue;
    }

    let planedot = planes[i].normal.dot(&p.point);
    let s0dot = p.normal.dot(&s.p0);
    let s1dot = p.normal.dot(&s.p1);

    if s0dot >= planedot || s1dot >= planedot {
        continue;
    }
    else {
        return false;
    }
  }
  //we tested all the planes and can return true
  true
}


