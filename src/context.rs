use object;
use scene;

use std::collections::{DList};
use std::sync::{RwLock, Arc};

pub struct Context
{
    pub selected : DList<Arc<RwLock<object::Object>>>,
    pub scene : Option<Arc<RwLock<scene::Scene>>>,
}

impl Context
{
    pub fn new() -> Context
    {
        Context {
            selected: DList::new(),
            scene : None
        }
    }
}

