echo "GET!!!!!!!!!!!!"
curl "http://localhost:8080/proxy?url=http://localhost:8080/__dev"
echo "HEADERS!!!!!!!!!!!!"
curl -H "X-Test-Header: Cat" `
    "http://localhost:8080/proxy?url=http://localhost:8080/__dev&header=1"
echo "BODY!!!!!!!!!!!!"
curl -d "key=value" `
    "http://localhost:8080/proxy?url=http://localhost:8080/__dev&header=1"
echo "AUTH!!!!!!!!!!!!"
curl -H "header_auth: password" `
    "http://localhost:8080/proxy?url=http://localhost:8080/__dev&header=1"