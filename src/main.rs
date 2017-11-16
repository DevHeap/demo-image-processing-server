#![feature(plugin)]
#![plugin(rocket_codegen)]
#![feature(custom_derive)]

#[macro_use]
extern crate log;
extern crate fern;
extern crate chrono;

extern crate reqwest;
extern crate ring;

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json as json;
extern crate uuid;

use rocket::response::content::Json;
use rocket::State;
use rocket::data::Data;

mod storage;
use storage::Storage;

use std::sync::Mutex;

struct SharedState {
    storage: Mutex<Storage>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ProcesssedImages {
    r: String,
    g: String,
    b: String,
}

#[get("/image/<name>")]
fn get_image(state: State<SharedState>, name: String) -> Option<Vec<u8>> {
    let storage = state.storage.lock().unwrap();
    let asset = storage.get(&name).to_vec();
    match asset {
        Ok(data) => Some(data),
        Err(e) => {
            error!("{}", e);
            None
        }
    }
}

#[post("/image", data = "<image>")]
fn process_image(state: State<SharedState>, image: Data) -> Option<Json<String>> {
    let storage = state.storage.lock().unwrap();
    let uuid = uuid::Uuid::new_v4().hyphenated().to_string();
    let name = format!("{}.jpg", uuid);
    
    {
        let mut asset = storage.get(&name);
        asset.create().unwrap();
    
        info!("Created {}", name);

        let mut asset_file = asset.file().unwrap();
        image.stream_to(&mut asset_file)
            .map(|n| info!("Wrote {:?} bytes to {:?}", n, asset.path()))
            .unwrap();
        asset_file.sync_all().unwrap();
    }

    info!("Invoking processor on {}", name);

    match invoke_processor(storage.path().to_str().unwrap(), &name) {
        Some((r, g, b)) => {
            let pi = ProcesssedImages {
                r, g, b
            };

            let response = json::to_string(&pi).unwrap();

            Some(Json(response))
        },
        None => {
            None
        }
    }
}

fn invoke_processor(dir: &str, file_name: &str) -> Option<(String, String, String)> {
    let status = std::process::Command::new("./process.sh")
        .arg(dir)
        .arg(file_name)
        .spawn().unwrap()
        .wait().unwrap();
    
    if status.code() != Some(0) {
        error!("Alena fucked up");
        None
    } else {
        Some((
            format!("r_{}", file_name),
            format!("g_{}", file_name),
            format!("b_{}", file_name),
        ))
    }
}


fn main() {
    fern::Dispatch::new()
        .chain(
            fern::Dispatch::new()
                .format(|out, message, record| {
                    out.finish(format_args!(
                        "{}[{}][{}] {}",
                        chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                        record.target(),
                        record.level(),
                        message
                    ))
                })
                .chain(std::io::stdout())
                .level(log::LogLevelFilter::Off)
                .level_for("demo_image_processing_server", log::LogLevelFilter::Trace)
        )
        .chain(
            fern::Dispatch::new()
                .format(|out, message, record| out.finish(format_args!("{}", message)))
                .chain(std::io::stdout())
                .level(log::LogLevelFilter::Off)
                .level_for("rocket", log::LogLevelFilter::Info)
        )
        .apply()
        .unwrap();

    debug!("logger initialized");

    rocket::ignite()
        .manage(SharedState {
            storage: Mutex::new(Storage::new("assets")),
        })
        .mount("/", routes![
            get_image,
            process_image
        ])
        .launch();
}

