use std::sync::{RwLock, Arc};
use std::fs;
use std::mem;
use std::ffi::{CString, CStr};
use std::str;
use libc::{c_void, c_int, size_t, c_char};

use dormin::vec;
use dormin::object;

pub fn objects_center(objects : &[Arc<RwLock<object::Object>>]) -> vec::Vec3
{
    let mut v = vec::Vec3::zero();
    for o in objects
    {
        v = v + o.read().unwrap().world_position();
    }

    v = v / objects.len() as f64;

    v
}

use std::path::{Path, PathBuf};
pub fn get_files_in_dir(path : &str) -> Vec<PathBuf>
{
    let files = fs::read_dir(path).unwrap();
    /*
    for file in files {
        println!("Name: {}", file.unwrap().path().display())
    }
    */

    files.map(|x| x.unwrap().path()).collect()
}

#[link(name = "joker")]
extern {
    fn do_something_with_slice(slice : *const c_void, len : size_t);
}

pub fn to_cstring(v : Vec<PathBuf>) -> Vec<CString>
{
    v.iter().map(|x| CString::new(x.to_str().unwrap()).unwrap()).collect()
}

pub fn string_to_cstring(v : Vec<String>) -> Vec<CString>
{
    v.iter().map(|x| CString::new(x.as_str()).unwrap()).collect()
}

pub fn print_vec_cstring(v : Vec<CString>)
{
    let y : Vec<*const c_char> = v.iter().map( |x| x.as_ptr()).collect();

    unsafe { do_something_with_slice(
            y.as_ptr() as *const c_void,
            y.len() as size_t); }
}

pub fn pass_slice() 
{
    let s = [ 
        CString::new("test").unwrap().as_ptr(),
        CString::new("caca").unwrap().as_ptr(),
        CString::new("bouda").unwrap().as_ptr() ];

    unsafe { do_something_with_slice(
            s.as_ptr() as *const c_void,
            s.len() as size_t); }
}

pub fn join_string(path : &Vec<String>) -> String
{
    let mut s = String::new();
    let mut first = true;
    for v in path {
        if !first {
            s.push('/');
        }
        s.push_str(v);
        first = false;
    }

    s
}

pub fn join_str(path : &Vec<&str>) -> String
{
    let mut s = String::new();
    let mut first = true;
    for v in path {
        if !first {
            s.push('/');
        }
        s.push_str(*v);
        first = false;
    }

    s
}

