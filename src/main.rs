use clap::{arg, Parser};
use log::LevelFilter;
use reqwest::Url;
use simple_logger::SimpleLogger;
use std::time::Duration;
/// Adguard Home (AGH) Prometheus exporter.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Base AGH API URL. Note the trailing slash!
    #[arg(short = 'a', long, default_value_t = Url::parse("http://localhost:80/control/").unwrap())]
    agh_host: Url,

    /// Address to bind to and serve metrics from, including metrics URL
    #[arg(short = 's', long, default_value_t = Url::parse("http://0.0.0.0:9100/metrics").unwrap())]
    serve_addr: Url,

    /// AGH username.
    #[arg(short = 'u', long)]
    agh_username: Option<String>,

    /// AGH password.
    #[arg(short = 'p', long)]
    agh_password: Option<String>,

    /// Scrape interval, in seconds.
    #[arg(short = 'i', long, default_value_t = 5)]
    scrape_interval: u64,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    SimpleLogger::new().init().unwrap();
    log::set_max_level(LevelFilter::Info);

    let bind_host = format!(
        "{}:{}",
        args.serve_addr.host_str().unwrap(),
        args.serve_addr.port_or_known_default().unwrap()
    );
    let metrics_path = args.serve_addr.path();

    let sock_addr = match tokio::net::lookup_host(bind_host).await {
        Ok(mut s) => match s.next() {
            Some(s) => s,
            None => unreachable!(),
        },
        Err(e) => {
            panic!("Cannot bind to requested address: {}", e)
        }
    };

    let rx = agh_exporter_rs::scrape::start_scrape_loop(
        args.agh_host,
        args.agh_username,
        args.agh_password,
        Duration::from_secs(args.scrape_interval),
    )
    .unwrap();

    agh_exporter_rs::server::serve(sock_addr, rx, Some(metrics_path.into()))
        .await
        .unwrap();
}
