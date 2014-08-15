use std::collections::{DList,Deque};
use std::mem;
use std::ptr;
use std::rc::Rc;
use std::cell::RefCell;

#[link(name = "joker")]
extern {
    fn simple_window_init();
    fn elm_simple_window_main();
}

mod shader;
mod mesh;
mod render;
mod object;

static vertex_data_test: [f32, ..6] = [
    0.0,  0.5,
    0.5, -0.5,
    -0.5, -0.5
      ];

fn main() {
    unsafe {
        simple_window_init();
    };

    /*
    spawn(proc() {
        shader::shader_init();
        println!("its been called, its over right");
    });
    */

    /*
    spawn(proc() {
        mesh::mesh_init();
    });
    */

    spawn(proc() {
        let mut mm = box render::MeshManager {
            mesh : Rc::new(RefCell::new(mesh::Mesh { name : String::from_str("chris test"), buffer : None, state : 0 }))
        };

        let mut rm = box render::RequestManager {
            requests : DList::new()
        };

        rm.requests.push( box render::Request { mesh : mm.mesh.clone() });
        match rm.requests.front_mut(){
            None => (),
            Some(req) => req.handle()
        }

        println!("mesh name : {} ", mm.mesh.borrow().name);

    });

    spawn(proc() {

        let mut r = box render::Render { 
            pass :box render::RenderPass{
                      name : String::from_str("passtest"),
                      material : box shader::Material { name : String::from_str("nouveau"), shader : None, state : 0 },
                      objects : DList::new(),
                      drawables : DList::new(),
                  }
        };

        let mut mesh = box mesh::Mesh { name : String::from_str("mesh_name"), buffer : None, state : 0 };
        r.pass.objects.push(object::Object{name : String::from_str("objectyep"), mesh : Some(*mesh)}); 

        let x : i32 = 3;

        //let closure = || -> () { println!("{}", x) };

        extern fn yepyep(r : *const render::Render) -> *const render::Drawable {
            unsafe {
            return (*r).getDrawable();
            }
            //println!("test") ; 
            //return ptr::null()
        };

        extern fn render_draw_cb(r : *const render::Render) -> () {
            unsafe {
            return (*r).draw_frame();
            }
            //println!("test") ; 
            //return ptr::null()
        };

        unsafe {
            render::draw_data_set(yepyep, &*r);
            render::draw_callback_set(render_draw_cb, &*r);
        }

        r.init();
        r.draw();

    });

    unsafe { 
        elm_simple_window_main();
    };
}

