use std::process::Command;
use xcb::{
    x::{self, Cw, EventMask, Screen, Window},
    Connection, Event, RequestWithoutReply, VoidCookieChecked,
};

pub struct WindowManager {
    conn: Connection,
    // screen: &Screen
}

impl WindowManager {
    pub fn new() -> Self {
        let (conn, screen_num) = Connection::connect(None).unwrap();
        println!("Connected to X.");
        // WindowManager { conn, screen}
        WindowManager { conn }
    }

    // we might not need to reparent and just add border to current might just be easier tbh 
    fn frame(&self, window_to_frame: Window) -> Window{
        let border_width: u8 = 3;
        let border_color: u32 = 0xff0000;
        let bg_color: u32 = 0x0000ff;

        // match self.conn.wait_for_reply(self.conn.send_request(&x::GetWindowAttributes { window: window_to_frame})) {
        //    Ok(attributes)=> {
        //     let frame: Window = self.conn.generate_id();
        //     self.conn.send_request_checked(&x::CreateWindow {
        //         depth: x::COPY_FROM_PARENT as u8,
        //         wid: frame,
        //         parent: self.conn.get_setup().roots().next().unwrap(),
        //         x:attributes.,
        //         y: 0,


        //     })
        //    }
        //    Err(e) => {
        //     println!("Failed to frame window: {:?}", window_to_frame);
        //     window_to_frame
        //    } 
        // }
        window_to_frame    
    }


    pub fn run(&self) -> xcb::Result<()> {
        let setup = self.conn.get_setup();
        let screen = setup.roots().next().unwrap();

        let values = [Cw::EventMask(
            EventMask::SUBSTRUCTURE_REDIRECT
                | EventMask::SUBSTRUCTURE_NOTIFY
                | EventMask::KEY_PRESS,
        )];

        match self
            .conn
            .send_and_check_request(&x::ChangeWindowAttributes {
                window: screen.root(),
                value_list: &values,
            }) {
            Ok(_) => println!("Succesfully set substructure redirect"),
            Err(e) => println!("Cannot set attributes: {:?}", e),
        }

        loop {
            match self.conn.wait_for_event()? {
                xcb::Event::X(x::Event::KeyPress(ev)) => {
                    println!("Received event: {:?}", ev);
                    if ev.detail() == 0x18 {
                        Command::new("alacritty").spawn();
                    }
                }

                xcb::Event::X(x::Event::MapRequest(ev)) => {
                    println!("Received event: {:?}", ev);
                    match self.conn.send_and_check_request(&x::MapWindow {
                        window: ev.window(),
                    }) {
                        Ok(_) => println!("Succesfully mapped window"),
                        Err(e) => println!("Failed to map window: {:?}", e),
                    }
                }
                ev => {
                    println!("Ignoring event: {:?}", ev);
                }
            }
        }
    }


}
