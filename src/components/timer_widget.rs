use std::{cell::RefCell, rc::Rc, time::Duration};

use gloo_timers::callback::Interval;
use yew::html::Scope;
use yew::prelude::*;

#[derive(PartialEq, Clone)]
pub struct TimerStorage {
    rc: Rc<RefCell<TimerStorageInner>>,
}

impl yew::html::ImplicitClone for TimerStorage {}

impl Default for TimerStorage {
    fn default() -> Self {
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
    pub on_create: Option<Callback<Scope<TimerWidget>>>,
}

pub struct TimerWidget {
    job: Option<Interval>,
}

impl Component for TimerWidget {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        if let Some(cb) = &ctx.props().on_create {
            cb.emit(ctx.link().clone());
        }
        Self { job: None }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::OnStart => {
                let did_start = {
                    let mut stor = ctx.props().storage.rc.borrow_mut();
                    if stor.current_start.is_none() {
                        stor.current_start = Some(instant::Instant::now());
                        true
                    } else {
                        false
                    }
                };

                if did_start {
                    if let Some(ref callback) = ctx.props().on_start {
                        callback.emit(());
                    }
                    let link = ctx.link().clone();
                    let handle = Interval::new(100, move || link.send_message(Msg::RenderAll));
                    self.job = Some(handle);
                }
            }
            Msg::RenderAll => {
                // This triggers a rerender because ShouldRender is returned true.

                // Also check if we need to keep the timer running.
                if !ctx.props().storage.is_active() {
                    self.job = None;
                }
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let elapsed = format!(
            "{:4.1}",
            ctx.props().storage.total_elapsed().as_millis() as f64 / 1000.0
        );
        let start_button = if ctx.props().show_start_button {
            let stor = ctx.props().storage.rc.borrow();
            let is_active = stor.current_start.is_some();
            let mut classes = vec!["btn", "timer-start-btn"];
            if is_active {
                classes.push("btn-active");
            }
            html! {
                <button class={classes} onclick={ctx.link().callback(|_| Msg::OnStart)}>{ "Start ⏱" }</button>
            }
        } else {
            html! {}
        };
        html! {
            <div class="timer">
                {start_button}
                <div>
                    {&ctx.props().text}<span class="elapsed">{&elapsed}</span>
                </div>
            </div>
        }
    }
}
