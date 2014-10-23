use vec;
use matrix;
use mesh_render;

use std::collections::{DList};
use sync::{RWLock, Arc};//,RWLockReadGuard};

#[deriving(Decodable, Encodable)]
pub struct Object
{
    pub name : String,
    pub mesh_render : Option<mesh_render::MeshRender>,
    pub position : vec::Vec3,
    pub orientation : vec::Quat,
    //pub angles : vec::Vec3,
    pub scale : vec::Vec3,
    pub children : DList<Arc<RWLock<Object>>>,
    pub parent : Option<Arc<RWLock<Object>>>
}

impl Object
{
    pub fn new(name : &str) -> Object
    {
        Object {
            name : String::from_str(name),
            mesh_render : None,
            position : vec::Vec3::zero(),
            orientation : vec::Quat::identity(),
            //angles : vec::Vec3::zero(),
            scale : vec::Vec3::one(),
            children : DList::new(),
            parent : None
        }
    }

    /*
    pub fn new_ref(name : &str) -> Rc<RefCell<Object>>
    {
        Rc::new(RefCell::new(Object::new(name)))
    }
    */


    pub fn empty() -> Object
    {
        Object::new("empty")
    }

    pub fn matrix_get(&self) -> matrix::Matrix4
    {
        let mt = matrix::Matrix4::translation(self.position);
        let mq = matrix::Matrix4::rotation(self.orientation);
        let ms = matrix::Matrix4::scale(self.scale);

        mt * mq * ms
    }

    /*
    pub fn child_add(&mut self, child : Arc<RWLock<Object>>)
    {
        self.children.push(child);
        child.write().parent = Some(self.clone());
    }
    */


    pub fn world_position(&self) -> vec::Vec3
    {
        match self.parent {
            None => return self.position,
            Some(ref parent) => {
                let wo = parent.read().world_orientation();
                let p = wo.rotate_vec3(&self.position);
                return p + parent.read().world_position();
            }
        }
    }

    pub fn world_orientation(&self) -> vec::Quat
    {
        match self.parent {
            None => return self.orientation,
            Some(ref p) => {
                return p.read().world_orientation() * self.orientation;
            }
        }
    }

    pub fn world_scale(&self) -> vec::Vec3
    {
        match self.parent {
            None => return self.scale,
            Some(ref p) => {
                return self.scale.mul(& p.read().world_scale());
            }
        }
    }
}

pub fn child_add(parent : Arc<RWLock<Object>>, child : Arc<RWLock<Object>>)
{
    parent.write().children.push(child.clone());
    child.write().parent = Some(parent.clone());
}


/*
impl PropertyShow for Object
{
    fn create_widget(window : &Window)
    {
        let c = unsafe { window_property_new(window) };
    }
}
*/

