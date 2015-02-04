use object;
use scene;
use vec;
use uuid;

use std::collections::{DList};
use std::sync::{RwLock, Arc};

pub struct Context
{
    pub selected : DList<Arc<RwLock<object::Object>>>,
    pub scene : Option<Arc<RwLock<scene::Scene>>>,
    pub saved_positions : Vec<vec::Vec3>
}

impl Context
{
    pub fn new() -> Context
    {
        Context {
            selected: DList::new(),
            scene : None,
            saved_positions : Vec::new()
        }
    }

    pub fn save_positions(&mut self)
    {
        self.saved_positions.clear();
        for o in self.selected.iter() {
            self.saved_positions.push(o.read().unwrap().position);
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
}

