use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied,Vacant};
use std::fs::File;
use byteorder::{LittleEndian, ReadBytesExt};
use std::path::Path;
use std::io::Read;
use rustc_serialize::{Encodable, Encoder, Decoder, Decodable};

//use libc::{c_char, c_int, c_uint, c_void};
use libc::{c_uint, c_void};
use std::mem;
use resource;
use geometry;
use vec;

fn read_string(file : &mut File) -> String
{
    let typelen = file.read_u16::<LittleEndian>().unwrap();
    println!("number : {} ", typelen);
    let mut typevec = vec![0u8; typelen as usize];
    file.read(&mut typevec);
    let typename = String::from_utf8(typevec).unwrap();
    println!("type name : {} ", typename);

    let len = file.read_u16::<LittleEndian>().unwrap();
    println!("number : {} ", len);
    let mut namevec = vec![0u8; len as usize];
    file.read(&mut namevec);
    let name = String::from_utf8(namevec).unwrap();
    println!("name : {} ", name);

    name
}

fn read_vec3(file : &mut File) -> vec::Vec3
{
    let x = file.read_f32::<LittleEndian>().unwrap() as f64;
    let y = file.read_f32::<LittleEndian>().unwrap() as f64;
    let z = file.read_f32::<LittleEndian>().unwrap() as f64;

    vec::Vec3::new(x,y,z)
}

fn read_quat(file : &mut File) -> vec::Quat
{
    let x = file.read_f32::<LittleEndian>().unwrap() as f64;
    let y = file.read_f32::<LittleEndian>().unwrap() as f64;
    let z = file.read_f32::<LittleEndian>().unwrap() as f64;
    let x = file.read_f32::<LittleEndian>().unwrap() as f64;

    vec::Quat::new(x,y,z,z)
}



#[derive(Clone)]
struct Bone
{
    //doesnt change
    name : String,
    position_base : vec::Vec3,
    rotation_base : vec::Quat,
    children : Vec<Bone>,

    position : vec::Vec3,
    rotation : vec::Quat,
}

impl Bone {
    fn new(file : &mut File) -> Bone 
    {
        let name = read_string(file);
        let pos = read_vec3(file);
        let rot = read_quat(file);

        let mut bone = Bone {
            name : name,
            position_base : pos,
            rotation_base : rot,
            children : Vec::new(),
            position: pos,
            rotation : rot
        };

        let child_count = file.read_u16::<LittleEndian>().unwrap() as usize;
        println!("child count : {} ", child_count);

        for i in 0usize..child_count {
            let child = Bone::new(file);
            bone.add_child(child);
        }

        bone
    }

    fn add_child(&mut self, child : Bone)
    {
        self.children.push(child);
    }
}

#[derive(Clone)]
enum FrameData
{
    Position(vec::Vec3),
    Orientation(vec::Quat),
}

#[derive(Clone)]
struct Frame {
   time : f64,
   data : FrameData
}

#[derive(Clone)]
enum Data
{
    Position,
    Quaternion,
    Euler,
    Scale
}

#[derive(Clone)]
struct Curve
{
    bone : usize, //Bone, //TODO just reference index, ou autre chose
    data : Data,
    frames : Vec<Frame>,
    frame_start : f64,
    frame_end : f64
}

impl Curve
{
    fn new(file : &mut File, bone_index : usize) -> Curve
    {
        //TODO
        //let bone_name = read_string(file);
        let data_kind_str = read_string(file);

        let data_kind = match data_kind_str.as_slice() {
            "position" => Data::Position,
            "quaternion" => Data::Quaternion,
            "euler" => Data::Euler,
            "scale" => Data::Scale,
            _ => panic!("armature curve : no such kind")
        };

        let curve = Curve {
            bone : bone_index,
            data : data_kind,
            frames : Vec::new(),
            frame_start : 0f64,
            frame_end : 0f64
        };

        curve
    }
}

#[derive(Clone)]
struct Action
{
    name : String,
    curves : Vec<Curve>,
    frame_start : f64,
    frame_end : f64
}

impl Action {
    fn new(file : &mut File) -> Action 
    {
        let name = read_string(file);

        let mut action = Action {
            name : name,
            curves : Vec::new(),
            frame_start: 0f64,
            frame_end : 0f64
        };

        let curves_nb = file.read_u16::<LittleEndian>().unwrap() as usize;
        println!("curves count : {} ", curves_nb);

        for i in 0usize..curves_nb {
            //TODO
            //let curve = Curve::new(file);
        }

        action
    }

    fn add_curve(&mut self, curve : Curve)
    {
        if curve.frame_start < self.frame_start {
            self.frame_start = curve.frame_start;
        }
        if curve.frame_end > self.frame_end {
            self.frame_end = curve.frame_end;
        }

        self.curves.push(curve);
    }
}


//TODO clone : only have one instance of the base armature
// and have ArmatureInstance for the variation
#[derive(Clone)]
pub struct Armature
{
    pub name : String,
    position : vec::Vec3,
    rotation : vec::Quat,
    scale : vec::Vec3,
    actions : Vec<Action>,
    bones : Vec<Bone>
}

impl Armature {

    pub fn new(file : &str) -> Armature 
    {
        Armature {
            name : String::from_str(file),
            position : vec::Vec3::zero(),
            rotation : vec::Quat::identity(),
            scale : vec::Vec3::zero(),
            actions : Vec::new(),
            bones : Vec::new()
        }
    }

    pub fn file_read(&mut self) 
    {
        /*
        if self.state != 0 {
            return;
        }
        */

        let mut file = match File::open(&Path::new(self.name.as_slice())) {
            Ok(f) => {f},
            Err(e) => {
                println!("Error reading file '{}'. Error: {}", self.name, e);
                return;
            }
        };

        let string_type = "string type" ;//read_string(&mut file);
        let yop = read_string(&mut file);
        //self.name = yop.clone();


        println!("______file read _________ name :::: {}, {}",
                 string_type, self.name);

        self.position = read_vec3(&mut file);
        println!("______ position _________ : {:?}", self.position);
        self.rotation = read_quat(&mut file);
        println!("______ rotation _________ : {:?}", self.rotation);
        self.scale = read_vec3(&mut file);
        println!("______ scale _________ : {:?}", self.scale);

        let bone_count = file.read_u16::<LittleEndian>().unwrap() as usize;
        println!("bone count : {} ", bone_count);

        for i in 0usize..bone_count {
            let bone = Bone::new(&mut file);
            self.bones.push(bone);
        }

        let action_count = file.read_u16::<LittleEndian>().unwrap() as usize;
        println!("action count : {} ", action_count);

        for i in 0usize..action_count {
            let action = Action::new(&mut file);
            self.actions.push(action);
        }
    }

}

impl Encodable for Armature {
  fn encode<E : Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
      println!("______ encode _________ name ::: {}", self.name);
      encoder.emit_struct("armature", 1, |encoder| {
          try!(encoder.emit_struct_field( "name", 0usize, |encoder| self.name.encode(encoder)));
          Ok(())
      })
  }
}

impl Decodable for Armature {
  fn decode<D : Decoder>(decoder: &mut D) -> Result<Armature, D::Error> {
    decoder.read_struct("root", 0, |decoder| {
         Ok(Armature{
          name: try!(decoder.read_struct_field("name", 0, |decoder| Decodable::decode(decoder))),
          position : vec::Vec3::zero(),
          rotation : vec::Quat::identity(),
          scale : vec::Vec3::zero(),
          actions : Vec::new(),
          bones : Vec::new()
        })
    })
  }
}

