# https2http-rs
[日本語話者はこちら!!](/README.ja.md)  
A tool designed to securely convert HTTPS traffic to HTTP for local communication.  
Simply start the service normally.  
You can access the following URL to get a response:  
```
/proxy?url=https://example.com
```
If you add the request parameter `body=0`, the response will not include the body. If you add `header=0`, the response will not include the headers.  
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
## About Authentication
Authentication is used to prevent malicious users from using this proxy.  
### header
This is a simple authentication method.  
You can authenticate by assigning the password written in `header_auth` in `config.json` to the `header_auth` header.  