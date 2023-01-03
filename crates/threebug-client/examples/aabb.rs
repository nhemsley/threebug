use core::time;
use std::{error::Error, thread};

use parry3d::na::Point3;
use rand::Rng;
use structopt::StructOpt;
use tracing::{error, info};

#[derive(Debug, StructOpt)]
struct Options {
    #[structopt(short, long, default_value = "10")]
    count: usize,
    #[structopt(short, long, default_value = "10")]
    volume_radius: f32,
    #[structopt(short, long, default_value = "2.0")]
    aabb_radius: f32,
    #[structopt(short, long, default_value = "0.5")]
    wait: f32,
    #[structopt(short, long)]
    random_radius: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    let opt = Options::from_args();

    let tb_client = threebug_client::client::default_client()
        .expect("Couldn't connect to server with default args");

    let mut client = tb_client.client;

    let mut rng = rand::thread_rng();

    for _ in 0..opt.count {
        let vrange = -opt.volume_radius..opt.volume_radius;
        let centre = Point3::origin().map(|_: f32| rng.gen_range(vrange.clone()));

        let radius = if opt.random_radius {
            opt.aabb_radius
        } else {
            rng.gen::<f32>() * opt.aabb_radius
        };
        let mins = centre.map(|c| c - radius);
        let maxs = centre.map(|c| c + radius);

        let aabb = parry3d::bounding_volume::Aabb::new(mins, maxs);

        let debug_entity_type =
            threebug_core::ipc::parry::ParryDebugEntityType::new_aabb_entity(aabb);

        match client.send_message(debug_entity_type) {
            Ok(_) => info!("Sent aabb"),
            Err(e) => error!("Couldnt send aabb to server: {:?}", e),
        }
        thread::sleep(time::Duration::from_secs_f32(opt.wait));
    }

    client.disconnect();

    Ok(())
}
