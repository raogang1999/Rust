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