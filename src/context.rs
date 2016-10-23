use dormin::object;
use dormin::scene;
use dormin::vec;
use uuid;
use dormin::transform;

use std::collections::{LinkedList};
use std::sync::{RwLock, Arc};
use std::rc::Rc;
use std::cell::{RefCell, BorrowState};

pub struct Context
{
    pub selected : Vec<Arc<RwLock<object::Object>>>,
    pub scene : Option<Rc<RefCell<scene::Scene>>>,
    pub saved_positions : Vec<vec::Vec3>,
    pub saved_scales : Vec<vec::Vec3>,
    pub saved_oris : Vec<transform::Orientation>
}

impl Context
{
    pub fn new() -> Context
    {
        Context {
            selected: Vec::new(),
            scene : None,
            saved_positions : Vec::new(),
            saved_scales : Vec::new(),
            saved_oris : Vec::new()
        }
    }

    pub fn set_scene(&mut self, scene : Rc<RefCell<scene::Scene>>)
    {
        self.scene = Some(scene);
        self.selected.clear();
    }

    pub fn get_scene(&self) -> Option<Rc<RefCell<scene::Scene>>>
    {
        self.scene.clone()
    }

    pub fn save_positions(&mut self)
    {
        self.saved_positions.clear();
        for o in &self.selected {
            self.saved_positions.push(o.read().unwrap().position);
        }
    }

    pub fn save_scales(&mut self)
    {
        self.saved_scales.clear();
        for o in &self.selected {
            self.saved_scales.push(o.read().unwrap().scale);
        }
    }

    pub fn save_oris(&mut self)
    {
        self.saved_oris.clear();
        for o in &self.selected {
            self.saved_oris.push(o.read().unwrap().orientation);
        }
    }

    pub fn get_selected_ids(&self) -> LinkedList<uuid::Uuid>
    {
        let mut list = LinkedList::new();
        for o in &self.selected {
            list.push_back(o.read().unwrap().id.clone());
        }

        list
    }

    pub fn get_vec_selected_ids(&self) -> Vec<uuid::Uuid>
    {
        let mut v = Vec::with_capacity(self.selected.len());
        for o in &self.selected {
            v.push(o.read().unwrap().id.clone());
        }

        v
    }

    pub fn remove_objects_by_id(&mut self, ids : Vec<uuid::Uuid>)
    {
        let mut new_list = Vec::new();
        for o in &self.selected {
            let mut not_found = true;
            for id in &ids {
                if *id == o.read().unwrap().id {
                    not_found = false;
                    break;
                }
            }
            if not_found {
                new_list.push(o.clone());
            }
        }

        self.selected = new_list;
    }

    pub fn add_objects_by_id(&mut self, ids : Vec<uuid::Uuid>)
    {
        for id in &ids {
            let mut found = false;
            for o in &self.selected {
                if *id == o.read().unwrap().id {
                    found = true;
                    break;
                }
            }
            if !found {
                if let Some(ref s) = self.scene {
                    for so in &s.borrow().objects {
                        if *id == so.read().unwrap().id {
                            self.selected.push(so.clone());
                            break;
                        }
                    }
                }
            }
        }
    }

    pub fn has_object_with_id(&self, id : &uuid::Uuid) -> bool
    {
        for o in &self.selected {
            if *id == o.read().unwrap().id {
                return true;
            }
        }

        false
    }

    pub fn has_object(&self, ob : &object::Object) -> bool
    {
        for o in &self.selected {
            if ob.id == o.read().unwrap().id {
                return true;
            }
        }

        false
    }

}

