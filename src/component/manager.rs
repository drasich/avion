use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::rc::Rc;
use std::cell::RefCell;
use object::Object;
//use std::thread;
///use std::sync::mpsc::channel;

pub trait Component
{
    //fn new(&self) -> Rc<RefCell<Box<Component>>>;
    fn copy(&self) -> Rc<RefCell<Box<Component>>>;
    fn update(&mut self, ob : &mut Object, dt : f64) {}
    fn get_name(&self) -> String;
}

type ComponentCreationFn = fn() -> Box<Component>;

pub struct Manager {
    name : String,
    components : HashMap<String, ComponentCreationFn>
}

impl Manager {
    pub fn new() -> Self
    {
        Manager {
            name : "test".to_string(),
            components: HashMap::new()
        }
    }
    pub fn register_component(&mut self, name : &str, 
                              f : ComponentCreationFn)
                              //f : fn() -> Box<Component>)
                              //f : fn() -> Component)
    {
        self.components.insert(name.to_string(), f);
    }

    pub fn create_component(&self, name : &str) -> Option<Box<Component>>
    {
        match self.components.get(&name.to_string()) {
            Some(f) => Some(f()),
            None => None
        }
    }
}


lazy_static! {
    pub static ref HASHMAP: Mutex<HashMap<u32, &'static str>> = {
        let mut m = HashMap::new();
        m.insert(0, "foo");
        m.insert(1, "bar");
        m.insert(2, "baz");
        println!("test chris");
        Mutex::new(m)
    };
    pub static ref COUNT: usize = {
        println!("size !!!!!!");
        let hash = &mut HASHMAP.lock().unwrap();
        hash.len()
    };
    static ref NUMBER: u32 = times_two(21);
    pub static ref COMP_MGR : Mutex<Manager> = Mutex::new(Manager::new());
}

fn times_two(n: u32) -> u32 { n * 2 }
