use crate::transaction::Transaction;
use io_uring::{opcode, types::Fd, IoUring};
use libc::{iovec, msghdr};
use std::{
    net::UdpSocket,
    os::unix::io::AsRawFd,
    sync::mpsc::Receiver,
};
use tungstenite::{connect, Message::Text};
use url::Url;

const BUFFER_SIZE: usize = 1024;

pub struct TransactionReceiver {
    ring: IoUring,
    socket: UdpSocket,
}

impl TransactionReceiver {
    pub fn new(address: &str) -> Self {
        let ring = IoUring::new(256).expect("Failed to create io_uring");
        let socket = UdpSocket::bind(address).expect("Failed to bind UDP socket");
        socket
            .set_nonblocking(true)
            .expect("Failed to set non-blocking mode");

        TransactionReceiver { ring, socket }
    }

    pub fn receive_transaction(&mut self) -> Option<Transaction> {
        // Prepare the buffer and msghdr
        let mut buf = [0u8; BUFFER_SIZE];
        let mut iov = iovec {
            iov_base: buf.as_mut_ptr() as *mut _,
            iov_len: buf.len(),
        };

        let mut msg: msghdr = unsafe { std::mem::zeroed() };
        msg.msg_iov = &mut iov;
        msg.msg_iovlen = 1;

        // Submit the recvmsg operation
        let fd = Fd(self.socket.as_raw_fd());
        let recvmsg_e = opcode::RecvMsg::new(fd, &mut msg).build().user_data(0);

        unsafe { self.ring.submission().push(&recvmsg_e).unwrap() };

        // Wait for the response
        self.ring
            .submit_and_wait(1)
            .expect("Failed to submit and wait");

        let cqe = self.ring.completion().next().unwrap();

        if cqe.result() > 0 {
            // Parse the received transaction
            let tx_data = &buf[..cqe.result() as usize];
            let tx = self.parse_transaction(tx_data);

            // Log the received transaction data
            // println!("Received transaction: {:?}", tx);

            return Some(tx);
        }

        None
    }

    fn parse_transaction(&self, data: &[u8]) -> Transaction {
        // Convert the byte data to a string
        let data_str = String::from_utf8_lossy(data).to_string();

        // Attempt to parse the string as JSON, ensuring it matches the expected structure
        let parsed_json: serde_json::Value =
            serde_json::from_str(&data_str).expect("Failed to parse JSON");

        let pub_key = parsed_json["pub_key"].as_str().unwrap_or("").to_string();
        let signature = parsed_json["signature"].as_str().unwrap_or("").to_string();
        let nonce = parsed_json["nonce"].as_u64().unwrap_or(0);
        let payload = parsed_json["payload"].as_str().unwrap_or("").to_string();

        Transaction::new(pub_key, signature, nonce, payload)
    }
}

pub fn server(receiver_channel: Receiver<String>) {
    let url_string = "ws://localhost:5050";

    // Connect to the WebSocket server
    let url = Url::parse(&url_string).expect("Invalid URL");

    // Use Tungstenite to perform the WebSocket handshake
    let (mut websocket, _response) =
        connect(url.to_string()).expect("Failed to connect to WebSocket server");

    loop {
        // Prepare the message to send
        let message = receiver_channel.recv().unwrap();
        println!("Message to send: {}", message);
        
        websocket.write(Text(message)).expect("Failed to send message");
        websocket.flush().expect("Flushing error");
    }
}
