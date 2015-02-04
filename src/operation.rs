use std::any::{Any};//, AnyRefExt};
use std::sync::{RwLock, Arc};
use std::cell::RefCell;
use std::rc::Weak;
use std::fmt;
use std::collections::DList;
use uuid;

use object;
use property;
use property::PropertyWrite;
use ui;
use control::WidgetUpdate;

pub struct Operation
{
    pub objects : DList<Arc<RwLock<object::Object>>>,
    pub name : Vec<String>,
    pub old : Box<Any>,
    pub new : Box<Any>,
}

#[derive(PartialEq)]
pub enum Change
{
    None,
    Property,
    Tree,
    Objects(String, DList<uuid::Uuid>),
    DirectChange(String),
    RectVisibleSet(bool),
    RectSet(f32, f32, f32, f32),
    SelectedChange,
    All
}


impl Operation
{
    pub fn new(
        objects : DList<Arc<RwLock<object::Object>>>,
        name : Vec<String>,
        old : Box<Any>,
        new : Box<Any>) -> Operation
    {
        Operation {
            objects : objects,
            name : name,
            old : old,
            new : new
        }
    }

    pub fn apply(&self)
    {
        /*
        let v: Vec<&str> = self.name.split('/').collect();
        let mut vs = Vec::new();
        for i in v.iter()
        {
            vs.push(i.to_string());
        }

        //let vs = vs.tail().to_vec();
        */

        for o in self.objects.iter() {
        //o.write().set_property_hier(vs, &*self.old);
        println!("apply {:?}, {:?}", self.name, self.new);
        //o.write().set_property_hier(self.name.clone(), &*self.new);
        o.write().unwrap().test_set_property_hier(join_string(&self.name).as_slice(), &*self.new);
        println!("applied {:?}", self.name);

        }

        match self.new.downcast_ref::<String>() {
            Some(s) => println!("applay with value {}" , s),
            None => {}
        };
    }

    pub fn undo(&self)
    {
        match self.old.downcast_ref::<String>() {
            Some(s) => println!("undo with value {}" , s),
            None => {}
        };

        let vs = self.name.tail().to_vec();

        for o in self.objects.iter() {
            //o.write().set_property_hier(self.name.clone(), &*self.old);
            //o.write().set_property_hier(vs, &*self.old);
            o.write().unwrap().test_set_property_hier(join_string(&vs).as_slice(), &*self.old);
        }
    }
}

trait AnyClone: Any + Clone {
}
impl<T: Any + Clone> AnyClone for T {}

pub struct OperationManager
{
    pub undo : Vec<Operation>,
    pub redo : Vec<Operation>,
}

impl OperationManager
{
    pub fn new(
        ) -> OperationManager
    {
        OperationManager {
            undo : Vec::new(),
            redo : Vec::new(),
        }
    }

    pub fn add(&mut self, op : Operation)
    {
        self.add_undo(op);
        self.redo.clear();
    }

    fn add_undo(&mut self, op : Operation)
    {
        self.undo.push(op);
    }

    fn add_redo(&mut self, op : Operation)
    {
        self.redo.push(op);
    }


    fn pop_undo(&mut self) -> Option<Operation>
    {
        self.undo.pop()
    }

    fn pop_redo(&mut self) -> Option<Operation>
    {
        self.redo.pop()
    }

    pub fn undo(&mut self) -> Change
    {
        let op = match self.pop_undo() {
            Some(o) => o,
            None => {
                println!("nothing to undo");
                return Change::None;
            }
        };

        op.undo();

        let mut list = DList::new();
        for o in op.objects.iter() {
            list.push_back(o.read().unwrap().id.clone());
        }

        let s = join_string(&op.name);

        self.add_redo(op);

        return Change::Objects(s,list);
    }

    pub fn redo(&mut self) -> Change
    {
        let op = match self.pop_redo() {
            Some(o) => o,
            None => {
                println!("nothing to redo");
                return Change::None;
            }
        };

        op.apply();

        let mut list = DList::new();
        for o in op.objects.iter() {
            list.push_back(o.read().unwrap().id.clone());
        }

        let s = join_string(&op.name);

        self.add_undo(op);

        return Change::Objects(s,list);
    }


}

//TODO remove
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

