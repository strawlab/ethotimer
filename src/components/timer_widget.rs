use std::{cell::RefCell, rc::Rc, time::Duration};

use yew::prelude::*;
use yew::services::{IntervalService, Task};

#[derive(PartialEq, Clone)]
pub struct TimerStorage {
    rc: Rc<RefCell<TimerStorageInner>>,
}

impl yew::html::ImplicitClone for TimerStorage {}

impl TimerStorage {
    pub fn new() -> Self {
        Self {
            rc: Rc::new(RefCell::new(TimerStorageInner {
                prev_elapsed: Duration::from_secs(0),
                current_start: None,
            })),
        }
    }
}

impl TimerStorage {
    pub fn is_active(&self) -> bool {
        self.rc.borrow().current_start.is_some()
    }
    pub fn total_elapsed(&self) -> Duration {
        let stor = self.rc.borrow();

        let cur_dur = match &stor.current_start {
            Some(start) => start.elapsed(),
            None => Duration::from_secs(0),
        };
        stor.prev_elapsed + cur_dur
    }
    pub fn clear(&mut self) {
        let mut stor = self.rc.borrow_mut();
        stor.current_start = None;
        stor.prev_elapsed = Duration::from_secs(0);
    }

    pub fn stop(&mut self) {
        let mut stor = self.rc.borrow_mut();
        let start = stor.current_start.take();
        if let Some(start) = start {
            let dur = start.elapsed();
            stor.prev_elapsed += dur;
        }
    }
}

pub enum Msg {
    OnStart,
    RenderAll,
}

#[derive(PartialEq)]
struct TimerStorageInner {
    current_start: Option<instant::Instant>,
    prev_elapsed: Duration,
}

#[derive(PartialEq, Clone, Properties)]
pub struct Props {
    /// The backing store for the data.
    pub storage: TimerStorage,
    #[prop_or(true)]
    pub show_start_button: bool,
    /// Text to show.
    pub text: String,
    /// Triggered when the timer is started.
    #[prop_or_default]
    pub on_start: Option<Callback<()>>,
    #[prop_or_default]
    pub on_create: Option<Callback<ComponentLink<TimerWidget>>>, //Some(|child_link| Msg::SetChildLink(child_link)),
}

pub struct TimerWidget {
    link: ComponentLink<Self>,
    job: Option<Box<dyn Task>>,
    storage: TimerStorage,
    show_start_button: bool,
    text: String,
    on_start: Option<Callback<()>>,
}

impl Component for TimerWidget {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        if let Some(cb) = props.on_create {
            cb.emit(link.clone());
        }
        Self {
            link,
            job: None,
            storage: props.storage,
            show_start_button: props.show_start_button,
            text: props.text,
            on_start: props.on_start,
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.storage = props.storage;
        self.show_start_button = props.show_start_button;
        self.text = props.text;
        self.on_start = props.on_start;
        true
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::OnStart => {
                let did_start = {
                    let mut stor = self.storage.rc.borrow_mut();
                    if stor.current_start.is_none() {
                        stor.current_start = Some(instant::Instant::now());
                        true
                    } else {
                        false
                    }
                };

                if did_start {
                    if let Some(ref mut callback) = self.on_start {
                        callback.emit(());
                    }
                    let handle = IntervalService::spawn(
                        Duration::from_millis(100),
                        self.link.callback(|_| Msg::RenderAll),
                    );
                    self.job = Some(Box::new(handle));
                }
            }
            Msg::RenderAll => {
                // This triggers a rerender because ShouldRender is returned true.

                // Also check if we need to keep the timer running.
                if !self.storage.is_active() {
                    self.job = None;
                }
            }
        }
        true
    }

    fn view(&self) -> Html {
        let elapsed = format!(
            "{:4.1}",
            self.storage.total_elapsed().as_millis() as f64 / 1000.0
        );
        let start_button = if self.show_start_button {
            let stor = self.storage.rc.borrow();
            let is_active = stor.current_start.is_some();
            let mut classes = vec!["btn", "timer-start-btn"];
            if is_active {
                classes.push("btn-active");
            }
            html! {
                <button class=classes onclick=self.link.callback(|_| Msg::OnStart)>{ "Start ⏱" }</button>
            }
        } else {
            html! {}
        };
        html! {
            <div class="timer">
                {start_button}
                <div>
                    {&self.text}<span class="elapsed">{&elapsed}</span>
                </div>
            </div>
        }
    }
}
