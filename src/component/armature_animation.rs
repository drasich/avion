//use component;
use std::rc::Rc;
use std::cell::RefCell;
use rustc_serialize::{json, Encodable, Encoder, Decoder, Decodable};


use component::{Component, CompData};
use component::manager::Encode;
//use object::ComponentFunc;
use object::Object;
use transform;
use armature;
use mesh;
use resource;

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
    armature : resource::ResTT<armature::Armature>,
    mesh : Option<mesh::Mesh>,
}

impl Component for ArmatureAnimation
{
    /*
    fn copy(&self) -> Rc<RefCell<Box<Component>>>
    {
        Rc::new(RefCell::new(box))
    }
    */

    fn update(&mut self, ob : &mut Object, dt : f64)
    {
    }

    fn get_name(&self) -> String {
        "armature_animation".to_string()
    }

    //fn new(ob : &Object) -> ArmatureAnimation
    /*
    fn new(ob : &Object) -> Box<Component>
    {
        let arm = {
            match ob.get_comp_data::<armature::ArmaturePath>(){
                Some(a) => a.clone(),
                None => panic!("no armature data")
            }
        };

        let arm_anim = ArmatureAnimation {
            state : State::Idle,
            armature : resource::ResTT::new(arm.as_ref()),
            mesh : None
        };

        box arm_anim
    }
    */

}

pub fn new(ob : &Object) -> Box<Component>
{
    let arm = {
        match ob.get_comp_data::<armature::ArmaturePath>(){
            Some(a) => a.clone(),
            None => panic!("no armature data")
        }
    };

    let arm_anim = ArmatureAnimation {
        state : State::Idle,
        armature : resource::ResTT::new(arm.as_ref()),
        mesh : None
    };

    box arm_anim
}

