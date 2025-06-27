# https2http-rs
[日本語話者はこちら!!](/README.ja.md)
A tool designed to securely convert HTTPS traffic to HTTP for local communication.
## Basic Usage
Simply start the service normally.
## Configuration
Create a `config.json` file.
Below is an example configuration:
```json
{
    "bind_address": "127.0.0.1",
    "bind_port": 8080,
    "log_level": "info",
    "auth": {
        "header_auth": "password"
    }
}
```
