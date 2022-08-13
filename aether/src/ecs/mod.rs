use serde::{Deserialize, Serialize};
use std::{borrow::BorrowMut, fmt::Debug};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub enum Component {
    Tag(String),
    Position { x: f32, y: f32 },
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
pub struct EntityID(Uuid);

#[derive(Debug, Deserialize, Serialize)]
pub struct Entity {
    id: EntityID,
    components: Vec<Component>,
}

impl Entity {
    #[must_use]
    #[allow(clippy::borrowed_box)]
    pub fn get_component<T>(&self) -> Option<&Component> {
        self.components
            .iter()
            .find(|component| matches!(component, T))
    }

    pub fn add_component(&mut self, component: Component) {
        self.components.push(component);
    }
}

pub trait System: Debug {
    fn run(&self, world: &World, entity: &Entity);
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct World {
    entities: Vec<Entity>,
    #[serde(skip)]
    systems: Vec<Box<dyn System>>,
}

impl EntityID {
    pub fn execute(&self, world: &mut World, func: fn(&mut Entity)) -> Option<()> {
        let entity = world
            .entities
            .iter_mut()
            .find(|entity| entity.id == *self)?;
        func(entity);
        Some(())
    }
}

impl World {
    #[must_use]
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            systems: Vec::new(),
        }
    }

    pub fn new_entity(&mut self) -> EntityID {
        let entity = Entity {
            id: EntityID(Uuid::new_v4()),
            components: Vec::new(),
        };
        let id = entity.id;
        self.entities.push(entity);
        id
    }

    pub fn tick(&self) {
        for system in &self.systems {
            for entity in &self.entities {
                system.run(self, entity);
            }
        }
    }
}
