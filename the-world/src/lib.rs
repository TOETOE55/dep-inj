//! Recording all component-apis

use std::ops::Deref;

pub trait TheWorld<Component>:
    // all api-xxx
    even_api::Even +
    odd_api::Odd +
    // share between thread safety
    Send +
    Sync +
    dyn_clone::DynClone
{
    fn project(&self) -> &Component;
}

dyn_clone::clone_trait_object!(<Component> TheWorld<Component>);

pub struct ContainerRef<Component> {
    the_world: Box<dyn TheWorld<Component>>,
    _dummy: (), // other components
}

impl<Component> Clone for ContainerRef<Component> {
    fn clone(&self) -> Self {
        Self {
            the_world: self.the_world.clone(),
            _dummy: ()
        }
    }
}

impl<Component> Deref for ContainerRef<Component> {
    type Target = Component;

    fn deref(&self) -> &Self::Target {
        self.the_world.project()
    }
}

impl<Component> ContainerRef<Component> {
    pub fn new(the_world: Box<dyn TheWorld<Component>>) -> Self {
        Self {
            the_world,
            _dummy: ()
        }
    }

    pub fn the_world(&self) -> &(dyn TheWorld<Component> + 'static) {
        &*self.the_world
    }
}