use std::thread;
use enigo::MouseControllable;
use websocket::sync::Server;
use websocket::OwnedMessage;
use enigo::Enigo;

fn main() {
	let server = Server::bind("192.168.1.139:2794").unwrap();
	for request in server.filter_map(Result::ok) {
		// Spawn a new thread for each connection.
		thread::spawn(|| {
            let mut enigo = Enigo::new();
			if !request.protocols().contains(&"rust-websocket".to_string()) {
				request.reject().unwrap();
				return;
			}

			let mut client = request.use_protocol("rust-websocket").accept().unwrap();

			let ip = client.peer_addr().unwrap();

			println!("Connection from {}", ip);

            let (width, height) = enigo.main_display_size();
            let (x, y) = enigo.mouse_location();
			let message = OwnedMessage::Text(width.to_string() + ":" + &height.to_string() + ";" + &x.to_string() + ":" + &y.to_string());
			client.send_message(&message).unwrap();

			let (mut receiver, mut sender) = client.split().unwrap();

			for message in receiver.incoming_messages() {
				let message = message.unwrap();
				match message {
                    OwnedMessage::Text(text) => {
                        if text == "click" {
                            enigo.mouse_click(enigo::MouseButton::Left);
                            continue;
                        }
                        if text.starts_with("scroll") {
                            println!("{text}");
                            if text.len() > 6 {
                                enigo.mouse_scroll_y(1);
                                continue;
                            }
                            enigo.mouse_scroll_y(-1);
                            continue;
                        }
                        let data: Vec<i32> = text.split(':').map(|x| x.parse::<i32>().unwrap_or_default()).collect();
                        if data.len() != 4 {
                            continue;
                        }
                        enigo.mouse_move_to(((data[0] as f32) / (data[1] as f32) * width as f32 + 0.5) as i32, ((data[2] as f32) / (data[3] as f32) * height as f32) as i32);
                    }
					OwnedMessage::Close(_) => {
						let message = OwnedMessage::Close(None);
						sender.send_message(&message).unwrap();
						println!("Client {} disconnected", ip);
						return;
					}
					OwnedMessage::Ping(ping) => {
						let message = OwnedMessage::Pong(ping);
						sender.send_message(&message).unwrap();
					}
					_ => sender.send_message(&message).unwrap(),
				}
			}
		});
	}
}
