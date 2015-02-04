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
use vec;

pub enum DataChange
{
    OldNew(Box<Any>, Box<Any>),
    Function(fn(DList<Arc<RwLock<object::Object>>>, Box<Any>), Box<Any>),
    List(DList<Box<Any>>, DList<Box<Any>>)
}

pub struct Operation
{
    pub objects : DList<Arc<RwLock<object::Object>>>,
    pub name : Vec<String>,
    pub change : DataChange
    //pub old : Box<Any>,
    //pub new : Box<Any>,
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

        /*
        fn fntest( objs : DList<Arc<RwLock<object::Object>>>, val : Box<Any>)
        {
        }

        let test = DataChange::Function(fntest, box 5);
        */

        match old.downcast_ref::<vec::Vec3>() {
            Some(v) => println!("old : {:?}", v),
            _ => {}
        }
        match new.downcast_ref::<vec::Vec3>() {
            Some(v) => println!("new : {:?}", v),
            _ => {}
        }

        let change = DataChange::OldNew(old, new);
        Operation {
            objects : objects,
            name : name,
            //old : old,
            //new : new
            change : change
        }
    }

    pub fn apply(&self)
    {
        for o in self.objects.iter() {
            match self.change {
                DataChange::OldNew(_,ref new) => {
                    //println!("apply {:?}, {:?}", self.name, self.new);
                    //o.write().unwrap().test_set_property_hier(join_string(&self.name).as_slice(), &*self.new);
                    o.write().unwrap().test_set_property_hier(join_string(&self.name).as_slice(), &**new);
                    println!("applied {:?}", self.name);
                },
                _ => {}
            }
        }

    }

    pub fn undo(&self)
    {
        //let vs = self.name.tail().to_vec();

        for o in self.objects.iter() {
            match self.change {
                DataChange::OldNew(ref old,_) => {
                    //o.write().set_property_hier(self.name.clone(), &*self.old);
                    //o.write().set_property_hier(vs, &*self.old);

                    //o.write().unwrap().test_set_property_hier(join_string(&vs).as_slice(), &**old);
                    o.write().unwrap().test_set_property_hier(join_string(&self.name).as_slice(), &**old);
                },
                _ => {}
            }
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

