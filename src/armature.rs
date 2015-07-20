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

use property::{PropertyRead, PropertyGet, PropertyWrite, WriteValue};
use std::any::Any;


fn read_string(file : &mut File) -> String
{
    /*
    let typelen = file.read_u16::<LittleEndian>().unwrap();
    println!("number : {} ", typelen);
    let mut typevec = vec![0u8; typelen as usize];
    file.read(&mut typevec);
    let typename = String::from_utf8(typevec).unwrap();
    println!("type name : {} ", typename);
    */

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
    let w = file.read_f32::<LittleEndian>().unwrap() as f64;

    vec::Quat::new(x,y,z,w)
}



#[derive(Clone)]
pub struct Bone
{
    //doesnt change
    pub name : String,
    pub position_base : vec::Vec3,
    pub rotation_base : vec::Quat,
    pub parent : Option<usize>,
    pub children : Vec<usize>,

    pub head_from_arm : vec::Vec3,
    pub head : vec::Vec3,
    pub tail : vec::Vec3,

    pub position_diff : vec::Vec3,
    pub rotation_diff : vec::Quat,
}

impl Bone {
    fn new(file : &mut File) -> Bone 
    {
        let name = read_string(file);
        println!("------------------------------------------------name---------------------: {}", name);
        let pos = read_vec3(file);
        println!("pos: {:?}", pos);
        let head = read_vec3(file);
        println!("head : {:?}", head);
        let head_from_arm = read_vec3(file);
        println!("head from armature : {:?}", head_from_arm);
        let tail = read_vec3(file);
        println!("tail : {:?}", tail);

        let rot = read_quat(file);
        println!("rot: {:?}", rot);

        let mut bone = Bone {
            name : name,
            position_base : head,
            rotation_base : rot,
            parent : None,
            children : Vec::new(),
            position_diff: vec::Vec3::zero(),
            head: head,
            head_from_arm: head_from_arm,
            tail: tail,
            rotation_diff : vec::Quat::identity(),
            //position_relative: head,
            //rotation_relative : vec::Quat::identity(),
        };

        let has_parent = file.read_u8().unwrap() as usize;
        println!(" {} ,,, has parent : {} ", bone.name, has_parent);
        if has_parent == 1 {
            let parent = file.read_u16::<LittleEndian>().unwrap() as usize;
            bone.parent = Some(parent);
        }

        let child_count = file.read_u16::<LittleEndian>().unwrap() as usize;
        println!("child count : {} ", child_count);

        for i in 0usize..child_count {
            //let child = Bone::new(file);
            //bone.add_child(child);
            let child_index = file.read_u16::<LittleEndian>().unwrap() as usize;
            bone.children.push(child_index);
        }

        bone
    }

    /*
    fn add_child(&mut self, child : Bone)
    {
        self.children.push(child);
    }
    */

    /*
    pub fn get_position_relative_to_armature(&self, arm: &ArmatureInstance) -> Vec3
    {
        if let Some(
    }
    */

}

#[derive(Clone)]
enum FrameData
{
    Position(vec::Vec3),
    Orientation(vec::Quat),
    Scale(vec::Vec3),
}

#[derive(Clone)]
struct Frame {
   time : f64,
   data : FrameData
}

#[derive(Clone, Debug)]
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
    bone_name : String,
    bone_index : usize, //Bone, //TODO just reference index, ou autre chose
    data : Data,
    frames : Vec<Frame>,
    frame_start : f64,
    frame_end : f64
}

impl Curve
{
    fn new(file : &mut File, bone_name : String, bone_index : usize) -> Curve
    {
        let data_kind_str = read_string(file);

        let data_kind = match data_kind_str.as_ref() {
            "position" => Data::Position,
            "quaternion" => Data::Quaternion,
            "euler" => Data::Euler,
            "scale" => Data::Scale,
            _ => panic!("armature curve : no such kind")
        };

        let frames_nb = file.read_u16::<LittleEndian>().unwrap() as usize;
        println!("frames nb : {}", frames_nb);

        let mut curve = Curve {
            bone_name : bone_name,
            bone_index : bone_index,
            data : data_kind,
            frames : Vec::with_capacity(frames_nb),
            frame_start : 0f64,
            frame_end : 0f64
        };

        for i in 0usize..frames_nb
        {
            let time = file.read_f32::<LittleEndian>().unwrap() as f64;

            if time < curve.frame_start {
                curve.frame_start = time;
            }

            if time > curve.frame_end {
                curve.frame_end = time;
            }

            let data = match curve.data {
                Data::Position => {
                    let pos = read_vec3(file);
                    FrameData::Position(pos)
                } ,
                Data::Quaternion => {
                    let q = read_quat(file);
                    FrameData::Orientation(q)
                },
                Data::Scale => {
                    let s = read_vec3(file);
                    FrameData::Scale(s)
                },
                _ => panic!("not done yet :  {:?} ", curve.data )
            };

            let frame = Frame {
                time : time,
                data : data
            };

            curve.frames.push(frame);
        }

        curve
    }

    fn get_frames(&self, time : f64) -> (&Frame,&Frame)
    {
        let mut start = None;
        let mut end = None;
        let mut last = None;

        for f in self.frames.iter() {
            last = Some(f);

            if time == f.time {
                return (f, f);
            }
            else if time > f.time {
                start = Some(f);
            }
            else if time < f.time {
                end = Some(f);
                break;
            }
        }

        let last = last.unwrap();

        match (start, end) {
            (Some(s), Some(e)) => (s, e),
            (Some(s), None) => (s, last),
            (None, _) => (last, last),
        }
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
    fn new(file : &mut File, armature : &Armature) -> Action 
    {
        let name = read_string(file);
        println!("!!!!!!!! action name : {}",name);

        let mut action = Action {
            name : name,
            curves : Vec::new(),
            frame_start: 0f64,
            frame_end : 0f64
        };

        let curves_nb = file.read_u16::<LittleEndian>().unwrap() as usize;
        println!(".....curves count : {} ", curves_nb);

        for i in 0usize..curves_nb {
            //TODO for now use the name
            let bone_name = read_string(file);
            //let bone_index = armature.find_bone(bone_name.as_ref());
            let bone_index = file.read_u16::<LittleEndian>().unwrap() as usize;
            println!("...............bone : {}, {} ", bone_name, bone_index);
            let curve = Curve::new(file, bone_name, bone_index);
            if curve.frame_start < action.frame_start {
                action.frame_start = curve.frame_start;
            }
            if curve.frame_end > action.frame_end {
                action.frame_end = curve.frame_end;
            }
            action.curves.push(curve);
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

#[derive(Clone, RustcEncodable, RustcDecodable)]
pub struct ArmaturePath {
   pub name : String
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
    bones : Vec<Bone>,
    pub state : usize
}

impl Armature {

    pub fn new(file : &str) -> Armature 
    {
        Armature {
            name : String::from(file),
            position : vec::Vec3::zero(),
            rotation : vec::Quat::identity(),
            scale : vec::Vec3::zero(),
            actions : Vec::new(),
            bones : Vec::new(),
            state : 0usize
        }
    }

    pub fn create_instance(&self) -> ArmatureInstance
    {
        let pos = vec![vec::Vec3::zero(); self.bones.len()];
        let rot = vec![vec::Quat::identity(); self.bones.len()];

        let mut instance = ArmatureInstance {
            position : self.position,
            rotation : self.rotation,
            scale : self.scale,
            bones : self.bones.clone(),
            position_relative : pos,
            rotation_relative : rot,
        };

        instance.update_relative_coords();

        instance
    }

    pub fn file_read(&mut self) 
    {
        /*
        if self.state != 0 {
            return;
        }
        */

        let path : &Path = self.name.as_ref();
        let mut file = match File::open(path) {
            Ok(f) => {f},
            Err(e) => {
                println!("Error reading file '{}'. Error: {}", self.name, e);
                return;
            }
        };

        let string_type = read_string(&mut file);
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
            let action = Action::new(&mut file, self);
            self.actions.push(action);
        }

        self.state = 1usize;

    }

    /*
    pub fn find_bone(&self, name : &str) -> usize
    {
        for i in 0..self.bones.len() {
            if self.bones[i].name == name{
                return i;
            }
        }

        return 0usize;
    }

    pub fn get_bone(&self, index : usize) -> &Bone
    {
        &self.bones[index]
    }
    */


    pub fn find_action(&self, name : &str) -> Option<&Action>
    {
        for i in 0..self.actions.len() {
            if self.actions[i].name == name{
                return Some(&self.actions[i]);
            }
        }

        None
    }


}

/*
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
          bones : Vec::new(),
          state : 0usize
        })
    })
  }
}
*/


#[derive(Clone)]
pub struct ArmatureInstance
{
    pub position : vec::Vec3,
    pub rotation : vec::Quat,
    pub scale : vec::Vec3,
    bones : Vec<Bone>,
    pub position_relative : Vec<vec::Vec3>,
    pub rotation_relative : Vec<vec::Quat>,
}

impl ArmatureInstance
{
    pub fn get_bone(&self, index : usize) -> &Bone
    {
        &self.bones[index]
    }

    pub fn get_bones(&self) -> &Vec<Bone>
    {
        &self.bones
    }

    fn get_mut_bone(&mut self, index : usize) -> &mut Bone
    {
        &mut self.bones[index]
    }

    pub fn set_pose(&mut self, armature : &Armature, action_name : &str, time : f64)
    {
        let action = match armature.find_action(action_name) {
            Some(a) => a,
            None => { println!("no such action  {}", action_name);  return }
        };

        let frame = {
            let mut f = time*30f64;

            if f < action.frame_start {
                f = action.frame_start;
            }
            if f > action.frame_end {
                f = action.frame_end;
            }

            f
        };

        for curve in action.curves.iter()
        {
            let bone :&mut Bone = self.get_mut_bone(curve.bone_index);

            let (start, end) = curve.get_frames(frame);

            let ratio = if start.time != end.time {
                (frame - start.time) / (end.time - start.time)
            }
            else {
                0f64
            };

            match (&start.data, &end.data) {
                (&FrameData::Position(s), &FrameData::Position(e)) => {
                    let v1 = s * (1f64 -ratio);
                    let v2 = e * ratio;
                    bone.position_diff = v1 + v2;
                },
                (&FrameData::Orientation(s), &FrameData::Orientation(e)) => {
                    bone.rotation_diff = vec::quat_slerp(s, e, ratio);
                },
                (&FrameData::Scale(s), &FrameData::Scale(e)) => {
                    println!("scale not done yet");
                },
                (_,_) => println!("not yet")
            };
        }

        self.update_relative_coords();
    }

    fn update_relative_coords(&mut self)
    {
        for b in 0..self.bones.len()
        {
            //self.bones[b].rotation_relative = self.get_bone_rotation(&self.bones[b]);
            //self.bones[b].position_relative = self.get_bone_position(&self.bones[b]);
            self.rotation_relative[b] = self.get_bone_rotation(&self.bones[b]);
            self.position_relative[b] = self.get_bone_position(&self.bones[b]);
        }
    }

    fn get_bone_rotation(&self, b : &Bone) -> vec::Quat
    {
        let local = b.rotation_diff;

        match b.parent{
            Some(p) => self.get_bone_rotation(&self.bones[p]) * local,
            None => local
        }
    }

    fn get_bone_position(&self, b : &Bone) -> vec::Vec3
    {
        let mut local =  b.position_base + b.position_diff;

        match b.parent{
            Some(p) => {
                let pbone = &self.bones[p];
                local = (pbone.tail - pbone.head) + local;
                self.get_bone_position(pbone) + self.get_bone_rotation(pbone).rotate_vec3(&local)
            },
            None => local
        }
    }
}

property_set_impl!(ArmaturePath,[name]);
property_get_impl!(ArmaturePath,[name]);

