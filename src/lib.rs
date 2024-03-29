#![recursion_limit = "1024"]

use wasm_bindgen::prelude::*;

use yew::html::Scope;
use yew::prelude::*;

use serde::{Deserialize, Serialize};

mod components;

use components::timer_widget::{self, TimerStorage, TimerWidget};

const VIEW_DATA_HASH: &str = "#view-data";
const FILENAME_TEMPLATE: &str = "ethotimer_%Y%m%d_%H%M%S.%f.csv";

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

struct TimedButtonPress {
    activity: u8,
    when: chrono::DateTime<chrono::Local>,
    is_active: bool,
}

struct Model {
    timer_master: TimerStorage,
    master_link: Option<Scope<TimerWidget>>,
    timer1: TimerStorage,
    timer2: TimerStorage,
    timer3: TimerStorage,
    history: Vec<TimedButtonPress>,
}

#[derive(Clone)]
pub enum Msg {
    Timer1Start,
    Timer2Start,
    Timer3Start,
    SetChildLink(Scope<TimerWidget>),
    StopAll,
    ClearData,
    ViewData,
    ViewTimers,
    DownloadCsv,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            timer_master: TimerStorage::default(),
            master_link: None,
            timer1: TimerStorage::default(),
            timer2: TimerStorage::default(),
            timer3: TimerStorage::default(),
            history: vec![],
        }
    }
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Timer1Start => {
                self.push_history(1);
                self.ensure_master_started();
                self.timer2.stop();
                self.timer3.stop();
            }
            Msg::Timer2Start => {
                self.push_history(2);
                self.ensure_master_started();
                self.timer1.stop();
                self.timer3.stop();
            }
            Msg::Timer3Start => {
                self.push_history(3);
                self.ensure_master_started();
                self.timer1.stop();
                self.timer2.stop();
            }
            Msg::StopAll => {
                self.stop_all();
            }
            Msg::ClearData => {
                self.timer_master.clear();
                self.timer1.clear();
                self.timer2.clear();
                self.timer3.clear();
                self.history = vec![];
            }
            Msg::ViewData => {
                self.stop_all();
                let location = web_sys::window().unwrap().location();
                let new_location = format!("{}{}", location.pathname().unwrap(), VIEW_DATA_HASH);

                web_sys::window()
                    .unwrap()
                    .history()
                    .unwrap()
                    .replace_state_with_url(&"".into(), "", Some(&new_location))
                    .unwrap();
            }
            Msg::ViewTimers => {
                let location = web_sys::window().unwrap().location();
                let new_location = location.pathname().unwrap();
                web_sys::window()
                    .unwrap()
                    .history()
                    .unwrap()
                    .replace_state_with_url(&"".into(), "", Some(&new_location))
                    .unwrap();
            }
            Msg::DownloadCsv => {
                let stamp = chrono::Local::now();
                let local: chrono::DateTime<chrono::Local> = stamp.with_timezone(&chrono::Local);
                let filename = local.format(FILENAME_TEMPLATE).to_string();
                let data_csv = self.get_data_csv();
                download_file(data_csv.as_bytes(), &filename);
            }
            Msg::SetChildLink(link) => {
                self.master_link = Some(link);
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // let document = web_sys::window().unwrap().document().unwrap();
        // // Should we use location() instead of url()?
        // let url = web_sys::Url::new(&document.url().unwrap()).unwrap();
        // let hash = url.hash();
        let hash = web_sys::window().unwrap().location().hash().unwrap();
        let inner = if hash == VIEW_DATA_HASH {
            self.view_data(ctx)
        } else {
            self.view_timers(ctx)
        };

        html! {
            <div id="page-container">
                <div id="content-wrap">
                    <h1>{"⏱ ethotimer"}</h1>
                    <p class="small-text">{"Timers for collecting data to make ethograms and related."}</p>
                    {inner}
                    <footer id="footer" class="small-text">{"Source code: "}<a href="https://github.com/strawlab/ethotimer/">{"strawlab/ethotimer"}</a>{" | "}{format!("Compile date: {} (revision {})",
                                        env!("GIT_DATE"),
                                        env!("GIT_HASH"))}
                    </footer>
                </div>
            </div>
        }
    }
}

impl Model {
    fn ensure_master_started(&mut self) {
        if let Some(link) = &self.master_link {
            link.send_message(timer_widget::Msg::OnStart);
        }
    }

    fn push_history(&mut self, activity: u8) {
        let when = chrono::Local::now();
        {
            // Currently, activities are mutually exclusive,
            // so starting one should stop the others.
            if activity != 1 && self.timer1.is_active() {
                self.history.push(TimedButtonPress {
                    activity: 1,
                    when,
                    is_active: false,
                });
            }
            if activity != 2 && self.timer2.is_active() {
                self.history.push(TimedButtonPress {
                    activity: 2,
                    when,
                    is_active: false,
                });
            }
            if activity != 3 && self.timer3.is_active() {
                self.history.push(TimedButtonPress {
                    activity: 3,
                    when,
                    is_active: false,
                });
            }
        }
        self.history.push(TimedButtonPress {
            activity,
            when,
            is_active: true,
        });
    }
    fn stop_all(&mut self) {
        let n_history = self.history.len();
        if n_history > 0 && self.history[n_history - 1].activity != 0 {
            self.push_history(0);
        }
        self.timer1.stop();
        self.timer2.stop();
        self.timer3.stop();
    }

    fn get_data_csv(&self) -> String {
        let mut lines = vec!["duration_from_start_seconds,activity_id,is_active".to_string()];
        if !self.history.is_empty() {
            let s0 = self.history[0].when;
            for row in self.history.iter() {
                let dur_msec = row.when.signed_duration_since(s0).num_milliseconds();
                lines.push(format!(
                    "{},{},{}",
                    dur_msec as f64 / 1000.0,
                    row.activity,
                    row.is_active as u8
                ));
            }
        }
        lines.join("\n")
    }

    fn view_data(&self, ctx: &Context<Self>) -> Html {
        let data_csv = self.get_data_csv();
        html! {
            <>
                <button class={classes!("btn","global-button")} id="view-btn" onclick={ctx.link().callback(|_| Msg::ViewTimers)}>{ "← Return to timers" }</button>
                <div class="csv-view">
                    <pre>{data_csv}</pre>
                </div>
                <button class="btn" id="download-csv-btn" onclick={ctx.link().callback(|_| Msg::DownloadCsv)}>{ "Download .csv" }</button>
            </>
        }
    }
    fn view_timers(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
                <section class="timers">
                    <TimerWidget
                        storage={&self.timer1}
                        text="Activity 1: "
                        on_start={ctx.link().callback(|_| Msg::Timer1Start)}
                        />
                    <TimerWidget
                        storage={&self.timer2}
                        text="Activity 2: "
                        on_start={ctx.link().callback(|_| Msg::Timer2Start)}
                        />
                    <TimerWidget
                        storage={&self.timer3}
                        text="Activity 3: "
                        on_start={ctx.link().callback(|_| Msg::Timer3Start)}
                        />
                </section>
                <section class="global-buttons">
                    <TimerWidget
                        storage={&self.timer_master}
                        text="Duration since start: "
                        show_start_button=false
                        on_create={Some(ctx.link().callback(Msg::SetChildLink))}
                        />
                </section>
                <section class="global-buttons">
                    <button class={classes!("btn","global-button")} id="stop-btn" onclick={ctx.link().callback(|_| Msg::StopAll)}>{ "Stop" }</button>
                    <button class={classes!("btn","global-button")} id="clear-btn" onclick={ctx.link().callback(|_| Msg::ClearData)}>{ "Clear Data" }</button>
                    <button class={classes!("btn","global-button")} id="view-btn" onclick={ctx.link().callback(|_| Msg::ViewData)}>{ "Stop and View Data" }</button>
                </section>
            </>
        }
    }
}

// -----------------------------------------------------------------------------

fn download_file(orig_buf: &[u8], filename: &str) {
    let mime_type = "application/octet-stream";
    let b = js_sys::Uint8Array::new(&unsafe { js_sys::Uint8Array::view(orig_buf) }.into());
    let array = js_sys::Array::new();
    array.push(&b.buffer());

    let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(
        &array,
        web_sys::BlobPropertyBag::new().type_(mime_type),
    )
    .unwrap();
    let data_url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();
    let document = web_sys::window().unwrap().document().unwrap();
    let anchor = document
        .create_element("a")
        .unwrap()
        .dyn_into::<web_sys::HtmlAnchorElement>()
        .unwrap();

    anchor.set_href(&data_url);
    anchor.set_download(filename);
    anchor.set_target("_blank");

    anchor.style().set_property("display", "none").unwrap();
    let body = document.body().unwrap();
    body.append_child(&anchor).unwrap();

    anchor.click();

    body.remove_child(&anchor).unwrap();
    web_sys::Url::revoke_object_url(&data_url).unwrap();
}

// -----------------------------------------------------------------------------

#[wasm_bindgen(start)]
pub fn run_app() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Info));
    yew::start_app::<Model>();
}
