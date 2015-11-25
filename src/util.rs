use std::sync::{RwLock, Arc};
use std::collections::LinkedList;

use vec;
use object;

pub fn objects_center(objects : &LinkedList<Arc<RwLock<object::Object>>>) -> vec::Vec3
{
    let mut v = vec::Vec3::zero();
    for o in objects.iter()
    {
        v = v + o.read().unwrap().position;
    }

    v = v / objects.len() as f64;

    v
}

