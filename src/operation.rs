use std::any::{Any};//, AnyRefExt};
use std::sync::{RwLock, Arc};
use std::cell::RefCell;
use std::rc::Weak;
use std::fmt;
use std::collections::LinkedList;
use uuid;

use object;
use property;
use property::PropertyWrite;
use ui;
use control::WidgetUpdate;
use vec;
use scene;

pub trait OperationTrait
{
    fn apply(&self) -> Change;
    fn undo(&self) -> Change;
}

pub enum OperationData
{
    OldNew(Box<Any>, Box<Any>),
    ToNone(Box<Any>),
    ToSome,
    Function(fn(LinkedList<Arc<RwLock<object::Object>>>, Box<Any>), Box<Any>),
    List(LinkedList<Box<Any>>, LinkedList<Box<Any>>),
    Vector(Vec<Box<Any>>, Vec<Box<Any>>),
    SceneAddObjects(Arc<RwLock<scene::Scene>>,Vec<Arc<RwLock<object::Object>>>)
}

pub struct Operation
{
    pub objects : LinkedList<Arc<RwLock<object::Object>>>,
    pub name : Vec<String>,
    pub change : OperationData
    //pub old : Box<Any>,
    //pub new : Box<Any>,
}

#[derive(PartialEq)]
pub enum Change
{
    None,
    Property,
    Tree,
    Objects(String, LinkedList<uuid::Uuid>),
    DirectChange(String),
    RectVisibleSet(bool),
    RectSet(f32, f32, f32, f32),
    SelectedChange,
    SceneAdd(uuid::Uuid, Vec<uuid::Uuid>),
    SceneRemove(uuid::Uuid, Vec<uuid::Uuid>),
    All
}

impl Operation
{
    pub fn new(
        objects : LinkedList<Arc<RwLock<object::Object>>>,
        name : Vec<String>,
        //old : Box<Any>,
        //new : Box<Any>) 
        change : OperationData
            )
        -> Operation
    {

        /*
        fn fntest( objs : LinkedList<Arc<RwLock<object::Object>>>, val : Box<Any>)
        {
        }

        let test = OperationData::Function(fntest, box 5);
        */

        /*
        if let OperationData::OldNew(ref old,ref new) = change {
        match old.downcast_ref::<vec::Vec3>() {
            Some(v) => println!("old : {:?}", v),
            _ => {}
        }
        match new.downcast_ref::<vec::Vec3>() {
            Some(v) => println!("new : {:?}", v),
            _ => {}
        }
        }
        */

        //let change = OperationData::OldNew(old, new);
        Operation {
            objects : objects,
            name : name,
            //old : old,
            //new : new
            change : change
        }
    }

}

impl OperationTrait for Operation
{
    fn apply(&self) -> Change
    {
        match self.change {
            OperationData::OldNew(_,ref new) => {
                println!("operation set property hier {:?}", self.name);
                let s = join_string(&self.name);
                let mut ids = LinkedList::new();
                for o in self.objects.iter() {
                    let mut ob = o.write().unwrap();
                    ob.test_set_property_hier(s.as_slice(), &**new);
                    ids.push_back(ob.id.clone());
                }
                return Change::Objects(s, ids);
            },
            OperationData::ToNone(_) => {
                println!("to none, apply,  operation set property hier {:?}", self.name);
                let s = join_string(&self.name);
                let mut ids = LinkedList::new();
                for o in self.objects.iter() {
                    let mut ob = o.write().unwrap();
                    ob.set_property_hier(s.as_slice(), property::WriteValue::None);
                    ids.push_back(ob.id.clone());
                }
                return Change::Objects(s, ids);
            },
            OperationData::ToSome => {
                println!("to some, apply,  operation set property hier {:?}", self.name);
                let s = join_string(&self.name);
                let mut ids = LinkedList::new();
                for o in self.objects.iter() {
                    let mut ob = o.write().unwrap();
                    ob.set_property_hier(s.as_slice(), property::WriteValue::Some);
                    ids.push_back(ob.id.clone());
                }
                return Change::Objects(s, ids);
            },
            OperationData::Vector(_,ref new) => {
                let mut i = 0;
                let s = join_string(&self.name);
                let mut ids = LinkedList::new();
                for o in self.objects.iter() {
                    let mut ob = o.write().unwrap();
                    ob.test_set_property_hier(
                        s.as_slice(),
                        &*new[i]);
                    i = i +1;
                    ids.push_back(ob.id.clone());
                }
                return Change::Objects(s, ids);
            },
            OperationData::SceneAddObjects(ref s, ref obs)  => {
                let mut sc = s.write().unwrap();
                sc.add_objects_by_vec(obs.clone());
                return Change::SceneAdd(sc.id.clone(), get_ids(obs));
            },
            _ => {}
        }

        Change::None
    }

    fn undo(&self) -> Change
    {
        match self.change {
            OperationData::OldNew(ref old,_) => {
                let s = join_string(&self.name);
                let mut ids = LinkedList::new();
                for o in self.objects.iter() {
                    let mut ob = o.write().unwrap();
                    ob.test_set_property_hier(s.as_slice(), &**old);
                    ids.push_back(ob.id.clone());
                }
                return Change::Objects(s, ids);
            },
            OperationData::ToNone(ref old) => {
                println!("to none, undo, operation set property hier {:?}", self.name);
                let s = join_string(&self.name);
                let mut ids = LinkedList::new();
                for o in self.objects.iter() {
                    let mut ob = o.write().unwrap();
                    ob.test_set_property_hier(s.as_slice(), &**old);
                    ids.push_back(ob.id.clone());
                }
                return Change::Objects(s, ids);
            },
            OperationData::ToSome => {
                println!("to some, undo,  operation set property hier {:?}", self.name);
                let s = join_string(&self.name);
                let mut ids = LinkedList::new();
                for o in self.objects.iter() {
                    let mut ob = o.write().unwrap();
                    ob.set_property_hier(s.as_slice(), property::WriteValue::None);
                    ids.push_back(ob.id.clone());
                }
                return Change::Objects(s, ids);
            },
            OperationData::Vector(ref old,_) => {
                let mut i = 0;
                let s = join_string(&self.name);
                let mut ids = LinkedList::new();
                for o in self.objects.iter() {
                    let mut ob = o.write().unwrap();
                    ob.test_set_property_hier(
                        s.as_slice(),
                        &*old[i]);
                    i = i +1;
                    ids.push_back(ob.id.clone());
                }
                return Change::Objects(s, ids);
            },
            OperationData::SceneAddObjects(ref s, ref obs)  => {
                println!("undo scene add objects !!!");
                let mut sc = s.write().unwrap();
                sc.remove_objects_by_vec(obs.clone());
                return Change::SceneRemove(sc.id.clone(), get_ids(obs));
            },
            _ => {}
        }

        Change::None
    }
}

trait AnyClone: Any + Clone {
}
impl<T: Any + Clone> AnyClone for T {}

pub struct OperationManager
{
    //pub undo : Vec<Operation>,
    //pub redo : Vec<Operation>,
    pub undo : Vec<Box<OperationTrait+'static>>,
    pub redo : Vec<Box<OperationTrait+'static>>,
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
        op.apply();
        self.add_undo(box op);
        self.redo.clear();
    }

    fn add_undo(&mut self, op : Box<OperationTrait+'static>)
    {
        self.undo.push(op);
    }

    fn add_redo(&mut self, op : Box<OperationTrait+'static>)
    {
        self.redo.push(op);
    }


    fn pop_undo(&mut self) -> Option<Box<OperationTrait+'static>>
    {
        self.undo.pop()
    }

    fn pop_redo(&mut self) -> Option<Box<OperationTrait+'static>>
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

        let change = op.undo();

        self.add_redo(op);

        return change;
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

        let change = op.apply();

        self.add_undo(op);
        return change;
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

fn get_ids(obs : &Vec<Arc<RwLock<object::Object>>>) -> Vec<uuid::Uuid>
{
    let mut list = Vec::new();
    for o in obs.iter() {
        list.push(o.read().unwrap().id.clone());
    }

    list
}
