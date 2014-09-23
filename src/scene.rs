use object;
use std::collections::{DList,Deque};
use std::rc::Rc;
use std::cell::RefCell;
use std::io::File;
use std::io::stdio;
use serialize::json::ToJson;
use serialize::{json, Encodable, Encoder, Decoder, Decodable};


#[deriving(Decodable, Encodable)]
pub struct Scene
{
    pub name : String,
    pub objects : DList<Rc<RefCell<object::Object>>>
}

impl Scene
{
    pub fn new(name : &str) -> Scene
    {
        Scene {
            name : String::from_str(name),
            objects : DList::new()
        }
    }

    pub fn new_from_file(file_path : &str) -> Scene
    {
        let file = File::open(&Path::new(file_path)).read_to_string().unwrap();
        let mut scene : Scene = json::decode(file.as_slice()).unwrap();

        scene
    }

    pub fn save(&self)
    {
        let mut file = File::create(&Path::new(self.name.as_slice()));
        let mut stdwriter = stdio::stdout();
        //let mut encoder = json::Encoder::new(&mut stdwriter);
        //let mut encoder = json::PrettyEncoder::new(&mut stdwriter);
        let mut encoder = json::PrettyEncoder::new(&mut file);
        //let mut encoder = json::Encoder::new(&mut file);

        //println!("scene : \n\n {}", json::encode(&scene));
        self.encode(&mut encoder).unwrap();
    }

}
