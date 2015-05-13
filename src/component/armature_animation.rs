//use component;
use std::rc::Rc;
use std::cell::RefCell;
use component::Component;
use component::manager::Encode;
//use object::ComponentFunc;
use object::Object;
use transform;
use rustc_serialize::{json, Encodable, Encoder, Decoder, Decodable};

use armature;
use mesh;

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
    armature : armature::Armature,
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

    fn new(ob : &Object) -> ArmatureAnimation
    {
        let arm = {
            match ob.get_comp_data::<armature::Armature>(){
                Some(a) => a.clone(),
                None => panic!("no armature data")
            }
        };

        ArmatureAnimation {
            state : State::Idle,
            armature : arm,
            mesh : None
        }
    }

}

