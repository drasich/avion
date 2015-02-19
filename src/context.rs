use object;
use scene;
use vec;
use uuid;
use transform;

use std::collections::{DList};
use std::sync::{RwLock, Arc};

pub struct Context
{
    pub selected : DList<Arc<RwLock<object::Object>>>,
    pub scene : Option<Arc<RwLock<scene::Scene>>>,
    pub saved_positions : Vec<vec::Vec3>,
    pub saved_scales : Vec<vec::Vec3>,
    pub saved_oris : Vec<transform::Orientation>
}

impl Context
{
    pub fn new() -> Context
    {
        Context {
            selected: DList::new(),
            scene : None,
            saved_positions : Vec::new(),
            saved_scales : Vec::new(),
            saved_oris : Vec::new()
        }
    }

    pub fn save_positions(&mut self)
    {
        self.saved_positions.clear();
        for o in self.selected.iter() {
            self.saved_positions.push(o.read().unwrap().position);
        }
    }

    pub fn save_scales(&mut self)
    {
        self.saved_scales.clear();
        for o in self.selected.iter() {
            self.saved_scales.push(o.read().unwrap().scale);
        }
    }

    pub fn save_oris(&mut self)
    {
        self.saved_oris.clear();
        for o in self.selected.iter() {
            self.saved_oris.push(o.read().unwrap().orientation);
        }
    }

    pub fn get_selected_ids(&self) -> DList<uuid::Uuid>
    {
        let mut list = DList::new();
        for o in self.selected.iter() {
            list.push_back(o.read().unwrap().id.clone());
        }

        list
    }

    pub fn remove(&mut self, ids : DList<uuid::Uuid>)
    {
        let mut new_list = DList::new();
        for o in self.selected.iter() {
            let mut not_found = true;
            for id in ids.iter() {
                if *id == o.read().unwrap().id {
                    not_found = false;
                    break;
                }
            }
            if not_found {
                new_list.push_back(o.clone());
            }
        }

        self.selected = new_list;
    }

}

