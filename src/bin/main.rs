use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;
use std::{fs, thread};
use test_web::ThreadPool;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let thread_pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        thread_pool.execute(|| handle_connection(stream));
    }
}

#[derive(Debug)]
enum Request {
    Get,
    Sleep,
    Stop,
}

impl Request {
    fn as_bytes(&self) -> &[u8] {
        match self {
            Request::Get =>  b"GET / HTTP/1.1\r\n",
            Request::Sleep => b"GET /sleep HTTP/1.1\r\n",
            Request::Stop => b"GET /stop HTTP/1.1\r\n"
        }
    }
    
    fn as_respond_str(&self) -> &str {
        match self {
            Request::Get | Request::Sleep=> "HTTP/1.1 200 OK",
            Request::Stop => "HTTP/1.1 404 NOT FOUND"
        }
    }
    
    fn from_bytes(bytes: [u8; 1024]) -> Request {
        if bytes.starts_with(Request::Get.as_bytes()) {
            Request::Get
        } else if bytes.starts_with(Request::Sleep.as_bytes()) {
            thread::sleep(Duration::from_secs(5));
            Request::Sleep
        } else {
            Request::Stop
        }
    }
}

#[derive(Debug)]
enum Response {
    Hello,
    FourFour,
}

impl Response {
    fn from_request(req: &Request) -> Response {
        match req {
            Request::Get => Response::Hello,
            Request::Sleep => Response::Hello,
            Request::Stop => Response::FourFour,
        }
    }
    
    fn get_resource(&self) -> &str {
        match self {
            Response::Hello => "hello.html",
            Response::FourFour => "404.html",
        }
    }
}


fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    stream.read(&mut buffer).unwrap();

    let req = Request::from_bytes(buffer);
    let rep = Response::from_request(&req);

    let contents = fs::read_to_string(rep.get_resource()).unwrap();

    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        req.as_respond_str(),
        contents.len(),
        contents
    );

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
