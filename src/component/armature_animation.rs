//use component;
use std::rc::Rc;
use std::cell::RefCell;
use rustc_serialize::{json, Encodable, Encoder, Decoder, Decodable};
use std::sync::{RwLock, Arc};


use component::{Component, CompData};
use component::manager::Encode;
//use object::ComponentFunc;
use object::Object;
use transform;
use armature;
use mesh;
use resource;

#[derive(Copy,Clone)]
pub enum State
{
    Idle,
    Play,
    Pause,
    Stop
}

pub struct ArmatureAnimation
{
    state : State,
    //armature : armature::Armature,
    armature : Arc<RwLock<armature::Armature>>,
    arm_instance : armature::ArmatureInstance,
    mesh : Option<resource::ResTT<mesh::Mesh>>,
    action : Option<String>,
    time : f64

    //TODO mesh component + dependencies
    //mesh_base : Option<resource::ResTT<MeshRenderComponent>>,
    //mesh_renderer : Rc<component::meshrender::MeshRenderer>,
}

impl Component for ArmatureAnimation
{
    /*
    fn copy(&self) -> Rc<RefCell<Box<Component>>>
    {
        Rc::new(RefCell::new(box))
    }
    */

    fn copy(&self) -> Rc<RefCell<Box<Component>>>
    {
        Rc::new(RefCell::new(
                box ArmatureAnimation
                {
                    state : self.state,
                    armature : self.armature.clone(),
                    mesh : self.mesh.clone(),
                    arm_instance : self.arm_instance.clone(),
                    action : self.action.clone(),
                    time : self.time

                }))
    }

    fn update(&mut self, ob : &mut Object, dt : f64)
    {
        println!("update armature anim");
        let mut mr = 
            if let Some(ref mut mr) = ob.mesh_render {
                mr
            }
        else {
            return;
        };

        let action = if let Some(ref a) = self.action {
            a
        }
        else {
            return
        };

        self.time = self.time + dt;

        self.arm_instance.set_pose(&*self.armature.read().unwrap(), action.as_str(), self.time);

        let mut mi = mr.get_or_create_mesh_instance();
            //if let Some(ref m) = mr.mesh_instance {
            //m.apply_armature_pose(self.arm_instance);
            //}

        //let normal_pose = 

        //TODO get the current animation pose with the action name and the time.
        // get the bones translation and rotation DIFFERENCE with the original pose.
        // ...
        //get the original mesh and apply weights 

    }

    fn get_name(&self) -> String {
        "armature_animation".to_string()
    }
}

pub fn new(ob : &Object, resource : &resource::ResourceGroup) -> Box<Component>
{
    println!("armature anim new---->>>>");
    let arm = {
        match ob.get_comp_data::<armature::ArmaturePath>(){
            Some(a) => a.clone(),
            None => panic!("no armature data")
        }
    };

    let armature = resource.armature_manager.borrow_mut().request_use_no_proc(arm.as_ref());
    let instance = armature.read().unwrap().create_instance();

    let arm_anim = ArmatureAnimation {
        state : State::Idle,
        armature : armature,
        arm_instance : instance,
        mesh : None,
        action : None,
        time : 0f64
    };

    box arm_anim
}


//TODO
//fn update_mesh_with_armature(mesh_instance : &mut mesh::Mesh
