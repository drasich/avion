use std::any::{Any, AnyRefExt};

pub struct Operation
{
    name : String,
    old : Box<Any>,
    new : Box<Any>,
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
