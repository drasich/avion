//use component;
use std::rc::Rc;
use std::cell::RefCell;
use component::Component;
use component::manager::Encode;
//use object::ComponentFunc;
use object::Object;
use transform;
use rustc_serialize::{json, Encodable, Encoder, Decoder, Decodable};
use resource;

#[derive(Clone, RustcEncodable, RustcDecodable)]
pub struct Player
{
    pub speed : f64
}

pub struct PlayerBehavior;

impl Player
{
    pub fn new() -> Player
    {
        Player {
            speed : 3f64
        }
    }
}

pub fn player_new(ob : &Object, resource : &resource::ResourceGroup) -> Box<Component>
//pub fn player_new() -> Box<Component>
{
    box PlayerBehavior
}

impl Component for PlayerBehavior
{
    fn copy(&self) -> Rc<RefCell<Box<Component>>>
    {
        Rc::new(RefCell::new(box PlayerBehavior))
    }

    //fn update(&mut self, dt : f64) {}
    fn update(&mut self, ob : &mut Object, dt : f64)
    {
        let speed = {
            match ob.get_mut_comp_data::<Player>(){
                Some(s) => s.speed,
                None => 0f64
            }
        };

        //let yep = ob.get_mut_comp_data::<Player>();

        let mut ori = ob.orientation.get_angle_xyz();
        ori.x += speed;
        ob.orientation = transform::Orientation::new_with_angle_xyz(&ori);
    }

    fn get_name(&self) -> String {
        "player_behavior".to_string()
    }

    /*
    fn new(ob : &Object) -> Box<Component>
    {
        box PlayerBehavior
    }
    */

    /*
    fn new(ob : &Object) -> Box<Component>
    {
        box PlayerBehavior
    }
    */
}

#[derive(Clone, RustcEncodable, RustcDecodable)]
pub struct Enemy {
    name : String
}

#[derive(Clone, RustcEncodable, RustcDecodable)]
pub struct Collider {
    name : String
}


impl Encode for Player
{
  fn encode_this<E : Encoder>(&self, encoder: &mut E)// -> Result<(), &str>
  {
      let _ = self.encode(encoder);

  }

}

