use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};  
use std::thread;
use colored::Colorize;
use std::sync::{Arc, Mutex};
use crate::mpsc::Sender;

//Im sorry for the spaghetti
//It started out pretty good


#[derive(Debug)]
pub struct Client{
    pub socket: TcpStream,
    pub ip: String,
    pub name: String, 
}

impl Client{
    pub fn connect_to_server(ip: &String, is_connected: &mut bool, should_end: Arc<Mutex<u8>>, old_sender: &mut Sender<String>){ 

        println!("You have started the connectiong...");

        let mut connect = match TcpStream::connect(ip) {
            Ok(_connect) => {
                _connect
            },
            Err(_) => {
                println!("Please enter a valid addr");
                return;
            }
        };

        connect.set_nonblocking(true).expect("Not non-blocking thing");

        let (sender, receiver) = mpsc::channel::<String>();
 

        thread::spawn(move || loop {


            //There is probably a faster and more reliable way of doing this
            //But it works
            let ending = should_end.lock().unwrap();

            if  *ending != 0 {
                println!("Disconnecting from server!");
                return;
            }

            let mut buffer = vec![0;64];

            match connect.read_exact(&mut buffer){
                Ok(_) =>{
                    let msg = buffer.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                    let msg = String::from_utf8(msg).expect("");

                    
                    //Do code here for making prints look nice
                    let msgs:Vec<String> = msg.split(":::").map(str::to_string).collect();

                    if msgs.len() > 1 {

                    let n_msg: &str = &*msgs[1];

                    match msgs[0].as_str(){
                        "black" => {
                            println!("{}", format!("{}", n_msg).on_black());
                        },
                        "red" => {
                            println!("{}", format!("{}",n_msg).red());
                        },
                        "green" => {
                            println!("{}", format!("{}",n_msg).green());
                        },
                        "yellow" => {
                            println!("{}", format!("{}",n_msg).yellow());
                        },
                        "blue" => {
                            println!("{}", format!("{}",n_msg).blue());
                        },
                        "magenta" => {
                            println!("{}", format!("{}",n_msg).bright_magenta());
                        },
                        "cyan" => {
                            println!("{}", format!("{}",n_msg).cyan());
                        },
                        "white" => {
                            println!("{}", format!("{}",n_msg).white());
                        },
                        _ => (),
                    }
                    }if msg.eq("") {
                        
                    }
                    else{
                        let n_msg: &str = &*msgs[0];
                        println!("{}", format!("{}",n_msg).white());
                    }
                },

                Err(ref err ) if err.kind() == ErrorKind::WouldBlock => (),
                Err(_) => {
                    println!("big error occured");
                    return;
                }
            }

            match receiver.try_recv() {
                // received message from channel
                Ok(msg) => {
                    let mut buffer = msg.clone().into_bytes();
                    buffer.resize(64, 0);
    
                    if connect.write_all(&buffer).is_err() {
                        println!("Failed to send message!")
                    }
                }, 
                // Varför inte göra något här?
                // Jo för den tror den blir disconnected här :((
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => ()
            }
        });
        println!("Connected to server");
        *is_connected = true;
        *old_sender = sender;
    }

}

#[derive(Debug)]
pub struct Server{
    pub ip: String,
    pub listener: TcpListener,
    pub clients: Vec<Client>,
}

impl Server{
    pub fn init_server(addr: String) -> Server{

        let tc_server = TcpListener::bind(&addr).expect("Listener failed");
        tc_server.set_nonblocking(true).expect("non blocking failed");
        let vec_clients:Vec<Client> = vec![];

        Server{
            ip: addr,
            listener: tc_server,
            clients: vec_clients,
        }

    }

    pub fn run_server(self){
        let (sender, listen) = mpsc::channel::<String>();

        let mut clients:Vec<Client> = vec![];

        thread::spawn(move || loop{
    
            if let Ok((mut socket, addr)) = self.listener.accept(){
                println!("Client connected!!!");

                let send = sender.clone();

                clients.push(Client{
                    socket: socket.try_clone().expect("Could not add client"), 
                    ip: addr.to_string(),
                    name: "New user".to_string(),
                });
            
                thread::spawn(move || loop{
                let mut buffer = vec![0; 64];

                match socket.read_exact(&mut buffer) {
                    Ok(_) =>{
                        let msg = buffer.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                        let mut msg = String::from_utf8(msg).expect("Invalid message");
 
                    //Here goes code for doing stuff with the text
                
                        if msg.chars().nth(0).unwrap() == '/' {
                            msg.remove(0);
                            let msg:Vec<String> = msg.split_whitespace().map(str::to_string).collect();
                            match msg[0].as_str() {
                                "change" => {
                                    let send_msg = "change#¤#".to_string() + &msg[1];
                                    send.send(send_msg).expect("Failed to send");
                                },
                                "whisper" => {
                                    let n_msg = &msg[2..msg.len()];
                                    let rest_msg = n_msg.join(" ");
                                    let mut send_msg = "whisper#¤#".to_owned() + &msg[1];
                                    send_msg.push_str("#¤#");
                                    send_msg.push_str(&rest_msg);
                                    send.send(send_msg).expect("Failed to send to send");
                                },
                                "current" => {
                                    let mut send_msg = (&*msg[0]).to_string();
                                    send_msg.push_str("#¤#");
                                    send_msg.push_str(&addr.to_string());
                                    send.send(send_msg).expect("Failed to send to send");
                                },
                                _ => (),
                            }

                        }else{
                            let send_msg = addr.to_string() + "#¤#" + &msg;
                            send.send(send_msg).expect("Failed to send to send")
                        }
                        },
                    Err(_) => (),    
                }});
            }

            if let Ok(msg) = listen.try_recv() {
                let mut none = "".to_string().clone().into_bytes();
                none.resize(64,0);
                
                let msgs:Vec<String> = msg.split("#¤#").map(str::to_string).collect();

                clients = clients.into_iter().filter_map(|mut client| {

                    if msgs[0].as_str() == "change" {
                        client.name = (&msgs[1]).to_string();

                        client.socket.write_all(&none).map(|_| client).ok()
                    }else if msgs[0].as_str() == "current" && msgs[1].eq(&client.ip) {
                        let send_msg = "".to_string();
                        let mut buffer = send_msg.clone().into_bytes();
                        buffer.resize(64,0);
                        client.socket.write_all(&buffer).map(|_| client).ok()
                    } else if msgs[0].as_str() == "whisper" && msgs[1].eq(&client.name) {
                        let mut buffer = msgs[2].clone().into_bytes();
                        buffer.resize(64, 0);
                        client.socket.write_all(&buffer).map(|_| client).ok()
                    } else if !&msgs[0].eq(&client.ip) {
                        let mut buffer = msgs[1].clone().into_bytes();
                        buffer.resize(64, 0);
                        client.socket.write_all(&buffer).map(|_| client).ok()
                    } else{
                        client.socket.write_all(&none).map(|_| client).ok()
                    }
                }).collect::<Vec<_>>();
            }

        });
    }


}


fn main() {

    let mut is_connected = false;

    let end = Arc::new(Mutex::new(0));

    //Ja skulle nog vara smartare att skicka _recive här till Client::connect_to_server, istället för att ta emot en sender från den funktionen men men
    let (mut sender, _recive) = mpsc::channel::<String>();

    let mut name: String = "New user".to_string();

    let mut color: String = "white".to_string();

    loop {
        let mut msg_buffer = String::new();
        io::stdin().read_line(&mut msg_buffer).expect("Failed to read user message!");

        let msg = msg_buffer.trim().to_string();

        if msg.len() >= 1 && msg.chars().nth(0).unwrap() == '/'{
            let mut msg:Vec<String> = msg.split_whitespace().map(str::to_string).collect();
            msg[0].remove(0);
            match msg[0].as_str() {
                "join" => {
                    if msg.len() > 2 && is_connected == false{
                        Client::connect_to_server(&msg[1], &mut is_connected, end.clone(), &mut sender);
                        name = (*msg[2]).to_string();
                        let send_msg = "/change ".to_string() + &name;
                        sender.send(send_msg).expect("Failed");
                    }else{
                        println!("{}", format!("Please enter a valid addr and name...").magenta());
                    }
                },
                "create" => {
                    if msg.len() > 1 {
                        let addr = (&*msg[1]).to_string();
                        let server: Server = Server::init_server(addr);
                        println!("{:?}", &server);
                        server.run_server();
                    }else{
                        println!("{}", format!("Please enter a valid addr...").magenta());
                    }
                },
                "leave" => {
                    let mut s_end = end.lock().unwrap();
                    *s_end = *s_end + 1;
                    println!("{}", format!("Good bye!").red()); 
                    break;
                },
                "set_name" => {
                    if msg.len() > 1 {
                        name = (*msg[1]).to_string();
                        let send_msg = "/change".to_string() + &name;
                        sender.send(send_msg).expect("Failed");
                    }
                },
                "set_color" => {
                    if msg.len() > 1 {
                        color = (*msg[1]).to_string();
                    }
                },
                "whisper" => {
                    if msg.len() > 2 {
                        let mut send_msg = "/whisper ".to_string();
                        send_msg.push_str(&msg[1]);
                        send_msg.push_str(" ");
                        send_msg.push_str("whisper from ");
                        send_msg.push_str(&name);
                        send_msg.push_str(": ");
                        let n_msg = &msg[2..msg.len()];
                        let rest_msg = n_msg.join(" ");
                        send_msg.push_str(&rest_msg);
                        sender.send(send_msg).expect("Failed to send");
                    }
                },
                "get_users" => {
                    sender.send("/current".to_string()).expect("Failed to send");
                },
                "help" => {
                    println!("The commands you can use are: ");
                    println!("/help                         ---Prints all available commands");
                    println!("/join -addr -name             ---Joins a chatroom");
                    println!("/create -addr                 ---Creates a chatroom");
                    println!("/leave                        ---Leaves a chatroom");
                    println!("/whisper -name -message       ---Send a DM to the given name");
                    println!("/set_name -name               ---Sets name to given name");
                    println!("/set_color -color             ---Sets textcolor to given color");
                    println!("Available colors are:");
                    println!("{}", format!("Red     : red").red());
                    println!("{}", format!("Black   : black").bright_black());
                    println!("{}", format!("Green   : green").green());
                    println!("{}", format!("Cyan    : cyan").cyan());
                    println!("{}", format!("Magenta : magenta").bright_magenta());
                    println!("{}", format!("White   : white").white());
                    println!("{}", format!("Blue    : blie").blue());
                    println!("{}", format!("Yellow  : yellow").yellow());
                }
                _ => println!("Please enter a valid command..."),
            }
        }else{
            let mut send_msg = (&*color).to_string();
            send_msg.push_str(":::");
            send_msg.push_str(&*name);
            send_msg.push_str(": ");
            send_msg.push_str(&msg);
            sender.send(send_msg.to_string()).expect("Failed to send");
        }

    }
}


