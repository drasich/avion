use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied,Vacant};
use std::fs::File;
use byteorder::{LittleEndian, ReadBytesExt};
use std::path::Path;
use std::io::Read;

//use libc::{c_char, c_int, c_uint, c_void};
use libc::{c_uint, c_void};
use std::mem;
use resource;
use geometry;
use vec;

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

enum FrameData
{
    Position(vec::Vec3),
    Orientation(vec::Quat),
}

struct Frame {
   time : f64,
   data : FrameData
}
