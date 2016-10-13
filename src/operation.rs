use std::any::{Any};//, AnyRefExt};
use std::sync::{RwLock, Arc};
use std::cell::RefCell;
use std::rc::{Rc,Weak};
use std::fmt;
use std::collections::LinkedList;
use uuid;

use dormin::object;
use dormin::property;
use dormin::property::PropertyWrite;
use ui;
use ui::PropertyUser;
use control::WidgetUpdate;
use dormin::vec;
use dormin::scene;
use dormin::component::CompData;
use ui::RefMut;

use dragger;

pub trait OperationTrait
{
    fn apply(&self) -> Change;
    fn undo(&self) -> Change;
}

pub enum OperationData
{
    //ToNone(Box<Any>),
    //ToSome,
    VecAdd(usize),
    VecDel(usize, Box<Any>),
    Function(fn(LinkedList<Arc<RwLock<object::Object>>>, Box<Any>), Box<Any>),
    List(LinkedList<Box<Any>>, LinkedList<Box<Any>>),
    Vector(Vec<Box<Any>>, Vec<Box<Any>>),
    SceneAddObjects(Rc<RefCell<scene::Scene>>, Vec<uuid::Uuid>, Vec<Arc<RwLock<object::Object>>>), //scene, parent, objects
    SceneRemoveObjects(Rc<RefCell<scene::Scene>>, Vec<uuid::Uuid>, Vec<Arc<RwLock<object::Object>>>),
    SetSceneCamera(
        Rc<RefCell<scene::Scene>>,
        Option<Arc<RwLock<object::Object>>>,
        Option<Arc<RwLock<object::Object>>>),
    //AddComponent(uuid::Uuid, uuid::Uuid) //object id, component id?
    AddComponent(Arc<RwLock<object::Object>>, Box<CompData>),
    OldNewVec(Vec<Box<Any>>, Box<Any>),

}

pub struct Operation
{
    pub objects : Vec<Arc<RwLock<object::Object>>>,
    pub name : Vec<String>,
    pub change : OperationData
    //pub old : Box<Any>,
    //pub new : Box<Any>,
}

pub enum OperationActor{
    Scene(uuid::Uuid),
    Object(uuid::Uuid),
    Objects(Vec<uuid::Uuid>),
    Ref(RefMut<PropertyWrite>),
    //PropertyWrite(&PropertyWrite),
}

pub struct OperationNew
{
    pub actor : OperationActor,
    pub name : String,
    pub change : OperationData
    //pub old : Box<Any>,
    //pub new : Box<Any>,
}

pub struct OldNew{
    pub object : RefMut<PropertyUser>,
    pub name : String,
    pub old : Box<Any>,
    pub new : Box<Any>
}

impl OldNew
{
    pub fn new(
        object : RefMut<PropertyUser>,
        name : String,
        old : Box<Any>,
        new : Box<Any>
        ) -> OldNew
    {
        OldNew{
            object : object,
            name : name,
            old : old,
            new : new
        }
    }

}

impl OperationTrait for OldNew
{
    fn apply(&self) -> Change
    {
        println!("NEW TEST operation set property hier {:?}", self.name);
        match self.object {
            RefMut::Arc(ref a) => {
                a.write().unwrap().test_set_property_hier(self.name.as_ref(), &*self.new);
            },
            RefMut::Cell(ref c) => { 
                c.borrow_mut().test_set_property_hier(self.name.as_ref(), &*self.new);
            }
        }

        Change::Property(self.object.clone(), self.name.clone())
    }

    fn undo(&self) -> Change
    {
        match self.object {
            RefMut::Arc(ref a) => {
                a.write().unwrap().test_set_property_hier(self.name.as_ref(), &*self.old);
            },
            RefMut::Cell(ref c) => { 
                c.borrow_mut().test_set_property_hier(self.name.as_ref(), &*self.old);
            }
        }

        Change::Property(self.object.clone(), self.name.clone())
    }
}

pub struct ToNone{
    pub object : RefMut<PropertyUser>,
    pub name : String,
    pub old : Box<Any>,
}

impl ToNone
{
    pub fn new(
        object : RefMut<PropertyUser>,
        name : String,
        old : Box<Any>
        ) -> ToNone
    {
        ToNone{
            object : object,
            name : name,
            old : old,
        }
    }
}

impl OperationTrait for ToNone
{
    fn apply(&self) -> Change
    {
        println!("TO NONE operation set property hier {:?}", self.name);
        match self.object {
            RefMut::Arc(ref a) => {
                a.write().unwrap().set_property_hier(self.name.as_ref(), property::WriteValue::None);
            },
            RefMut::Cell(ref c) => { 
                c.borrow_mut().set_property_hier(self.name.as_ref(), property::WriteValue::None);
            }
        }

        Change::Property(self.object.clone(), self.name.clone())
    }

    fn undo(&self) -> Change
    {
        match self.object {
            RefMut::Arc(ref a) => {
                a.write().unwrap().test_set_property_hier(self.name.as_ref(), &*self.old);
            },
            RefMut::Cell(ref c) => { 
                c.borrow_mut().test_set_property_hier(self.name.as_ref(), &*self.old);
            }
        }

        Change::Property(self.object.clone(), self.name.clone())
    }
}

pub struct ToSome{
    pub object : RefMut<PropertyUser>,
    pub name : String,
}

impl ToSome
{
    pub fn new(
        object : RefMut<PropertyUser>,
        name : String
        ) -> ToSome
    {
        ToSome{
            object : object,
            name : name,
        }
    }
}

impl OperationTrait for ToSome
{
    fn apply(&self) -> Change
    {
        println!("TO Some operation set property hier {:?}", self.name);
        match self.object {
            RefMut::Arc(ref a) => {
                a.write().unwrap().set_property_hier(self.name.as_ref(), property::WriteValue::Some);
            },
            RefMut::Cell(ref c) => { 
                c.borrow_mut().set_property_hier(self.name.as_ref(), property::WriteValue::Some);
            }
        }

        Change::Property(self.object.clone(), self.name.clone())
    }

    fn undo(&self) -> Change
    {
        match self.object {
            RefMut::Arc(ref a) => {
                a.write().unwrap().set_property_hier(self.name.as_ref(), property::WriteValue::None);
            },
            RefMut::Cell(ref c) => { 
                c.borrow_mut().set_property_hier(self.name.as_ref(), property::WriteValue::None);
            }
        }

        Change::Property(self.object.clone(), self.name.clone())
    }
}



//#[derive(PartialEq)]
pub enum Change
{
    None,
    Property(RefMut<PropertyUser>, String),
    Tree,
    Objects(String, LinkedList<uuid::Uuid>),
    DirectChange(String),
    ChangeSelected(Vec<Arc<RwLock<object::Object>>>),
    SelectedChange,
    SceneAdd(uuid::Uuid, Vec<uuid::Uuid>, Vec<uuid::Uuid>),
    SceneRemove(uuid::Uuid, Vec<uuid::Uuid>, Vec<uuid::Uuid>),
    Scene(uuid::Uuid),
    ComponentChanged(uuid::Uuid, String),

    VecAdd(Vec<uuid::Uuid>, String, usize),
    VecDel(Vec<uuid::Uuid>, String, usize),

    RectVisibleSet(bool),
    RectSet(f32, f32, f32, f32),
    DraggerClicked,
    DraggerTranslation(vec::Vec3),
    DraggerScale(vec::Vec3),
    DraggerRotation(vec::Quat),

    DraggerOperation(dragger::Operation),
    Undo,
    Redo,
    CameraChange,
    DraggerChange,

    All
}

impl Operation
{
    pub fn new(
        objects : Vec<Arc<RwLock<object::Object>>>,
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
            /*
            OperationData::ToNone(_) => {
                println!("to none, apply,  operation set property hier {:?}", self.name);
                let s = join_string(&self.name);
                let mut ids = LinkedList::new();
                for o in &self.objects {
                    let mut ob = o.write().unwrap();
                    ob.set_property_hier(s.as_ref(), property::WriteValue::None);
                    ids.push_back(ob.id.clone());
                }
                return Change::Objects(s, ids);
            },
            */
            OperationData::VecAdd(i) => {
                println!("vec add operation {:?}, {}", self.name,i);
                let s = join_string(&self.name);
                let mut ids = Vec::new();
                for o in &self.objects {
                    let mut ob = o.write().unwrap();
                    ob.add_item(s.as_ref(), i, &String::from("empty"));
                    ids.push(ob.id.clone());
                }
                return Change::VecAdd(ids, s, i);
            },
            OperationData::VecDel(i,_) => {
                println!("vec del operation {:?}", self.name);
                let s = join_string(&self.name);
                let mut ids = Vec::new();
                for o in &self.objects {
                    let mut ob = o.write().unwrap();
                    //TODO chris
                    println!("yeeeeeeeeeeeee call del_item : {}", ob.name);
                    ob.del_item(s.as_ref(), i);
                    ids.push(ob.id.clone());
                }
                return Change::VecDel(ids, s, i);
            },
            /*
            OperationData::ToSome => {
                println!("to some, apply,  operation set property hier {:?}", self.name);
                let s = join_string(&self.name);
                let mut ids = LinkedList::new();
                for o in &self.objects {
                    let mut ob = o.write().unwrap();
                    ob.set_property_hier(s.as_ref(), property::WriteValue::Some);
                    ids.push_back(ob.id.clone());
                }
                return Change::Objects(s, ids);
            },
            */
            OperationData::Vector(_,ref new) => {
                let mut i = 0;
                let s = join_string(&self.name);
                let sp = if !self.name.is_empty() && self.name.last().unwrap() == "*" {
                    let mut sp = self.name.clone();
                    sp.pop();
                    join_string(&sp)
                }
                else {
                    s.clone()
                };
                let mut ids = LinkedList::new();
                for o in &self.objects {
                    let mut ob = o.write().unwrap();
                    ob.test_set_property_hier(
                        sp.as_str(),
                        &*new[i]);
                    i = i +1;
                    ids.push_back(ob.id.clone());
                }
                return Change::Objects(s, ids);
            },
            OperationData::SceneAddObjects(ref s, ref parents, ref obs)  => {
                let mut sc = s.borrow_mut();
                sc.add_objects(parents, obs);
                return Change::SceneAdd(sc.id.clone(), parents.clone(), get_ids(obs));
            },
            OperationData::SceneRemoveObjects(ref s, ref parents, ref obs)  => {
                let mut sc = s.borrow_mut();
                sc.remove_objects(parents, obs);
                return Change::SceneRemove(sc.id.clone(), parents.clone(), get_ids(obs));
            },
            OperationData::SetSceneCamera(ref s, _, ref new)   => {
                println!("operation set camera");
                let sc = s.borrow();
                if let Some(ref c) = sc.camera {
                    if let Some(ref o) = *new {
                        println!("I set thhe camera !!!!!!!");
                        c.borrow_mut().object = o.clone();
                        c.borrow_mut().object_id = Some(o.read().unwrap().id.clone());
                        return Change::Scene(sc.id.clone());
                    }
                    else {
                        println!("dame 10");
                        c.borrow_mut().object_id = None;
                    }
                }
                else {
                    println!("dame 00");
                }
            },
            OperationData::AddComponent(ref o, ref compo)  => {
                let mut ob = o.write().unwrap();
                ob.add_comp_data(compo.clone());
                return Change::ComponentChanged(ob.id.clone(), compo.get_kind_string());
            },
            _ => {}
        }

        Change::None
    }

    fn undo(&self) -> Change
    {
        match self.change {
            /*
            OperationData::ToNone(ref old) => {
                println!("to none, undo, operation set property hier {:?}", self.name);
                let s = join_string(&self.name);
                let mut ids = LinkedList::new();
                for o in &self.objects {
                    let mut ob = o.write().unwrap();
                    ob.test_set_property_hier(s.as_ref(), &**old);
                    ids.push_back(ob.id.clone());
                }
                return Change::Objects(s, ids);
            },
            OperationData::ToSome => {
                println!("to some, undo,  operation set property hier {:?}", self.name);
                let s = join_string(&self.name);
                let mut ids = LinkedList::new();
                for o in &self.objects {
                    let mut ob = o.write().unwrap();
                    ob.set_property_hier(s.as_ref(), property::WriteValue::None);
                    ids.push_back(ob.id.clone());
                }
                return Change::Objects(s, ids);
            },
            */
            OperationData::VecAdd(i) => {
                println!("vec add operation undo {:?}, {}", self.name, i);
                let s = join_string(&self.name);
                let mut ids = Vec::new();
                for o in &self.objects {
                    let mut ob = o.write().unwrap();
                    ob.del_item(s.as_ref(), i);
                    ids.push(ob.id.clone());
                }
                return Change::VecDel(ids, s, i);
            },
            OperationData::VecDel(i,ref value) => {
                println!("vec del operation undo {:?}", self.name);
                let s = join_string(&self.name);
                let mut ids = Vec::new();
                for o in &self.objects {
                    let mut ob = o.write().unwrap();
                    ob.add_item(s.as_ref(), i, &**value);
                    ids.push(ob.id.clone());
                }
                //return Change::Objects(s, ids);
                return Change::VecAdd(ids, s, i);
            },
            OperationData::Vector(ref old,_) => {
                let mut i = 0;
                let s = join_string(&self.name);
                let sp = if !self.name.is_empty() && self.name.last().unwrap() == "*" {
                    let mut sp = self.name.clone();
                    sp.pop();
                    join_string(&sp)
                }
                else {
                    s.clone()
                };
                let mut ids = LinkedList::new();
                for o in &self.objects {
                    let mut ob = o.write().unwrap();
                    ob.test_set_property_hier(
                        sp.as_str(),
                        &*old[i]);
                    i = i +1;
                    ids.push_back(ob.id.clone());
                }
                return Change::Objects(s, ids);
            },
            OperationData::SceneAddObjects(ref s, ref parents, ref obs)  => {
                println!("undo scene add objects !!!");
                let mut sc = s.borrow_mut();
                sc.remove_objects(parents, obs);
                return Change::SceneRemove(sc.id.clone(), parents.clone(), get_ids(obs));
            },
            OperationData::SceneRemoveObjects(ref s, ref parents, ref obs)  => {
                println!("undo scene remove objects !!!");
                let mut sc = s.borrow_mut();
                sc.add_objects(parents, obs);
                return Change::SceneAdd(sc.id.clone(), parents.clone(), get_ids(obs));
            },
            OperationData::SetSceneCamera(ref s, ref old, _)   => {
                let sc = s.borrow();
                if let Some(ref c) = sc.camera {
                    if let Some(ref o) = *old {
                        c.borrow_mut().object = o.clone();
                        c.borrow_mut().object_id = Some(o.read().unwrap().id.clone());
                        return Change::Scene(sc.id.clone());
                    }
                    else {
                        c.borrow_mut().object_id = None
                    }
                }
            },
            OperationData::AddComponent(ref o, ref compo)  => {
                let mut ob = o.write().unwrap();
                ob.remove_comp_data(compo.clone());
                return Change::ComponentChanged(ob.id.clone(), compo.get_kind_string());
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

    pub fn add(&mut self, op : Operation) -> Change
    {
        let change = op.apply();
        self.add_undo(box op);
        self.redo.clear();

        change
    }

    pub fn add_with_trait(&mut self, op : Box<OperationTrait>) -> Change
    {
        let change = op.apply();
        self.add_undo(op);
        self.redo.clear();

        change
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
fn join_string(path : &[String]) -> String
{
    let mut s = String::new();
    let mut first = true;
    for v in path {
        if !first {
            s.push('/');
        }
        s.push_str(v.as_ref());
        first = false;
    }

    s
}

fn get_ids(obs : &[Arc<RwLock<object::Object>>]) -> Vec<uuid::Uuid>
{
    let mut list = Vec::new();
    for o in obs {
        list.push(o.read().unwrap().id.clone());
    }

    list
}
