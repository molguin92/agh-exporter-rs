# agh-exporter-rs

A small HTTP server exporting AdGuard Home metrics for consumption in Prometheus.

Built on top of [`tokio-rs`](https://tokio.rs/), [`serde-rs`](https://serde.rs/), [`reqwest`](https://docs.rs/reqwest/latest/reqwest/), and [`axum`](https://docs.rs/axum/latest/axum/).

## Installing and running the crate

### From `crates.io`

Install using `cargo`:

```bash
$ cargo install agh-exporter-rs

$ agh-exporter-rs -h           
A small HTTP server exporting AdGuard Home metrics for consumption in Prometheus.

Usage: agh-exporter-rs [OPTIONS]

Options:
  -a, --agh-host <AGH_HOST>
          Base AGH API URL. Note the trailing slash! [env: AGH_HOST=] [default: http://localhost/control/]
  -s, --serve-addr <SERVE_ADDR>
          Address to bind to and serve metrics from, including metrics URL [env: AGH_SERVE_ADDR=] [default: http://0.0.0.0:9100/metrics]
  -u, --agh-username <AGH_USERNAME>
          AGH username [env: AGH_USERNAME=]
  -p, --agh-password <AGH_PASSWORD>
          AGH password [env: AGH_PASSWORD=]
  -i, --scrape-interval <SCRAPE_INTERVAL>
          Scrape interval, in seconds. Should ideally be less than half the Prometheus server scrape interval [env: AGH_SCRAPE_INTERVAL=] [default: 5]
  -h, --help
          Print help
  -V, --version
          Print version

```

### Using Docker

Pull the image from the GitHub Container Registry:

```bash
$ docker pull ghcr.io/molguin92/agh-exporter-rs:latest
```

You can also include the image in a `docker-compose.yml` like so:

```yaml
version: '3'
services:
  agh-exporter:
    image: ghcr.io/molguin92/agh-exporter-rs:latest
    environment:
      AGH_HOST: http://localhost/adguard/control/
      AGH_SERVE_ADDR: http://localhost:9100/metrics
      AGH_USERNAME: foo
      AGH_PASSWORD: bar
      AGH_SCRAPE_INTERVAL: 5
    ports:
      - "9100:9100"
    restart: on-failure
```

Make sure to update the environment variables to match your AdGuard Home deployment.

## Building locally

Clone the repository and build with `cargo`:

```bash
$ git clone git@github.com:molguin92/agh-exporter-rs.git

$ cd agh-exporter-rs

$ cargo build
```

## License

Licensed under an Apache 2.0 License.
See [LICENSE](LICENSE.md) for details.