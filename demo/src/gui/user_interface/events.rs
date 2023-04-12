use crate::{
    Actions, FrameTime, GpuRenderer, Handle, Identity, Parent, SystemEvent,
    UIBuffer, UiFlags, Widget, WidgetAny, UI,
};
use graphics::*;
use hecs::World;
use slab::Slab;
use std::{
    any::Any,
    cell::RefCell,
    collections::{HashMap, VecDeque},
    marker::PhantomData,
    rc::Rc,
    vec::Vec,
};
use winit::{
    dpi::PhysicalPosition,
    event::{
        DeviceEvent, ElementState, Event, KeyboardInput, ModifiersState,
        MouseButton, MouseScrollDelta, WindowEvent,
    },
    window::Window,
};

impl<Message> UI<Message> {
    pub fn event_draw(
        &mut self,
        world: &mut World,
        ui_buffer: &mut UIBuffer,
        renderer: &mut GpuRenderer,
        time: &FrameTime,
    ) -> Result<(), AscendingError> {
        for handle in &self.zlist.clone() {
            let mut ui = world
                .get::<&mut WidgetAny<Message>>(handle.get_key())
                .expect("Widget is missing its inner UI Type?");

            ui.draw(ui_buffer, renderer, time)?;
        }

        ui_buffer.ui_buffer.finalize(renderer);
        ui_buffer.text_renderer.finalize(renderer);
        Ok(())
    }

    pub fn event_mouse_position(
        &mut self,
        world: &mut World,
        ui_buffer: &mut UIBuffer,
        renderer: &mut GpuRenderer,
        position: Vec2,
        screensize: Vec2,
        events: &mut Vec<Message>,
    ) {
        self.new_mouse_pos = position;

        if self.moving {
            if let Ok(win_pos) = renderer.window().outer_position() {
                let mut win_pos = Vec2::new(win_pos.x as f32, win_pos.y as f32);
                win_pos.x += position.x - self.mouse_clicked.x;
                win_pos.y += (position.y - self.mouse_clicked.y) * -1.0;
                renderer.window_mut().set_outer_position(
                    PhysicalPosition::new(win_pos.x, win_pos.y),
                );
            } else {
                panic!("Not Supported. This will be a Soft warning via log later on.")
            }
        } else {
            if let Some(handle) = self.focused {
                let action = world
                    .get::<&Actions>(handle.get_key())
                    .expect("Widget is missing its actions?")
                    .0;

                if action.get(UiFlags::Moving) {
                    let parent_bounds = if let Some(parent) = world
                        .get::<&Parent>(handle.get_key())
                        .ok()
                        .map(|p| p.get_id())
                    {
                        let ui = world
                            .get::<&WidgetAny<Message>>(parent.get_key())
                            .expect("Widget is missing its inner UI Type?");

                        ui.get_bounds()
                    } else {
                        Vec4::new(0.0, 0.0, screensize.x, screensize.y)
                    };

                    let pos = Vec2::new(
                        position.x - self.mouse_pos.x,
                        (position.y - self.mouse_pos.y) * -1.0,
                    );

                    let bounds;

                    {
                        let mut ui = world
                            .get::<&mut WidgetAny<Message>>(handle.get_key())
                            .expect("Widget is missing its inner UI Type?");

                        bounds = ui.get_bounds();

                        if bounds.x + pos.x <= parent_bounds.x
                            || bounds.y + pos.y <= parent_bounds.y
                            || bounds.x + bounds.z + pos.x >= parent_bounds.z
                            || bounds.y + bounds.w + pos.y >= parent_bounds.w
                        {
                            return;
                        }

                        let mut control_pos = ui.get_position();
                        control_pos.x += pos.x;
                        control_pos.y += pos.y;

                        ui.set_position(control_pos);
                        //todo ui.set_bounds();
                    }

                    self.widget_position_update(renderer, handle, pos, bounds);
                }
            }

            self.mouse_over_event(world, ui_buffer, renderer, events);
        }

        self.mouse_pos = position;
    }

    pub fn event_mouse_button(
        &mut self,
        world: &mut World,
        ui_buffer: &mut UIBuffer,
        renderer: &mut GpuRenderer,
        button: MouseButton,
        pressed: bool,
        events: &mut Vec<Message>,
    ) {
        self.button = button;
        self.mouse_clicked = self.mouse_pos;

        if pressed {
            self.mouse_press(world, ui_buffer, renderer, events);
        } else {
            self.mouse_release(world, ui_buffer, renderer, events);
        }
    }

    pub fn event_modifiers(&mut self, modifier: ModifiersState) {
        self.modifier = modifier;
    }

    pub fn handle_events(
        &mut self,
        world: &mut World,
        ui_buffer: &mut UIBuffer,
        renderer: &mut GpuRenderer,
        event: &Event<()>,
        hidpi: f32,
    ) -> Vec<Message> {
        let mut events: Vec<Message> = Vec::new();

        match *event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == renderer.window().id() => match event {
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: _,
                            virtual_keycode: Some(_key_code),
                            scancode: _,
                            ..
                        },
                    ..
                } => {}
                WindowEvent::MouseInput { state, button, .. } => {
                    let pressed = *state == ElementState::Pressed;
                    self.event_mouse_button(
                        world,
                        ui_buffer,
                        renderer,
                        *button,
                        pressed,
                        &mut events,
                    );
                }
                WindowEvent::CursorMoved {
                    position: PhysicalPosition { x, y },
                    ..
                } => {
                    let size = renderer.size();
                    let pos = Vec2::new(
                        (*x as f32) * hidpi,
                        size.height - ((*y as f32) * hidpi),
                    );
                    self.event_mouse_position(
                        world,
                        ui_buffer,
                        renderer,
                        pos,
                        Vec2::new(size.width, size.height),
                        &mut events,
                    );
                }
                _ => (),
            },
            Event::DeviceEvent { ref event, .. } => match *event {
                DeviceEvent::MouseMotion { delta: _ } => {}
                DeviceEvent::MouseWheel {
                    delta: MouseScrollDelta::LineDelta(_dx, _dy),
                } => {
                    /* if dx != 0.0 {
                        self.mouse_wheel.0 = dx.signum();
                    }

                    if dy != 0.0 {
                        self.mouse_wheel.1 = dy.signum();
                    }*/
                }
                DeviceEvent::MouseWheel {
                    delta:
                        MouseScrollDelta::PixelDelta(PhysicalPosition {
                            x: _,
                            y: _,
                        }),
                } => {
                    /*if x != 0.0 {
                        self.mouse_wheel.0 = x.signum() as f32;
                    }

                    if y != 0.0 {
                        self.mouse_wheel.1 = y.signum() as f32;
                    }*/
                }
                _ => (),
            },
            _ => (),
        }

        events
    }
}