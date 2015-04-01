//use component;
use std::rc::Rc;
use std::cell::RefCell;
use object::Component;
use object::Object;

pub struct Player
{
    pub speed : f64
}

impl Component for Player
{
    fn copy(&self) -> Rc<RefCell<Box<Component>>>
    {
        Rc::new(RefCell::new(box Player { speed : 3f64 }))
    }
    //fn update(&mut self, dt : f64) {}
    fn update(&self, ob : &Object, dt : f64)
    {

    }

}
