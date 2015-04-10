//use component;
use std::rc::Rc;
use std::cell::RefCell;
use object::Component;
use object::Object;
use transform;

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

}
