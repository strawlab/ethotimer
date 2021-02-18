#![recursion_limit = "1024"]

use wasm_bindgen::prelude::*;

use yew::prelude::*;

use serde::{Deserialize, Serialize};

mod components;

use components::timer_widget::{TimerStorage, TimerWidget};

// -----------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
struct MyError {}

impl From<std::num::ParseIntError> for MyError {
    fn from(_orig: std::num::ParseIntError) -> MyError {
        MyError {}
    }
}

impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "err")
    }
}

// -----------------------------------------------------------------------------

struct Model {
    link: ComponentLink<Self>,
    timer1: TimerStorage,
    timer2: TimerStorage,
    timer3: TimerStorage,
}

#[derive(Clone)]
pub enum Msg {
    Timer1Start,
    Timer2Start,
    Timer3Start,
    StopAll,
    Clear,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            timer1: TimerStorage::new(),
            timer2: TimerStorage::new(),
            timer3: TimerStorage::new(),
        }
    }

    fn change(&mut self, _: ()) -> ShouldRender {
        false
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Timer1Start => {
                self.timer2.stop();
                self.timer3.stop();
            }
            Msg::Timer2Start => {
                self.timer1.stop();
                self.timer3.stop();
            }
            Msg::Timer3Start => {
                self.timer1.stop();
                self.timer2.stop();
            }
            Msg::StopAll => {
                self.timer1.stop();
                self.timer2.stop();
                self.timer3.stop();
            }
            Msg::Clear => {
                self.timer1.clear();
                self.timer2.clear();
                self.timer3.clear();
            }
        }
        true
    }

    fn view(&self) -> Html {
        html! {
            <div id="page-container",>
                <div id="content-wrap",>
                    <h1>{"⏱ ethotimer"}</h1>
                    <section class="timers">
                        <TimerWidget
                            storage=&self.timer1,
                            on_start=self.link.callback(|_| Msg::Timer1Start),
                            />
                        <TimerWidget
                            storage=&self.timer2,
                            on_start=self.link.callback(|_| Msg::Timer2Start),
                            />
                        <TimerWidget
                            storage=&self.timer3,
                            on_start=self.link.callback(|_| Msg::Timer3Start),
                            />
                    </section>
                    <section class="global-buttons">
                        <button class=("btn","global-button"), id="stop-btn", onclick=self.link.callback(|_| Msg::StopAll),>{ "Stop" }</button>
                        <button class=("btn","global-button"), id="clear-btn", onclick=self.link.callback(|_| Msg::Clear),>{ "Clear" }</button>
                    </section>
                    <footer id="footer">{"Source code: "}<a href="https://github.com/strawlab/ethotimer/">{"strawlab/ethotimer"}</a>{" | "}{format!("Compile date: {} (revision {})",
                                        env!("GIT_DATE"),
                                        env!("GIT_HASH"))}
                    </footer>
                </div>
            </div>
        }
    }
}

// fn empty() -> Html {
//     html! {
//         <></>
//     }
// }

// -----------------------------------------------------------------------------

#[wasm_bindgen(start)]
pub fn run_app() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Info));
    yew::start_app::<Model>();
}
