use object;

use std::collections::{DList};
use sync::{RWLock, Arc};

pub struct Context
{
    pub objects : DList<Arc<RWLock<object::Object>>>
}

