use crate::{
    CallBack, CallBackKey, CallBacks, Commands, GuiRender, Handle, Identity,
    InternalCallBacks, UiFlags, Widget, WidgetRef,
};
use graphics::*;
use slab::Slab;
use std::{
    any::Any,
    cell::RefCell,
    collections::{HashMap, VecDeque},
    marker::PhantomData,
    rc::Rc,
    vec::Vec,
};
use winit::event::{KeyboardInput, ModifiersState};
use winit::window::Window;

#[derive(Default)]
pub struct Widgets<T> {
    /// Callback mapper. Hashes must be different.
    callbacks: HashMap<CallBackKey, InternalCallBacks>,
    user_callbacks: HashMap<CallBackKey, CallBacks<T>>,
    name_map: HashMap<Identity, Handle>,
    widgets: Slab<WidgetRef>,
    ///Contains All Visible widgets in rendering order
    zlist: VecDeque<Handle>,
    ///The Visible Top widgets.
    visible: VecDeque<Handle>,
    ///The loaded but hidden Top children.
    hidden: Vec<Handle>,
    focused: Option<Handle>,
    over: Option<Handle>,
    clicked: Option<Handle>,
    mouse_clicked: [i32; 2],
    mouse_pos: [i32; 2],
    new_mouse_pos: [i32; 2],
    moving: bool,
    button: u32,
    modifier: ModifiersState,
}

impl<T> Widgets<T> {
    pub fn new() -> Self {
        Widgets {
            callbacks: HashMap::with_capacity(100),
            user_callbacks: HashMap::with_capacity(100),
            name_map: HashMap::with_capacity(100),
            widgets: Slab::with_capacity(100),
            zlist: VecDeque::with_capacity(100),
            visible: VecDeque::with_capacity(100),
            hidden: Vec::with_capacity(100),
            focused: Option::None,
            over: Option::None,
            clicked: Option::None,
            mouse_clicked: [0; 2],
            mouse_pos: [0; 2],
            new_mouse_pos: [0; 2],
            moving: false,
            button: 0,
            modifier: ModifiersState::default(),
        }
    }

    fn get_widget(&self, handle: Handle) -> WidgetRef {
        self.widgets
            .get(handle.get_key())
            .expect("ID Existed but widget does not exist?")
            .clone()
    }

    pub fn event_mouse_position(
        &mut self,
        window: &mut Window,
        position: [i32; 2],
        screensize: [i32; 2],
        _user_data: &mut T,
    ) {
        self.new_mouse_pos = position;

        if self.moving {
            if let Ok(mut win_pos) = window.outer_position() {
                win_pos.x = position[0] + win_pos.x - self.mouse_clicked[0];
                win_pos.y = position[1] + win_pos.y - self.mouse_clicked[1];
                window.set_outer_position(win_pos);
            } else {
                panic!("Not Supported. This will be a Soft warning via log later on.")
            }
        } else if let Some(handle) = self.focused {
            let focused = self.get_widget(handle);

            if focused.borrow().actions.get(UiFlags::Moving) {
                let pos = [
                    position[0] - self.mouse_pos[0],
                    position[1] - self.mouse_pos[1],
                ];
                let mut bounds = focused.borrow().ui.get_bounds();

                if bounds.0 + pos[0] <= 0
                    || bounds.1 + pos[1] <= 0
                    || bounds.0 + bounds.2 + pos[0] >= screensize[0]
                    || bounds.1 + bounds.3 + pos[1] >= screensize[1]
                {
                    return;
                }

                bounds.0 += pos[0];
                bounds.1 += pos[1];

                focused.borrow_mut().ui.set_position([bounds.0, bounds.1]);
                self.widget_position_update(&mut focused.borrow_mut());
            }
        }

        self.mouse_pos = position;
    }

    fn widget_manual_focus(&mut self, control: &WidgetRef) {
        if control.borrow().actions.get(UiFlags::CanFocus) {
            let handle = control.borrow().id;

            if let Some(pos) = self.zlist.iter().position(|x| *x == handle) {
                self.zlist.remove(pos);
                self.zlist.push_back(handle);
            }

            if let Some(parent_handle) = control.borrow().parent {
                let parent = self.get_widget(parent_handle);

                if let Some(pos) =
                    parent.borrow().iter().position(|x| *x == handle)
                {
                    parent.borrow_mut().children.remove(pos);
                    parent.borrow_mut().children.push_back(handle);
                }
            }

            if let Some(focused_handle) = self.focused {
                let focused = self.get_widget(parent_handle);
                self.widget_focused_callback(&focused, false);
            }

            control.borrow_mut().actions.set(UiFlags::IsFocused) = true;
            self.focused = Some(handle);
            self.widget_focused_callback(control, true);
        }
    }

    pub fn event_mouse_button(
        &mut self,
        button: u32,
        pressed: bool,
        user_data: &mut T,
    ) {
        self.button = button;
        self.mouse_clicked = self.mouse_pos;

        if pressed == 1 {
            self.mouse_press(user_data);
        } else {
            mouse_release(user_data);
        }
    }

    fn widget_focused_callback(&mut self, control: &WidgetRef, focused: bool) {
        let mut mut_wdgt = control.borrow_mut();
        let key = mut_wdgt.callback_key(CallBack::MousePress);

        mut_wdgt.actions.set(UiFlags::IsFocused) = true;
        self.focused = Some(handle);

        if let Some(InternalCallBacks::FocusChange(focus_changed)) =
            self.callbacks.get(&key)
        {
            focus_changed(&mut mut_wdgt, focused);
        }
    }

    fn widget_mouse_press_callbacks(
        &mut self,
        control: &WidgetRef,
        pressed: bool,
        user_data: &mut T,
    ) {
        let mut mut_wdgt = control.borrow_mut();
        let key = mut_wdgt.callback_key(CallBack::MousePress);
        let mut commands = Commands::new();

        if let Some(InternalCallBacks::MousePress(mouse_press)) =
            self.callbacks.get(&key)
        {
            mouse_press(&mut mut_wdgt, self.button, pressed, self.modifier);
        }

        if let Some(CallBacks::MousePress(mouse_press)) =
            self.user_callbacks.get(&key)
        {
            mouse_press(
                &mut mut_wdgt,
                self.button,
                pressed,
                self.modifier,
                &mut commands,
                user_data,
            );
        }
    }

    fn widget_set_clicked(&mut self, control: &WidgetRef, user_data: &mut T) {
        let in_bounds =
            control.borrow().ui.check_mouse_bounds(self.mouse_clicked);

        if control.borrow().actions.get(UiFlags::CanMoveWindow) && in_bounds {
            self.moving = true;
        }

        if (control.borrow().actions.get(UiFlags::CanClickBehind)) {
            if let Some(parent_handle) = control.borrow().parent {
                let parent = self.get_widget(parent_handle);

                if parent.borrow().actions.get(UiFlags::CanMoveWindow)
                    && parent.borrow().ui.check_mouse_bounds(self.mouse_clicked)
                {
                    self.moving = true;
                }

                self.clicked = Some(parent_handle);
                self.widget_mouse_press_callbacks(&parent, true, user_data);

                return;
            }

            return;
        }

        if control.borrow().actions.get(UiFlags::MoveAble) && in_bounds {
            control.borrow_mut().actions.set(UiFlags::Moving) = true;
        }

        self.widget_mouse_press_callbacks(&control, true, user_data);
    }

    fn widget_set_focus(&mut self, control: &WidgetRef, user_data: &mut T) {
        let handle = control.borrow().id;

        if let Some(pos) = self.zlist.iter().position(|x| *x == handle) {
            self.zlist.remove(pos);
            self.zlist.push_back(handle);
        }

        if let Some(parent_handle) = control.borrow().parent {
            let parent = self.get_widget(parent_handle);

            if let Some(pos) = parent.borrow().iter().position(|x| *x == handle)
            {
                parent.borrow_mut().children.remove(pos);
                parent.borrow_mut().children.push_back(handle);
            }
        }

        if let Some(focused_handle) = self.focused {
            let focused = self.get_widget(parent_handle);
            self.widget_focused_callback(&focused, false);
        }

        // Show Children()

        self.widget_focused_callback(control, true);
        self.widget_set_clicked(&control, user_data);
    }

    fn is_parent_focused(
        &mut self,
        control: &WidgetRef,
        user_data: &mut T,
    ) -> bool {
        if control.borrow().actions.get(UiFlags::AlwaysUseable) {
            return true;
        }

        let mut parent_opt = control.borrow().parent;

        while let Some(parent_handle) = parent_opt {
            let parent = self.get_widget(parent_handle);

            if parent.borrow().actions.get(UiFlags::CanFocus) {
                if parent.borrow().actions.get(UiFlags::CanFocus) {
                    return true;
                } else {
                    //setmanualfocus

                    if parent.borrow().actions.get(UiFlags::FocusClick) {
                        self.widget_set_clicked(&parent, user_data);
                    }

                    return true;
                }
            } else if parent.borrow().actions.get(UiFlags::AlwaysUseable)
                && parent.borrow().actions.get(UiFlags::ClickAble)
                && control.borrow().parent = Some(parent_handle)
                && control.borrow().actions.get(UiFlags::CanClickBehind)
            {
                return true;
            }

            parent_opt = parent.borrow().parent;
        }

        false
    }

    fn mouse_press_event(&mut self, control: &WidgetRef, user_data: &mut T) {
        if control.borrow().actions.get(UiFlags::CanFocus) {
            if self.focused != Some(control.borrow().id) {
                self.widget_set_focus(control, user_data);
            } else {
                self.widget_set_clicked(control, user_data);
            }
        } else if self.is_parent_focused(control, user_data) {
            self.widget_set_clicked(control, user_data);
        }
    }

    fn mouse_press(&mut self, user_data: &mut T) {
        for handle in self.zlist.iter().rev() {
            let child = self.get_widget(handle);

            if child.borrow().actions.get(UiFlags::ClickAble)
                && child.borrow().ui.check_mouse_bounds(self.mouse_clicked)
            {
                if child.borrow().actions.get(UiFlags::MoveAble) {
                    child.borrow_mut().actions.set(UiFlags::Moving) = false;
                }

                self.mouse_press_event(handle, user_data);
                return;
            }

            if child.borrow().actions.get(UiFlags::MoveAble)
                && child.borrow().ui.check_mouse_bounds(self.mouse_clicked)
            {
                child.borrow_mut().actions.set(UiFlags::Moving) = false;
            }
        }
    }

    fn mouse_release(&mut self, user_data: &mut T) {
        if let Some(focused_handle) = self.focused {
            let focused = self.get_widget(focused_handle);

            if focused.borrow().actions.get(UiFlags::Moving) {
                focused.borrow_mut().actions.set(UiFlags::Moving) = false;
            }
        }

        for handle in self.zlist.iter().rev() {
            let control = self.get_widget(handle);

            if control.borrow().actions.get(UiFlags::ClickAble)
                && control.borrow().ui.check_mouse_bounds(self.mouse_clicked)
            {
                if control.borrow().actions.get(UiFlags::CanMoveWindow) {
                    self.moving = false;
                }

                self.widget_mouse_press_callbacks(&control, false, user_data);
                return;
            }
        }
    }

    pub fn event_modifiers(&mut self, modifier: ModifiersState) {
        self.modifier = modifier;
    }

    pub fn clear_widgets(&mut self) {
        self.visible.clear();
        self.zlist.clear();
        self.hidden.clear();
        self.callbacks.clear();
        self.user_callbacks.clear();
        self.name_map.clear();
        self.widgets.clear();
        self.focused = None;
        self.over = None;
        self.clicked = None;
    }

    fn widget_position_update(&mut self, parent: &mut Widget) {
        let key = parent.callback_key(CallBack::PositionChange);

        if let Some(InternalCallBacks::PositionChange(internal_update_pos)) =
            self.callbacks.get(&key)
        {
            internal_update_pos(parent);
        }

        for handle in &parent.children {
            let widget = self.get_widget(*handle);

            if !widget.borrow().children.is_empty() {
                self.widget_position_update(&mut widget.borrow_mut());
            } else {
                let key =
                    widget.borrow().callback_key(CallBack::PositionChange);
                let mut mut_wdgt = widget.borrow_mut();

                if let Some(InternalCallBacks::PositionChange(
                    internal_update_pos,
                )) = self.callbacks.get(&key)
                {
                    internal_update_pos(&mut mut_wdgt);
                }
            }
        }
    }
}
