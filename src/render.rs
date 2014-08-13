extern crate libc;

use std::sync::Arc;
use std::ptr;
use std::mem;
use std::collections::{DList,Deque};
use self::libc::{c_void};

use shader;
use mesh;
use object;

pub struct Drawable
{
    shader: *const shader::Shader,
    buffer: *const mesh::Buffer
}

pub struct CypherList
{
    data : *const c_void,
    next : *const CypherList
}

#[link(name = "cypher")]
extern {
    pub fn draw_data_set(
        cb: extern fn(*const Render) -> *const Drawable,
        render: *const Render
        ) -> ();
}


pub struct RenderPass
{
    pub name : String,
    //pub material : Arc<shader::Material>,
    pub material : Box<shader::Material>,
    pub objects : DList<object::Object>,
    pub drawables : DList<Drawable>
}

impl RenderPass
{
    pub fn canDraw(&self) -> bool
    {
        return !self.drawables.is_empty();
    }

    pub fn getDrawable(&self) -> *const Drawable
    {
        match self.drawables.front() {
            Some(d) => unsafe {return mem::transmute(&d)},
            None => return ptr::null()
        }
    }

    pub fn getDrawables(&self) -> *const CypherList
    {
        let mut cl = box CypherList{ data : ptr::null(), next : ptr::null()};
        /*
        {

        //let mut parent = &mut *cl;
        let mut parent : Option<&CypherList> = None;
        let mut current = &mut *cl;

        let first = true;
        if !self.drawables.is_empty() {

            let mut it = self.drawables.iter();

            loop {

                match it.next() {
                    None => break,
                    Some(ref dr) => {
                        if first {
                            unsafe {
                                current.data = mem::transmute(dr)
                            }
                            first = false;
                        }
                        else {
                            current = &mut * box CypherList{data : unsafe{mem::transmute(dr)}, next : ptr::null()};
                        }

                        match parent {
                            None => {


                            },
                            Some(li) => 
                                unsafe {
                                    li.next = mem::transmute(current)
                                }
                        }

                        parent = Some(&*current);
                    }
                }

            }
        }

        }
        */

        unsafe {
            return mem::transmute(&*cl);
        }
    }

    pub fn draw(&mut self)
    {
        if (*(self.material)).shader == None {
            return;
        }

        let s : *const shader::Shader;
        match (*self.material).shader  {
            None => return,
            Some(sh) => s = sh
        }

        if !self.drawables.is_empty() {
            return; //for now... TODO
        }

        self.drawables.clear();
        for o in self.objects.iter() {
            match  o.mesh  {
                None => continue,
                Some(ref m) => {
                    match m.buffer {
                        None => { continue;}
                        Some(b) => self.drawables.push( Drawable{ shader: s, buffer :b})
                    }
                }
            }
        }

    }

    pub fn init(&mut self)
    {
        for o in self.objects.mut_iter() {
            match (*o).mesh {
                None => continue,
                Some(ref mut m) => if m.state == 0 {
                    mesh::mesh_buffer_init(m);
                }
            }
        }
    }
}

pub struct Render
{
    pub pass : Box<RenderPass>
}

impl Render{
    pub fn init(&mut self)
    {
        shader::material_shader_init(&mut *(*self.pass).material);

        (*self.pass).init();
    }

    pub fn draw(&mut self)
    {
        loop {
            (*self.pass).draw();
        }
    }

    pub fn getDrawable(&self) -> *const Drawable
    {
        return (*self.pass).getDrawable();
    }

}

