use std::any::{Any, AnyRefExt};
use std::sync::{RWLock, Arc};
use std::cell::RefCell;
use std::rc::Weak;
use std::fmt;

use object;
use property;
use property::PropertyWrite;
use ui;
use control::WidgetUpdate;

pub struct Operation
{
    pub object : Arc<RWLock<object::Object>>,
    pub name : Vec<String>,
    pub old : Box<Any>,
    pub new : Box<Any>,
}

impl Operation
{
    pub fn new(
        object : Arc<RWLock<object::Object>>,
        name : Vec<String>,
        old : Box<Any>,
        new : Box<Any>) -> Operation
    {
        Operation {
            object : object,
            name : name,
            old : old,
            new : new
        }
    }

    pub fn apply(&self)
    {
        let o = &self.object;

        /*
        let v: Vec<&str> = self.name.split('/').collect();
        let mut vs = Vec::new();
        for i in v.iter()
        {
            vs.push(i.to_string());
        }

        //let vs = vs.tail().to_vec();
        */

        //o.write().set_property_hier(vs, &*self.old);
        println!("apply {}, {}", self.name, self.new);
        //o.write().set_property_hier(self.name.clone(), &*self.new);
        o.write().test_set_property_hier(join_string(&self.name).as_slice(), &*self.new);
        println!("applied {}", self.name);

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
        let o = &self.object;

        let vs = self.name.tail().to_vec();

        //o.write().set_property_hier(self.name.clone(), &*self.old);
        //o.write().set_property_hier(vs, &*self.old);
        o.write().test_set_property_hier(join_string(&vs).as_slice(), &*self.old);
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
        self.undo.push(op);
    }

    pub fn pop_undo(&mut self) -> Option<Operation>
    {
        self.undo.pop()
    }

    /*
    pub fn undo(&mut self)
    {
        let op = match self.undo.pop() {
            Some(o) => o,
            None => return
        };

        op.undo();
    }
    */

    pub fn redo(&mut self)
    {
        let op = match self.redo.pop() {
            Some(o) => o,
            None => return
        };

        op.apply();
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

