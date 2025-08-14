# https2http-rs
HTTPSの通信をなるべく安全にローカルに対してhttp通信に変換する奴です。
## 簡単な使い方
普通に起動するだけです。  
このURLにアクセスすればリクエストが返ってきます。  
```
/proxy?url=https://example.com
```
リクエストパラメーターに`body=0`を付与した場合はボディを付与せず、`header=0`を付与した場合はヘッダーを付与しません。  
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
## 認証について
認証は悪意のある者にこのproxyを使わせないようにするものです。  
### header
簡易的な認証方法です。  
`config.json`の`header_auth`にかかれたパスワードを`header_auth`というヘッダーに代入することで認証することができます。  