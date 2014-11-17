use std::any::{Any, AnyRefExt};
use sync::{RWLock, Arc};
use object;

pub struct Operation
{
    object : Arc<RWLock<object::Object>>,
    name : String,
    old : Box<Any>,
    new : Box<Any>,
}

impl Operation
{
    pub fn new(
        object : Arc<RWLock<object::Object>>,
        name : String,
        old : Box<Any>,
        new : Box<Any>) -> Operation
    {
        Operation {
            object : object,
            name : name,
            old : old,
            new : new
        }

    }
}

pub struct OperationManager
{
    pub operations : Vec<Operation>
}

impl OperationManager
{
    pub fn new() -> OperationManager
    {
        OperationManager {
            operations : Vec::new()
        }
    }
}
