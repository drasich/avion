use std::rc::Rc;
use std::cell::RefCell;
use std::any::{Any,AnyRefExt};
use sync::{RWLock, Arc};

use uuid::Uuid;

use operation;
use camera;
use context;
use ui;
//use ui::property;
//use ui::tree;
use intersection;
use object;
use property;
use property::ChrisProperty;

pub enum MasterState
{
    Idle,
    CameraRotation
}

pub struct Control
{
    pub op_mgr : operation::OperationManager,
    pub camera : Rc<RefCell<camera::Camera>>,
    pub state : MasterState,
    pub context : Rc<RefCell<context::Context>>,

    //TODO control listener
    //pub property : Option<Rc<RefCell<ui::Property>>>, //TODO change to weak
    //pub tree : Option<Rc<RefCell<ui::Tree>>>, //TODO change to weak
    pub property : Option<Rc<RefCell<Box<ui::Property>>>>, //TODO change to weak
    pub tree : Option<Rc<RefCell<Box<ui::Tree>>>>, //TODO change to weak
}

impl Control
{
    pub fn new(
        camera : Rc<RefCell<camera::Camera>>,
        context : Rc<RefCell<context::Context>>,
        ) -> Control
    {
        Control {
            op_mgr : operation::OperationManager::new(),
            camera : camera,
            property : None,
            tree : None,
            state : Idle,
            context : context
        }
    }

    pub fn mouse_up(
            &mut self, 
            button : i32,
            x : i32, 
            y : i32,
            timestamp : i32)
    {
        println!("control fn mouse up");
        match self.state {
            CameraRotation => {
                self.state = Idle;
                return;
            },
            _ => {}
        }

        //println!("rust mouse up button {}, pos: {}, {}", button, x, y);
        let r = match self.camera.try_borrow(){
            Some(c) => {
                c.ray_from_screen(x as f64, y as f64, 10000f64)
            },
            None => { println!("cannot borrow camera"); return; }
        };

        //TODO
        /*
        match m.render.line.write().mesh_render {
            Some (ref mr) => {
                match mr.mesh.resource {
                    resource::ResData(ref mesh) => {
                        mesh.write().add_line(
                            geometry::Segment::new(r.start, r.start + r.direction),
                            vec::Vec4::zero()); },
                            _ => {}
                }
            },
            None => {}
        }
        */

        let mut c = match self.context.try_borrow_mut(){
            Some(con) => con,
            None => { println!("cannot borrow context"); return; }
        };

        c.selected.clear();

        let scene = match c.scene {
            Some(ref s) => s.clone(),
            None => return
        };

        for o in scene.read().objects.iter() {
            let ir = intersection::ray_object(&r, &*o.read());
            if ir.hit {
                println!(" I hit object {} ", o.read().name);
                c.selected.push(o.clone());
            }
        }

        if c.selected.len() == 1 {
            //TODO select tree
            match (c.selected.front()) {
                Some(o) => {
                    match self.property {
                        Some(ref pp) => {
                            match pp.try_borrow_mut() {
                                Some(ref mut p) => unsafe {
                                    p.set_object(&*o.read());
                                },
                                None => {println!("cannot borrow property");}
                            };
                        },
                        None => {
                            println!("control no property");
                        }
                    }

                    match self.tree {
                        Some(ref tt) => {
                            match tt.try_borrow_mut() {
                                Some(ref mut t) => {
                                    t.select(&o.read().id);
                                }
                                _ => {}
                            }
                        },
                        None => {
                            println!("control no tree");
                        }
                    }
                },
                _ => {},
            }
        }
    }

    pub fn select(&mut self, id : &Uuid)
    {
        //TODO same as the code at the end of mouse_up, so factorize
        println!("control .................select is called : {} ", id);
        let mut c = match self.context.try_borrow_mut(){
            Some(con) => con,
            None => { println!("cannot borrow context"); return; }
        };

        c.selected.clear();

        let scene = match c.scene {
            Some(ref s) => s.clone(),
            None => return
        };

        for o in scene.read().objects.iter() {
            if o.read().id == *id {
                c.selected.push(o.clone());
                match self.property {
                    Some(ref mut pp) =>
                        match pp.try_borrow_mut() {
                            Some(ref mut p) => {
                                p.set_object(&*o.read());
                            },
                            None=> {}
                        },
                    None => {}
                }
                break;
            }
        }
    }

    pub fn request_operation<T : Any+PartialEq>(
        &mut self,  
        name : Vec<String>,
        old : Box<T>,
        new : Box<T>)
    {
        if *old == *new {
            return;
        }

        let mut c = match self.context.try_borrow_mut(){
            Some(con) => con,
            None => { println!("cannot borrow context"); return; }
        };

        let o = match c.selected.front() {
            Some(o) => o.clone(),
            None => {
                println!("no objetcs selected");
                return;
            }
        };

        let op = operation::Operation::new(
            o.clone(), 
            name,
            old,
            new); 

        op.apply();

        //TODO update the widget that has this object/property

        self.op_mgr.add(op);
    }

    pub fn request_direct_change(
        &mut self,  
        name : Vec<String>,
        new : &Any)
    {
        let mut c = match self.context.try_borrow_mut(){
            Some(con) => con,
            None => { println!("cannot borrow context"); return; }
        };

        let o = match c.selected.front() {
            Some(o) => o.clone(),
            None => {
                println!("no objetcs selected");
                return;
            }
        };

        let vs = name.tail().to_vec();

        //TODO update the widget that has this object/property

        o.write().cset_property_hier(vs, new);
    }

    pub fn get_selected_object(&self) -> Option<Arc<RWLock<object::Object>>>
    {
        let mut c = match self.context.try_borrow_mut(){
            Some(con) => con,
            None => { println!("cannot borrow context"); return None; }
        };

        match c.selected.front() {
            Some(o) => return Some(o.clone()),
            None => {
                println!("no objetcs selected");
                return None;
            }
        };

    }

    //TODO move this out of control?
    pub fn request_display_property(
        &self,
        property : &mut ui::Property,
        name : Vec<String>,
        )
    {
        //TODO remove property tail
        let yep = name.tail().to_vec();

        match self.get_selected_object() {
            Some(o) => {
                match property::find_property(&*o.read(), yep.clone()) {
                    Some(ppp) => {
                        property.create_entries(&*ppp, name.clone());
                    },
                    None => {
                        println!("could not find property {} ", name);
                    }
                }
            },
            None => {
                println!("no objetcs selected");
            }
        }
    }

    pub fn undo(&mut self)
    {
        let op = match self.op_mgr.pop_undo() {
            Some(o) => o,
            None => return
        };

        op.undo();

        let s = join_string(&op.name);

        match self.get_selected_object() {
            Some(o) => {
                if op.object.read().id == o.read().id  {
                    match self.property {
                        Some(ref mut pp) =>
                            match pp.try_borrow_mut() {
                                Some(ref mut p) => {
                                    println!("join string : {}", s);
                                    p.update_changed(s.as_slice(), &*op.old);
                                },
                                None=> {}
                            },
                            None => {}
                    };
                }
            },
            None => {
            }
        }

        if s.as_slice() == "object/name" {
            match self.tree {
                Some(ref mut tt) =>
                    match tt.try_borrow_mut() {
                        Some(ref mut t) => {
                            t.update_object(&op.object.read().id);
                        },
                        None=> {}
                    },
                    None => {}
            };
        }


    }
}

fn join_string(path : &Vec<String>) -> String
{
    let mut s = String::new();
    let mut first = true;
    for v in path.iter() {
        if !first {
            s.push('/');
        }
        s.push_str(v.as_slice());
        first = false;
    }

    s
}

pub trait WidgetUpdate {

    //fn update_changed<T : Any+Clone>(
    fn update_changed(
        &mut self,
        name : &str,
        //old : Option<&T>,
        new : &Any);
}

