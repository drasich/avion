//#![feature(log_syntax)]
//#![feature(trace_macros)]
//#![feature(slicing_syntax)]
#![feature(box_syntax)]
#![feature(core)]
#![feature(convert)]
#![feature(step_by)]
#![feature(zero_one)]

#![feature(vec_push_all)]
#![feature(borrow_state)]

//TODO remove
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(improper_ctypes)]
//#![allow(ctypes)]

#![feature(plugin)]
//#![plugin(clippy)]

extern crate rustc_serialize;
extern crate byteorder;
extern crate libc;
//extern crate sync;
extern crate png;
extern crate toml;
//extern crate debug;
extern crate uuid;
extern crate core;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate lua;

extern crate dormin;

//use serialize::{json, Encodable, Encoder, Decoder, Decodable};
use std::collections::HashMap;
use std::sync::{RwLock, Arc};
//use std::rc::Rc;
//use std::cell::RefCell;
use std::mem;
//use std::any::{Any, AnyRefExt};

use std::io::{self, Write};
use std::path::Path;
use std::process;

/*
#[macro_use]
use dormin::property;

use dormin::resource;
mod shader;
mod material;
mod armature;
mod mesh;
//mod mesh_render;
mod render;
mod object;
mod uniform;
mod matrix;
mod vec;
mod camera;
mod scene;
mod texture;
mod geometry;
mod intersection;
mod fbo;
mod factory;


mod transform;

mod model;

mod component;
use component::manager;
*/

mod dragger;
mod ui;
mod operation;
mod context;
mod control;
mod util;

use dormin::component;

static mut sTest : i32 = 5;
#[derive(Debug)]
struct Ob {
    pos : i32
}

#[derive(Debug)]
struct Cha {
    left : Ob,
    right : Ob
}

fn testcha(left : &mut Ob, right : &mut Ob)
{
    left.pos = 13;
    right.pos = 16;
}


fn main() {
    let files = util::get_files_in_dir("scene");
    let cs = util::to_cstring(files);
    util::print_vec_cstring(cs);
    util::pass_slice();
    unsafe {
    sTest = 4432;
    }

    let mut t = Vec::new();
    t.push(Ob { pos : 1i32 }); 
    t.push(Ob { pos : 2i32 }); 
    t.push(Ob { pos : 3i32 }); 
    t.push(Ob { pos : 4i32 }); 
    t.push(Ob { pos : 5i32 }); 
    t.push(Ob { pos : 6i32 }); 
    t.push(Ob { pos : 7i32 }); 

    let slicefirst = t.as_mut_slice();

    let mut col = slicefirst.iter_mut().filter(|x| x.pos != 1).collect::<Vec<_>>();
    let slice = col.as_mut_slice();

    let (left,right) = slice.split_at_mut(2);
    let (middle, right) = right.split_at_mut(1);

    let left = &mut left[0];
    let middle = &mut middle[0];
    let right = &mut right[0];

    left.pos = middle.pos + 10;
    middle.pos = left.pos + 100;
    right.pos = left.pos + middle.pos;
    println!("left, middle, r2, {:?}, {:?}, {:?}", left, middle, right);

    //println!("test {:?}", slicefirst);

    let mut cha = Cha {
      left : Ob { pos : 5i32 },
      right : Ob { pos : 7i32 },
    };

    testcha(&mut cha.left, &mut cha.right);
    println!("cha : {:?}", cha);


    {
     println!("The map has {} entries.", *component::manager::COUNT);
    }

    {
    let mut hash = &mut component::manager::HASHMAP.lock().unwrap();
    println!("going to insert 5");
    hash.insert(5, "cinq");
    println!("The entry for `1` is \"{}\".", hash.get(&1).unwrap());
    println!("The entry for `0` is \"{}\".", hash.get(&0).unwrap());
    println!("The entry for `5` is \"{}\".", hash.get(&5).unwrap());
    }

    {
     println!("The map has {} entries.", *component::manager::COUNT);
    }

    {
        //let mut cm = component::Manager::new();
        let mut cm = component::manager::COMP_MGR.lock().unwrap();
        cm.register_component("player_behavior", component::player::player_new);
        cm.register_component(
            "armature_animation",
            component::armature_animation::new);
    }

    /*
    let mut c = property::Chris::new();
    //let yep  = c.cget_property("x");
    c.cset_property_hier(
        [String::from_str("boxpos"),String::from_str("x")].to_vec(),
        &555f64);
    c.cset_property_hier(
        [String::from_str("boxpos")].to_vec(),
        &vec::Vec3::new(111f64,-9f64,-33f64));
    let yep  = c.cget_property_hier(
        [String::from_str("boxpos"),String::from_str("x")].to_vec());
    match yep {
        property::BoxAny(v) => {
            match v.downcast_ref::<f64>() {
                Some(vv) => println!("I found the value {}", *vv ),
                None => {}
            }
        },
        _ => {}
    }
    */

    /*

    let mut mattt = Arc::new(RwLock::new( material::Material {
        name : String::from_str("material/my_mat"),
        //shader : Some(shader::Shader::new("shader/simple.sh")),
        shader : Some(resource::ResTT::new("shader/simple.sh")),
        state : 0,
        textures : HashMap::new(),
        uniforms : HashMap::new(),
    }));
    //mattt.write().textures.insert(String::from_str("texnametest"), material::SamplerImageFile(resource::ResTT::new("image/base_skeleton_col.png")));
    mattt.write().unwrap().textures.insert(String::from_str("texnametest"), material::Sampler::Fbo(resource::ResTT::new("fbo_all"),material::Attachment::Depth));
    //mattt.write().uniforms.insert(String::from_str("unitest"), box shader::Vec2(vec::Vec2::new(5.6f64, 1.2f64)));
    //mattt.read().savetoml();
    mattt.read().unwrap().save();
    */

    /*
    let matoml = r#"name = "material/my_mat"
state = 0
[shader]
name = "shader/simple.sh"
[texture]
name = "image/base_skeleton_col.png"
"#;

    let mattt = shader::Material::new_toml(matoml);
    */

    //let mat = Arc::new(RwLock::new(shader::Material::new_from_file("material/simple.mat")));
    //mat.read().save();

    //let cam = camera::Camera::new();

    /*
    let oo = r.scene.read().object_find("yep");
    match oo {
        Some(o) => { println!("I found the obj");
            o.write().child_add(Arc::new(RwLock::new(object::Object::new("my_child"))));
            println!("yooooooooooooooooooooooooooooooooo");
            property::print_pt(o.read().get_property("name"));
        }
        None => {}
    }
    */


        //r.prepare_passes();

        /*
        for o in scene.objects.iter() {
            r.pass.objects.push((*o).clone());
        }
        */


        /*
        {
            let mut o = object::Object::new("nouvel object");
            o.mesh_render = Some(mesh_render::MeshRender::new("model/skeletonmesh.mesh", "material/simple.mat"));
            //o.mesh_set(mesh::new::from_file("model/skeletonmesh.mesh");
            //o.position = vec::Vec3::new(100f64,0f64,-1000f64);
            let mut oref = Rc::new(RefCell::new(o));
            scene.objects.push(oref.clone());
            //r.pass.objects.push(oref.clone());
        }
        */

        /*
        {
            let mut o = object::Object::new("yep2");
            o.mesh_set(mesh.clone());
            o.position = vec::Vec3::new(-100f64,-100f64,-1000f64);
            o.orientation = vec::Quat::new_axis_angle(vec::Vec3::new(0f64,1f64,0f64), consts::PI/2f64);
            o.scale = vec::Vec3::new(2f64,2f64,2f64);
            let mut oref = Rc::new(RefCell::new(o));
            //scene.objects.push(oref.clone());
            r.pass.objects.push(oref.clone());

            ////println!("{}", json::encode(&*oref.borrow()));
            //let ob_encoded = json::encode(&*oref.borrow());
            //let ob_decoded : object::Object = json::decode(ob_encoded.as_slice()).unwrap();
            //println!(" ob decoded {}", json::encode(&ob_decoded));
            //
        }
        */

        //scene.save();

    {

    let mut lua = lua::State::new();
    lua.openlibs();
    /*
    match lua.loadfile(None) {
        Ok(()) => (),
        Err(lua::LoadFileError::ErrSyntax) => panic!("syntax error"),
        Err(lua::LoadFileError::ErrMem) => panic!("memory allocation error"),
        Err(lua::LoadFileError::ErrFile) => panic!("file error (?!?)")
    }
    lua.call(0, 0);
    */

    // Load the file containing the script we are going to run
    let path = Path::new("simpleapi.lua");
    match lua.loadfile(Some(&path)) {
        Ok(_) => (),
        Err(_) => {
            // If something went wrong, error message is at the top of the stack
            let _ = writeln!(&mut io::stderr(),
            "Couldn't load file: {}", lua.describe(-1));
            process::exit(1);
        }
    }

    /*
     * Ok, now here we go: We pass data to the lua script on the stack.
     * That is, we first have to prepare Lua's virtual stack the way we
     * want the script to receive it, then ask Lua to run it.
     */
    lua.newtable(); // We will pass a table

    for i in 1..6 {
        lua.pushinteger(i);   // Push the table index
        lua.pushinteger(i*2); // Push the cell value
        lua.rawset(-3);       // Stores the pair in the table
    }

    // By what name is the script going to reference our table?
    lua.setglobal("foo");

    // Ask Lua to run our little script
    match lua.pcall(0, lua::MULTRET, 0) {
        Ok(()) => (),
        Err(_) => {
            let _ = writeln!(&mut io::stderr(),
                             "Failed to run script: {}", lua.describe(-1));
            process::exit(1);
        }
    }

    // Get the returned value at the to of the stack (index -1)
    let sum = lua.tonumber(-1);

    println!("Script returned: {}", sum);

    lua.pop(1); // Take the returned value out of the stack

    }
























    //let m = box ui::Master::new();
    //let m = Rc::new(RefCell::new(ui::Master::new()));
    let mut container = box ui::WidgetContainer::new();
    let m = ui::Master::new(&mut container);


    unsafe {
        let appdata = ui::AppCbData{
            master : mem::transmute(box m.clone()),
            container : mem::transmute(&container)  };

        ui::init_callback_set(
            ui::init_cb,
            //mem::transmute(box m.clone()));
            mem::transmute(box appdata.clone()));

        ui::exit_callback_set(
            ui::exit_cb,
            //mem::transmute(box m.clone()));
            mem::transmute(box appdata));
    }

    unsafe {
        ui::elm_simple_window_main();
    };
}


