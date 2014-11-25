use object;
use scene;

use std::collections::{DList};
use sync::{RWLock, Arc};

pub struct Context
{
    pub selected : DList<Arc<RWLock<object::Object>>>,
    pub scene : Option<Arc<RWLock<scene::Scene>>>,
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

