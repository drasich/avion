use object;
use geometry;
use mesh_render;
use mesh;
use resource;
use std::iter::range_step;
use std::num::abs;

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

pub fn ray_object(ray : geometry::Ray, o : &object::Object) -> IntersectionRay
{
    let mut out = IntersectionRay::new();

    match o.mesh_render {
        None => return out,
        Some(ref mr) => {
            match mr.mesh.resource {
                resource::ResData(ref m) => {
                    let wp = o.world_position();
                    let wq = o.world_orientation();
                    let ws = o.world_scale();
                    println!("object {} : {}, {}, {}", o.name, wp, wq, ws);

                    //TODO
                    //let ir_box = ray_box(ray, .... 
                    return ray_mesh(ray, &*m.read(), &wp, &wq, &ws);
                },
                _ => return out
            }
        }
    }

    out
}

pub fn ray_mesh(
    ray : geometry::Ray,
    m : &mesh::Mesh,
    position : &Vec3,
    rotation : &Quat,
    scale : &Vec3
    ) -> IntersectionRay
{
    let mut out = IntersectionRay::new();
    let r = geometry::Repere::new(*position, *rotation);

    let start = r.world_to_local(&ray.start);
    let direction = r.world_to_local(&(ray.direction + ray.start)) - start;

    let newray = geometry::Ray::new(start, direction);

    let vertices = match m.buffer_f32_get("position") {
        None => return out,
        Some(v) => v
    };

    fn get_vertex(v : &Vec<f32>, index: uint) -> Vec3
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
            //for i in b.data.iter() {
            for i in range_step(0, b.data.len(), 3) {
                let index = b.data[i] as uint;
                let v0 = get_vertex(&vertices.data, index).mul(scale);
                let index = b.data[i+1] as uint;
                let v1 = get_vertex(&vertices.data, index).mul(scale);
                let index = b.data[i+2] as uint;
                let v2 = get_vertex(&vertices.data, index).mul(scale);

                let tri = geometry::Triangle::new(v0,v1,v2);
                //TODhttps://www.asiatorrents.me/index.php?page=torrentsO
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

    let mut a0;
    let mut a1;
    let mut a2;
    let mut b0;
    let mut b1;
    let mut b2;

    if abs(n.x) > abs(n.y) {
        if abs(n.x) > abs(n.z) {
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
        if abs(n.y) > abs(n.z) {
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

    let temp = a1*a2 - b1*a2;

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

