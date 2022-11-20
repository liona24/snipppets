use std::env;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;

#[derive(Debug)]
struct ApplicationOptions {
    is_server: bool,
    port: usize,
    ip_addr: String,
}

#[derive(Debug)]
struct TcpClient {
    id: usize,
    stream: TcpStream,
}

fn spawn_server_monitor(exit_requested: &Arc<Mutex<bool>>) -> thread::JoinHandle<()> {
    let exit_requested = Arc::clone(exit_requested);
    thread::spawn(move || loop {
        let mut buf = String::new();
        if io::stdin().read_line(&mut buf).unwrap_or_default() > 0 {
            if buf.trim() == "quit" {
                let mut exit_requested = exit_requested.lock().unwrap();
                *exit_requested = true;
                break;
            } else {
                println!("Type 'quit' to shutdown the server!");
            }
        }
    })
}

fn run_server(ip_addr: &str, port: usize) -> io::Result<()> {
    let listener = TcpListener::bind(format!("{0}:{1}", ip_addr, port))?;
    listener
        .set_nonblocking(true)
        .expect("Cannot set non-blocking!");
    let clients = Arc::new(Mutex::new(Vec::<TcpClient>::new()));
    let mut client_handles = vec![];

    let exit_requested = Arc::new(Mutex::new(false));
    let server_monitor = spawn_server_monitor(&exit_requested);

    loop {
        match listener.accept() {
            Ok((socket, addr)) => {
                println!("Client [{:?}] connected.", addr);

                let id = client_handles.len();
                let clients = clients.clone();
                let exit_requested = exit_requested.clone();

                socket
                    .set_nonblocking(true)
                    .expect("non_blocking socket failed!");

                let handle = thread::spawn(move || {
                    let client = TcpClient { id, stream: socket };
                    {
                        let mut clients_locked = clients.lock().unwrap();
                        (*clients_locked).push(client);
                    }
                    handle_client(id, clients, exit_requested);
                });
                client_handles.push(handle);
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
            Err(e) => println!("Could not get client: {:?}", e),
        }

        {
            if *exit_requested.lock().unwrap() {
                server_monitor.join().unwrap();
                break;
            }
        }

        let sleep_dur = time::Duration::from_millis(200);
        thread::sleep(sleep_dur);
    }

    for handle in client_handles {
        handle.join().unwrap();
    }

    Ok(())
}

fn run_client(remote_ip_addr: &str, remote_port: usize) -> io::Result<()> {
    let stream = TcpStream::connect(format!("{0}:{1}", remote_ip_addr, remote_port))?;
    stream
        .set_read_timeout(Some(time::Duration::from_millis(50)))
        .expect("Could not set read timeout!");
    let stream = Arc::new(Mutex::new(stream));

    {
        let stream = stream.clone();
        let _handle = thread::spawn(move || loop {
            {
                let mut stream = stream.lock().unwrap();
                let mut buf = [0u8; 100];
                if stream.read(&mut buf).unwrap_or(0) > 0 {
                    println!(
                        "> {}",
                        String::from_utf8(buf.to_vec())
                            .unwrap_or("[ Err ] Could not decode message!".to_string())
                    );
                }
            }

            let sleep_dur = time::Duration::from_millis(200);
            thread::sleep(sleep_dur);
        });
    }

    loop {
        let mut buf = String::new();
        io::stdin().read_line(&mut buf)?;
        buf.pop(); // pop new line character

        if buf.len() > 100 {
            buf.truncate(100);
        } else if buf.len() < 100 {
            let fill_values = String::from_utf8(vec![0_u8; 100 - buf.len()]);
            buf.extend(fill_values);
        }

        let mut stream = stream.lock().unwrap();
        if stream.write(&buf.as_bytes()).unwrap_or(0usize) == 0 {
            break;
        }
    }

    Ok(())
}

fn handle_client(id: usize, clients: Arc<Mutex<Vec<TcpClient>>>, exit_requested: Arc<Mutex<bool>>) {
    loop {
        {
            if *exit_requested.lock().unwrap() {
                break;
            }
        }

        {
            let mut clients = clients.lock().unwrap();
            if let Some(client) = clients.iter_mut().find(|c| c.id == id) {
                let mut buf = [0u8; 100];

                match client.stream.read(&mut buf) {
                    Ok(n) => {
                        if n < buf.len() {
                            println!("Recieved message of invalid size! [{:?}]", client);
                        } else {
                            println!(
                                "Received message: [{}]",
                                String::from_utf8(buf.to_vec()).unwrap_or_default()
                            );
                            for c in clients.iter_mut() {
                                if c.id != id {
                                    c.stream.write(&buf).unwrap_or_else(|_| {
                                        println!(
                                            "Could not forward message to client [{:?}]",
                                            c.stream
                                        );
                                        0usize
                                    });
                                }
                            }
                        }
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                    Err(e) => panic!("Encountered io error: {}", e),
                }
            } else {
                break;
            }
        }

        let sleep_dur = time::Duration::from_millis(200);
        thread::sleep(sleep_dur);
    }
}

fn print_usage() {
    println!("Usage:");
    println!("I  - Start a server instance:");
    println!("./chat is_server=true port=9000 ip_addr=0.0.0.0");
    println!("II - Start as many clients as you want:");
    println!("./chat is_server=false port=9000 ip_addr=<ip of server>");
    println!("Messages from clients are broadcasted to all other clients");
}

fn parse_args(args: Vec<String>) -> ApplicationOptions {
    let mut is_server = false;
    let mut port = 9000;
    let mut ip_addr = String::from("0.0.0.0");
    let mut any_parse_errors = false;

    for arg in args.iter().skip(1) {
        let pair: Vec<&str> = arg.split("=").collect();
        if pair.len() != 2 {
            println!("Error in argument '{}'!", &arg);
            any_parse_errors = true;
        } else {
            match pair[0] {
                "is_server" => {
                    is_server = pair[1].parse::<bool>().unwrap_or_else(|err| {
                        println!("{}", err);
                        any_parse_errors = true;
                        is_server
                    })
                }
                "port" => {
                    port = pair[1].parse::<usize>().unwrap_or_else(|err| {
                        println!("{}", err);
                        any_parse_errors = true;
                        port
                    })
                }
                "ip_addr" => ip_addr = pair[1].to_string(),
                &_ => println!("Unknown option '{}'!", &arg),
            }
        }
    }

    if any_parse_errors {
        print_usage();
        process::exit(1);
    }

    ApplicationOptions {
        is_server,
        port,
        ip_addr,
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let options = parse_args(args);
    println!("{:?}", options);

    if options.is_server {
        run_server(&options.ip_addr, options.port).expect("Error in server!");
    } else {
        run_client(&options.ip_addr, options.port).expect("Error in client!");
    }

    println!("Program exited gracefully!");
}
