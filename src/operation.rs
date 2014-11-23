use std::any::{Any, AnyRefExt};
use sync::{RWLock, Arc};
use std::cell::RefCell;
use std::rc::Weak;
use std::fmt;

use object;
use property;
use property::ChrisProperty;
use ui;
use ui::WidgetUpdate;

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

trait AnyClone: Any + Clone {
}
impl<T: Any + Clone> AnyClone for T {}

pub struct OperationManager
{
    pub undo : Vec<Operation>,
    pub redo : Vec<Operation>,
    //pub onUndo : Option<fn test
    //pub closure: Option<|&str,Any+Clone|:'static -> ()>,
    //pub closure: Option<|&str|:'static>,
    //pub closure: Option<|&str|:'a>
    //pub closure: Option<|&str, &Any|:'a>
    //pub closure: Option<|&str|:'static>
    //pub master : Option<Weak<RefCell<ui::Master>>>,
    //pub master : Option<&'a ui::Master<'a>>
}

impl OperationManager
{
    pub fn new(
        //master : Weak<RefCell<ui::Master>>
        ) -> OperationManager
    {
        OperationManager {
            undo : Vec::new(),
            redo : Vec::new(),
            //closure : None
            //master: None
        }
    }

    pub fn add(&mut self, op : Operation)
    {
        let captured_var = 10i;

        let t = |t : &Any|  {};
        //let t = |t : &Any+Clone|  {};
        
        //let closure_args = |arg: int,yep : &Any+Clone| -> int {
        let closure_args = |arg: int| -> int {
            println!("captured_var={}, arg={}", captured_var, arg);
            arg // Note lack of semicolon after 'arg'
        };
        self.undo.push(op);
    }

    pub fn undo(&mut self)
    {
        let op = match self.undo.pop() {
            Some(o) => o,
            None => return
        };

        op.undo();

        /*
        match self.master {
            Some(ref masterr) => {
                match masterr.upgrade() {
                    Some(m) => { 
                        match m.try_borrow_mut() {
                            Some(ref mut mm) => {
                                let s = join_string(&op.name);
                                mm.update_changed(s.as_slice(), &*op.new);
                                //mm.update_changed(s.as_slice(), &1f32);
                            },
                            _ => { println!("already borrowed : operation undo")}
                        }
                    },
                    None => { println!("the master of the operation doesn't exist anymore");}
                }
            },
            None => {
                println!("no master !!!!!!!!");
            }
        }
        */
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
