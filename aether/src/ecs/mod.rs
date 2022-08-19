use serde::{Deserialize, Serialize};
use std::fmt::Debug;
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
    children: Vec<EntityID>,
}

impl Entity {
    #[must_use]
    #[allow(unused_variables)] // Weirdly detects the generic T type as an unused variable
    #[allow(non_snake_case)]
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
    pub fn execute<F>(&self, world: &mut World, func: F) -> Option<()>
    where
        F: Fn(&mut Entity),
    {
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

    pub fn new_entity(&mut self, parent: Option<EntityID>) -> EntityID {
        let entity = Entity {
            id: EntityID(Uuid::new_v4()),
            components: Vec::new(),
            children: Vec::new(),
        };

        if let Some(p) = parent {
            p.execute(self, |e| e.children.push(entity.id));
        }

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
