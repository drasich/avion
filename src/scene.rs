use object;
//use std::collections::{DList,Deque};
use std::collections::{DList};
use std::rc::Rc;
use std::cell::RefCell;
use sync::{RWLock, Arc,RWLockReadGuard};
use std::io::File;
//use serialize::{Encodable, Encoder, Decoder, Decodable};
use serialize::{json, Encodable, Encoder, Decoder};


#[deriving(Decodable, Encodable)]
pub struct Scene
{
    pub name : String,
    pub objects : DList<Rc<RefCell<object::Object>>>,
    pub tests : DList<Arc<RWLock<object::Object>>>
}

impl Scene
{
    pub fn new(name : &str) -> Scene
    {
        Scene {
            name : String::from_str(name),
            objects : DList::new(),
            tests : DList::new()
        }
    }

    pub fn new_from_file(file_path : &str) -> Scene
    {
        let file = File::open(&Path::new(file_path)).read_to_string().unwrap();
        let scene : Scene = json::decode(file.as_slice()).unwrap();

        scene
    }

    pub fn save(&self)
    {
        let mut file = File::create(&Path::new(self.name.as_slice()));
        //let mut stdwriter = stdio::stdout();
        //let mut encoder = json::Encoder::new(&mut stdwriter);
        //let mut encoder = json::PrettyEncoder::new(&mut stdwriter);
        let mut encoder = json::PrettyEncoder::new(&mut file);
        //let mut encoder = json::Encoder::new(&mut file);

        //println!("scene : \n\n {}", json::encode(&scene));
        self.encode(&mut encoder).unwrap();
    }

}
