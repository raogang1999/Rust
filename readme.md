# Web框架
一个简单的Rust web框架的实现

## 1. TCP Server and Client

1. 创建工作空间

   ```
   cargo new rust_web && cd rust_web
   ```

2. 创建tcp server and client

   ```
   cargo new tcpserver
   cargo new tcpclient
   ```

3. 配置工作空间

   ```rust
   //Cargo.toml
   [workspace]
   
   members = ["tcpserver","tcpclient"]
   ```

### TCPServer

```rust
use std::net::TcpListener;
fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    println!("Running on port 8080");
    for stream in listener.incoming() {
        let _stream = stream.unwrap();
        println!("Connection established!");
    }
}

```

在工作空间使用

```
cargo run -p tcpserver
```



### Client

```rust
use std::io::{Read, Write};
use std::net::TcpStream;
use std::str;
fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:8080").unwrap();
    stream.write("Hello".as_bytes()).unwrap();
    let mut buffer = [0;5];
    stream.read(& mut buffer).unwrap();
    println!("server : {:?}",
             str::from_utf8(&buffer).unwrap()
    );
 
}

```

## 2. 构建HTTP Server

Rust没有内置Http

包含：

- Server:监听TCP字节流
- Router：决定调用哪一个Handler
- Handler：处理HTTP请求，构建HTTP响应
- HTTP Library：解释字节流，把它转化为HTTP请求，把HTTP请求转回字节流。



构建步骤：

1. 解析HTTP请求消息
2. 构建HTTP响应消息
3. 路由与Handler
4. 测试

四个数据结构

### 请求

- HttpRequest:struct

  ```rust
  #[derive(Debug)]
  pub struct HttpRequest{
      pub method:Method,
      pub version:Version,
      pub resource:Resource,
      pub headers:HashMap<String,String>,
      pub msg_body:String,
  }
  
  impl From<String> for HttpRequest {
      fn from(req: String) -> Self {
          let mut parsed_method = Method::Uninitialized;
          let mut parsed_version = Version::V1_1;
          let mut parsed_resource = Resource::Path("".to_string());
          let mut parsed_headers = HashMap::new();
          let mut parsed_msg_body = "".to_string();
          for line in req.lines(){
              if line.contains("HTTP"){
                  let (method,resource,version) = process_req_line(line);
                  parsed_method = method;
                  parsed_resource= resource;
                  parsed_version = version;
              }else if line.contains(":"){
                  let (key,value) = process_header_line(line);
                  parsed_headers.insert(key,value);
              }else if line.len()==0{
  
              }else {
                  parsed_msg_body = line.to_string();
              }
          }
          HttpRequest{
              method:parsed_method,
              version:parsed_version,
              resource:parsed_resource,
              headers:parsed_headers,
              msg_body:parsed_msg_body,
          }
      }
  }
  fn process_req_line(s:&str) ->(Method,Resource,Version){
      let mut words = s.split_whitespace();
      let method = words.next().unwrap();
      let resource = words.next().unwrap();
      let version = words.next().unwrap();
      (method.into(),Resource::Path(resource.to_string()),version.into())
  }
  fn process_header_line(s:&str) -> (String,String){
      let mut  header_items = s.split(':');
      let mut key = String::from("");
      let mut value = String::from("");
      if let Some(k) = header_items.next(){
          key = k.to_string();
      }
      if let Some(v) = header_items.next(){
          value = v.to_string();
      }
      (key,value)
  }
  ```



- Method:enum

  ```rust
  #[derive(Debug,PartialEq)]
  pub enum Method{
      Get,
      Post,
      Uninitialized,
  }
  
  impl From<&str> for Method {
      fn from(s: &str) -> Self {
          match s {
              "GET" => Method::Get,
              "Post"=> Method::Post,
              _ =>Method::Uninitialized,
          }
      }
  }
  ```

- Version:enum

  ```rust
  #[derive(Debug,PartialEq)]
  pub enum Version{
      V1_1,
      V2_0,
      Uninitialized,
  }
  
  impl From<&str> for Version {
      fn from(s: &str) -> Self {
          match s {
              "HTTP/1.1"=>Version::V1_1,
              "HTTP/2.0"=>Version::V2_0,
              _ =>Version::Uninitialized,
          }
      }
  }
  ```

- Resource

  ```rust
  #[derive(Debug,PartialEq)]
  pub enum Resource{
      Path(String),
  }
  ```



都需要实现三个Trait

- From<&str>,包字符串切片转为HttpRequset
- Debug，调试信息
- PartialEq，用于解析和自动化测试脚本作比较

创建两个新包

```
cargo new httpserver
cargo new --lib http
```

配置

```
[workspace]

members = ["tcpserver","tcpclient","http","httpserver"]
```

在lib.rs中编写以上结构体。



### 响应

结构体

```rust
use std::collections::HashMap;

#[derive(Debug,PartialEq,Clone)]
pub struct HttpResponse<'a>{
    version: &'a str,
    status_code:&'a str,
    status_text:&'a str,
    headers:HashMap<&'a str, &'a str>,
    body: Option<String>,
}
```

方法与Trait

- Default 默认值
- new 创建新的结构体
- send_response 构建响应
- getter 获取成员值
- From trait ,将HttpResponse 转为String

```rust
use std::io::Result;
use std::collections::HashMap;
use std::fmt::format;
use std::io::Write;

#[derive(Debug,PartialEq,Clone)]
pub struct HttpResponse<'a>{
    version: &'a str,
    status_code:&'a str,
    status_text:&'a str,
    headers:Option<HashMap<&'a str, &'a str>>,
    body: Option<String>,
}

impl <'a> Default for HttpResponse<'a> {
    fn default() -> Self {
        Self{
            version: "HTTP/1.1".into(),
            status_code:"200".into(),
            status_text:"OK".into(),
            headers:None,
            body:None,
        }
    }
}

impl <'a> From<HttpResponse<'a>> for String {
    fn from(res: HttpResponse) -> String {
        let res1 = res.clone();
        format!(
            "{} {} {}\r\n{}Content-Length: {}\r\n\r\n{}",
            &res1.version(),&res1.status_code(),&res1.status_text(),
            &res1.headers(),
            &res.body.unwrap().len(),
            &res1.body(),
        )
    }
}

impl <'a> HttpResponse<'a> {
    pub fn new(
        status_code:&'a str,
        headers:Option<HashMap<&'a str, &'a str>>,
        body: Option<String>,
    ) -> HttpResponse<'a>{

        let mut response:HttpResponse<'a> = HttpResponse::default();
        if status_code != "200"{
            response.status_code = status_code.into();
        }
        response.headers = match &headers {
            Some(_h) => headers,
            None =>{
                let mut h = HashMap::new();
                h.insert("Content-Type","text/html");
                Some(h)
            }
        };
        response.status_text = match response.status_code {
            "200" => "OK".into(),
            "400" => "Bad Request".into(),
            "404" => "Not Found".into(),
            "500" => "Internal Server Error".into(),
            _ => "Not Found".into()
        };
        response.body = body;
        response
    }
    pub fn send_response(&self,write_stream:&mut impl Write) ->Result<()>{
        let res = self.clone();
        let response_string = String::from(res);
        let _ = write!(write_stream,"{}",response_string);
        Ok(())
    }
    fn version(&self) ->&str{
        self.version
    }
    fn status_code(&self) ->&str{
        self.status_code
    }
    fn status_text(&self) ->&str{
        self.status_text
    }
    fn headers(&self) -> String{
        let map:HashMap<&str,&str> = self.headers.clone().unwrap();
        let mut header_string :String = "".into();
        for (k,v) in map.iter(){
            header_string = format!("{}{}:{}\r\n",header_string,k,v);
        }
        header_string
    }
    pub fn body(&self)->&str{
        match &self.body {
            Some(b)=>b.as_str(),
            None => "",
        }
    }
}
```

### server

```
[dependencies]
http = {path="../http"}
serde = {version = "1.0.136",features=["derive"]}
serde_json = {version ="1.0.78"}
```

```rust
use std::io::prelude::*;
use std::net::TcpListener;
use http::httprequest::HttpRequest;
use super::router::Router;
use std::str;

pub struct Server<'a>{
    socket_addr: &'a str,
}

impl <'a> Server<'a> {
    pub fn new(socket_addr:&'a str) ->Self{
        Server{socket_addr}
    }
    pub fn run(&self){
        let connection_listener = TcpListener::bind(self.socket_addr).unwrap();
        println!("Running on {}",self.socket_addr);
        for stream in connection_listener.incoming() {
            let mut stream = stream.unwrap();
            println!("Connection established");
            let mut read_buffer = [0;512];
            stream.read(&mut read_buffer).unwrap();
            let req:HttpRequest =  String::from_utf8(read_buffer.to_vec()).unwrap().into();
            Router::route(req,&mut stream);

        }
    }

}

```

### Router

```
[dependencies]
http = {path="../http"}
serde = {version = "1.0.136",features=["derive"]}
serde_json = {version ="1.0.78"}
```

```rust
use super::handler::{Handler,PageNotFoundHandler,StaticPageHandler,WebServiceHandler};
use http::{httprequest,httprequest::HttpRequest,httpresponse::HttpResponse};
use std::io::prelude::*;

pub struct Router;

impl Router {
    pub fn route(req:HttpRequest,stream:&mut impl Write)->(){
        match req.method {
            //只处理GET
            httprequest::Method::Get => match &req.resource {
                httprequest::Resource::Path(s) =>{
                    let router: Vec<&str> = s.split("/").collect();
                    match router[1] {
                        "api" =>{
                            let resp:HttpResponse = WebServiceHandler::handler(&req);
                            let _ = resp.send_response(stream);
                        }
                        _ =>{
                            let resp:HttpResponse = StaticPageHandler::handler(&req);
                            let _ = resp.send_response(stream);
                        }
                    }
                }
            },
            _ =>{
                let resp:HttpResponse = PageNotFoundHandler::handler(&req);
                let _ = resp.send_response(stream);
            }
        }
    }
}
```

### Handler

```rust
use http::{httprequest::HttpRequest,httpresponse::HttpResponse};
use serde::{Deserialize,Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use http::httprequest::Resource::Path;

pub trait Handler{
    fn handler(req:&HttpRequest)->HttpResponse;
    fn load_file(flie_name:&str) ->Option<String>{
        let default_path = format!("{}/public",env!("CARGO_MANIFEST_DIR"));
        let public_path = env::var("PUBLIC_PATH").unwrap_or(default_path);
        let full_path = format!("{}/{}",public_path,flie_name);
        let contents = fs::read_to_string(full_path);
        contents.ok()
    }
}
pub struct StaticPageHandler;
pub struct PageNotFoundHandler;
pub struct WebServiceHandler;
#[derive(Serialize,Deserialize)]
pub struct OrderStatus{
    order_id:i32,
    order_date:String,
    order_status:String,
}
impl Handler for PageNotFoundHandler{
    fn handler(req: &HttpRequest) -> HttpResponse {
        HttpResponse::new("404",None,Self::load_file("404.html"))
    }
}

impl Handler for StaticPageHandler {
    fn handler(req: &HttpRequest) -> HttpResponse {
        let http::httprequest::Resource::Path(s) = &req.resource;
        let route:Vec<&str> = s.split("/").collect();
        match route[1] {
            "" => HttpResponse::new("200",None,Self::load_file("index.html")),
            "health" => HttpResponse::new("200",None,Self::load_file("health.html")),
            path => match Self::load_file(path) {
                Some(contents)=>{
                    let mut map: HashMap<&str,&str> = HashMap::new();
                    if path.ends_with(".css"){
                        map.insert("Content-Type","text/css");
                    }else if path.ends_with(".js") {
                        map.insert("Content-Type","text/js");
                    }else {
                        map.insert("Content-Type", "text/html");
                    }
                    HttpResponse::new("200",Some(map),Some(contents))
                }
                None => HttpResponse::new("404",None,Self::load_file("404.html"))
            },
        }
    }
}

impl WebServiceHandler {
    fn load_json()->Vec<OrderStatus>{
        let default_path = format!("{}/data",env!("CARGO_MANIFEST_DIR"));
        let data_path = env::var("DATA_PATH").unwrap_or(default_path);
        let full_path = format!("{}/{}",data_path,"orders.json");
        let json_contents = fs::read_to_string(full_path);
        let orders:Vec<OrderStatus> = serde_json::from_str(json_contents.unwrap().as_str()).unwrap();
        orders
    }
}
impl Handler for WebServiceHandler{
    fn handler(req: &HttpRequest) -> HttpResponse {
        let http::httprequest::Resource::Path(s) = &req.resource;
        let route:Vec<&str> = s.split("/").collect();
        //localhost:8080/api/shipping/orders
        match route[2] {
            "shipping" if route.len() > 2 && route[3] =="orders" =>{
                let body = Some(serde_json::to_string(&Self::load_json()).unwrap());
                let mut headers :HashMap<&str,&str> = HashMap::new();
                headers.insert("Content-Type","application/json");
                HttpResponse::new("200",Some(headers),body)
            },
            _ => HttpResponse::new("404",None,Self::load_file("404.html"))
        }
    }
}
```



