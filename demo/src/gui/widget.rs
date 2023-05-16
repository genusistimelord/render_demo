use crate::{SystemEvent, UIBuffer, WidgetEvent, UI};
use graphics::*;
use hecs::Entity;
use input::FrameTime;
use std::{
    any::Any,
    cell::RefCell,
    collections::VecDeque,
    ops::{Deref, DerefMut},
    rc::Rc,
    sync::RwLock,
    vec::Vec,
};
use ubits::bitfield;
use wgpu::StencilFaceState;
use winit::event::{KeyboardInput, ModifiersState};

#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub struct Handle(pub(crate) Entity);

impl Handle {
    pub fn get_key(&self) -> Entity {
        self.0
    }
}

impl Default for Handle {
    fn default() -> Self {
        Self(Entity::DANGLING)
    }
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct Identity {
    pub name: String,
    pub id: u64,
}

impl Identity {
    pub fn new(name: &str, id: u64) -> Self {
        Self {
            name: name.to_owned(),
            id,
        }
    }
}

bitfield! {
    pub u16 UiField
    UiFlags {
        0 : IsFocused,
        1 : CanFocus,
        2 : MouseOver,
        3 : MoveAble,
        4 : Moving,
        5 : CanClickBehind,
        6 : AlwaysUseable,
        7 : Minimized,
        8 : Checked,
        9 : FocusClick,
        10 : IsPassword,
        11 : CanMoveWindow,
        12 : Clicked,
        13 : ClickAble,
        14 : AllowChildren,
        15 : InnerScroll,
    }
}

pub trait Control<Message> {
    /// Widgets Name and user given ID All widgets must contain this.
    fn get_id(&self) -> &Identity;

    fn check_mouse_bounds(&self, mouse_pos: Vec2) -> bool;

    fn get_bounds(&self) -> Option<WorldBounds>;

    fn get_size(&self) -> Vec2;

    fn get_position(&mut self) -> Vec3;

    fn event(
        &mut self,
        actions: UiField,
        ui_buffer: &mut UIBuffer,
        renderer: &mut GpuRenderer,
        event: SystemEvent,
        events: &mut Vec<Message>,
    ) -> WidgetEvent;

    fn draw(
        &mut self,
        ui_buffer: &mut UIBuffer,
        renderer: &mut GpuRenderer,
        frametime: &FrameTime,
    ) -> Result<(), AscendingError>;

    fn default_actions(&self) -> UiField;
}

pub trait AnyData<Message>: Control<Message> {
    fn as_any(&self) -> &dyn Any;
    fn as_mut_any(&mut self) -> &mut dyn Any;
}

impl<Message, U: Any + Control<Message>> AnyData<Message> for U {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Default)]
pub struct Parent(pub Handle);

impl Parent {
    pub fn get_key(&self) -> Entity {
        self.0.get_key()
    }

    pub fn get_id(&self) -> Handle {
        self.0
    }
}

#[derive(Default)]
pub struct Actions(pub UiField);

impl Actions {
    pub fn get(&self) -> &UiField {
        &self.0
    }

    pub fn get_mut(&mut self) -> &mut UiField {
        &mut self.0
    }

    pub fn exists(&self, flag: UiFlags) -> bool {
        self.0.get(flag)
    }

    pub fn set(&mut self, flag: UiFlags) {
        self.0.set(flag);
    }

    pub fn clear(&mut self, flag: UiFlags) {
        self.0.clear(flag);
    }
}

pub struct Hidden;

pub struct WidgetAny<Message: 'static>(
    pub Box<dyn AnyData<Message> + Send + Sync>,
);

impl<Message: 'static> Deref for WidgetAny<Message> {
    type Target = Box<dyn AnyData<Message> + Send + Sync>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<Message: 'static> DerefMut for WidgetAny<Message> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct Widget;

// TODO: Make Bounds Updater that will Update all the internal Bounds based on
#[derive(Default)]
pub struct WidgetBounds(pub WorldBounds);

impl WidgetBounds {
    pub fn get(&self) -> &WorldBounds {
        &self.0
    }

    pub fn get_mut(&mut self) -> &mut WorldBounds {
        &mut self.0
    }
}

// TODO: Make Bounds of before any major clipping occurs so it can be reset back.
#[derive(Default)]
pub struct OriginalBounds(pub WorldBounds);

impl OriginalBounds {
    pub fn get(&self) -> &WorldBounds {
        &self.0
    }

    pub fn get_mut(&mut self) -> &mut WorldBounds {
        &mut self.0
    }
}
