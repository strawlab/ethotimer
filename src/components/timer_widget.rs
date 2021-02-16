use std::{cell::RefCell, rc::Rc, time::Duration};

use yew::prelude::*;
use yew::services::{IntervalService, Task};

#[derive(PartialEq, Clone)]
pub struct TimerStorage {
    rc: Rc<RefCell<TimerStorageInner>>,
}

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

impl TimerStorageInner {
    fn total_elapsed(&self) -> Duration {
        let cur_dur = match &self.current_start {
            Some(start) => start.elapsed(),
            None => Duration::from_secs(0),
        };
        self.prev_elapsed + cur_dur
    }
}

#[derive(PartialEq, Clone, Properties)]
pub struct Props {
    /// The backing store for the data.
    pub storage: TimerStorage,
    /// Triggered when the timer is started.
    #[prop_or_default]
    pub on_start: Option<Callback<()>>,
}

pub struct TimerWidget {
    link: ComponentLink<Self>,
    job: Option<Box<dyn Task>>,
    storage: TimerStorage,
    on_start: Option<Callback<()>>,
}

impl Component for TimerWidget {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            job: None,
            storage: props.storage,
            on_start: props.on_start,
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.storage = props.storage;
        self.on_start = props.on_start;
        true
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::OnStart => {
                let mut stor = self.storage.rc.borrow_mut();
                if stor.current_start.is_none() {
                    stor.current_start = Some(instant::Instant::now());

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
                let stor = self.storage.rc.borrow();
                if stor.current_start.is_none() {
                    self.job = None;
                }
            }
        }
        true
    }

    fn view(&self) -> Html {
        let elapsed = format!(
            "{:4.1}",
            self.storage.rc.borrow().total_elapsed().as_millis() as f64 / 1000.0
        );
        html! {
            <div class="timer">
                <div>
                    <button class=("btn",), onclick=self.link.callback(|_| Msg::OnStart),>{ "Start ⏱" }</button>
                </div>
                <div>
                    <span class="elapsed">{&elapsed}</span>
                </div>
            </div>
        }
    }
}
