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
    mesh : Option<resource::ResTT<mesh::Mesh>>,

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
                    mesh : self.mesh.clone()

                }))
    }

    fn update(&mut self, ob : &mut Object, dt : f64)
    {
        println!("update armature anim");
        let mr = 
            if let Some(ref mr) = ob.mesh_render {
                mr
            }
        else {
            return;
        };


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

    let arm_anim = ArmatureAnimation {
        state : State::Idle,
        armature : resource.armature_manager.borrow_mut().request_use_no_proc(arm.as_ref()),
        mesh : None
    };

    box arm_anim
}

