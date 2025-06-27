# https2http-rs
HTTPSの通信をなるべく安全にローカルに対してhttp通信に変換する奴です。
## 簡単な使い方
普通に起動するだけです。
## 設定
`config.json`ファイルを作成してください。
下記が設定例です。
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