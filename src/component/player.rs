//use component;
use std::rc::Rc;
use std::cell::RefCell;
use component::Component;
//use object::ComponentFunc;
use object::Object;
use transform;
use rustc_serialize::{json, Encodable, Encoder, Decoder, Decodable};

#[derive(RustcEncodable, RustcDecodable)]
pub struct Player
{
    pub speed : f64
}

impl Player
{
    pub fn new() -> Player
    {
        Player {
            speed : 3f64
        }
    }
}

pub fn player_new() -> Box<Component>
{
    box Player::new()
}

impl Component for Player
{
    fn copy(&self) -> Rc<RefCell<Box<Component>>>
    {
        Rc::new(RefCell::new(box Player { speed : 3f64 }))
    }

    //fn update(&mut self, dt : f64) {}
    fn update(&mut self, ob : &mut Object, dt : f64)
    {
        let mut ori = ob.orientation.get_angle_xyz();
        ori.x += 1f64;
        ob.orientation = transform::Orientation::new_with_angle_xyz(&ori);
    }

    fn get_name(&self) -> String {
        "player".to_string()
    }

}
