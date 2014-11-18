use std::any::{Any, AnyRefExt};
use sync::{RWLock, Arc};
use object;
use property;
use property::ChrisProperty;

pub struct Operation
{
    object : Arc<RWLock<object::Object>>,
    name : Vec<String>,
    old : Box<Any>,
    new : Box<Any>,
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

        //o.write().cset_property_hier(vs, &*self.old);
        println!("apply {}, {}", self.name, self.new);
        o.write().cset_property_hier(self.name.clone(), &*self.new);
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
        o.write().cset_property_hier(self.name.clone(), &*self.old);
    }
}

pub struct OperationManager
{
    pub undo : Vec<Operation>,
    pub redo : Vec<Operation>
}

impl OperationManager
{
    pub fn new() -> OperationManager
    {
        OperationManager {
            undo : Vec::new(),
            redo : Vec::new()
        }
    }

    pub fn add(&mut self, op : Operation)
    {
        self.undo.push(op);
    }

    pub fn undo(&mut self)
    {
        let op = match self.undo.pop() {
            Some(o) => o,
            None => return
        };

        op.undo();
    }

    pub fn redo(&mut self)
    {
        let op = match self.redo.pop() {
            Some(o) => o,
            None => return
        };

        op.apply();
    }
}
