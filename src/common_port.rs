use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream}, sync::{Arc, Mutex},
};

use goxoy_address_parser::address_parser::AddressParser;
//use secp256k1::{PublicKey, SecretKey};

#[derive(Debug)]
pub struct CommonPort {
    addr: String,
    list:Arc<Mutex<Vec<String>>>,
    shutdown:Arc<Mutex<bool>>,
    shutdown_is_done:Arc<Mutex<bool>>,
}
impl CommonPort {
    pub fn new(addr:String) -> Self {
        CommonPort {
            addr,
            list:Arc::new(Mutex::new(Vec::new())),
            shutdown:Arc::new(Mutex::new(false)),
            shutdown_is_done:Arc::new(Mutex::new(false)),
        }
    }
    pub fn get_msg(&mut self) -> String{
        let mut result=String::new();
        if self.list.lock().unwrap().len()>0{
            result = self.list.lock().unwrap()[0].clone();
            self.list.lock().unwrap().remove(0);
        }
        result
    }
    pub fn close(&mut self){
        *self.shutdown.lock().unwrap()=true;
        loop{
            if self.shutdown_is_done.lock().unwrap().clone()==true {
                std::thread::sleep(std::time::Duration::from_millis(100));
                break;
            }
        }
    }
    pub fn start(&mut self) -> bool {
        let common_server_started = self.start_common_server_socket();
        if common_server_started == false {
            return false;
        }
        return true;
    }
    fn start_common_server_socket(&mut self) -> bool {
        let common_addr = self.addr.clone().to_address_parser_object();
        let local_addr_url = AddressParser::local_addr_for_binding(common_addr.clone());
        let listener = TcpListener::bind(&local_addr_url);
        if listener.is_err() {
            println!("listener error");
            return false;
        }
        let list_cloned = self.list.clone();
        let server = listener.unwrap();
        let shutdown_is_done_cloned=self.shutdown_is_done.clone();
        let shutdown_cloned=self.shutdown.clone();
        std::thread::spawn(move || 'func_thread_loop: loop {
            if server.set_nonblocking(true).is_err() {
                break 'func_thread_loop;
            }
            'main_thread_loop: loop {
                if let Ok((mut socket, _addr)) = server.accept() {
                    'inner_thread_loop: loop {
                        let mut buff = vec![0; 1024 * 1024];
                        match socket.read(&mut buff) {
                            Ok(_income_data_length) => {
                                if socket.write_all("ok".as_bytes()).is_err() {
                                    //println!("{}  >  {}",host_addr_cloned.clone(),"Message Sending Error [code:6587]".to_string(),);
                                }
                                let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                                let response = String::from_utf8_lossy(&msg).to_string();                
                                list_cloned.lock().unwrap().push(response.clone());
                                println!("{}  >  Message Received > ", response.clone());
                                break 'inner_thread_loop;
                            }
                            Err(_) => {}
                        }
                        if shutdown_cloned.lock().unwrap().clone()==true{
                            break 'inner_thread_loop;
                        }
                    }
                }
                if shutdown_cloned.lock().unwrap().clone()==true{
                    break 'main_thread_loop;
                }
            }

            if shutdown_cloned.lock().unwrap().clone()==true{
                *shutdown_is_done_cloned.lock().unwrap()=true;
                break 'func_thread_loop;
            }
        });
        true
    }
}

pub trait AddressParserConverter {
    fn to_address_parser_object(&self) -> AddressParser;
}
impl AddressParserConverter for str {
    fn to_address_parser_object(&self) -> AddressParser {
        AddressParser::string_to_object(self.clone().to_string())
    }
}

pub trait AddressParserConverterForString {
    fn to_address_parser_object(&self) -> AddressParser;
}
impl AddressParserConverterForString for String {
    fn to_address_parser_object(&self) -> AddressParser {
        AddressParser::string_to_object(self.clone())
    }
}

pub trait AddressParserToConnection {
    fn one_time_connection(&self,msg_data: String) -> usize;
    fn to_string(&self) -> String;
}
impl AddressParserToConnection for AddressParser {
    fn one_time_connection(&self,msg_data: String,) -> usize {
        let client_obj = TcpStream::connect(AddressParser::local_addr_for_binding(self.clone()));
        if client_obj.is_err() {
            return 77;
        }
        let mut client = client_obj.unwrap();
        let blocking_result = client.set_nonblocking(true);
        if blocking_result.is_err() {
            return 66;
        }

        let writed_result = client.write_all(&msg_data.as_bytes());
        if writed_result.is_err() {
            return 88;
        }
        let mut buff = vec![0; 1024 * 32];
        loop {
            let income = client.read(&mut buff);
            if income.is_ok() {
                let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                let response = String::from_utf8_lossy(&msg).to_string();
                if response.eq("ok"){
                    return 0;
                }else{
                    return 33;
                }
            }
        }
        return 44;
    }

    fn to_string(&self) -> String {
        AddressParser::object_to_string(self.clone())
    }
}
