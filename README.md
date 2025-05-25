# ip_webhook

A simple Rust utility to monitor your public IP address and send updates to a configured webhook (e.g., Discord) when your IP changes.

## Features

- Checks your public IP address periodically.
- Sends notifications to a webhook when the IP changes.
- Configurable wait interval and webhook URL via `config.toml`.

## Usage

1. **Build the project:**
    ```sh
    cargo build
    ```

2. **First run:**
    - On first run, a `config.toml` file will be created if it does not exist.
    - Edit `config.toml` and set your webhook URL and desired wait interval (in seconds).

    Example `config.toml`:
    ```toml
    webhook = "https://your.webhook.url"
    wait_seconds = 60
    ```

3. **Run the program:**
    ```sh
    cargo run
    ```

## Configuration

- `webhook`: The URL to send IP change notifications to.
- `wait_seconds`: How often (in seconds) to check for IP changes.

## License
MIT