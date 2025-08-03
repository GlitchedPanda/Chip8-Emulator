mod processor;
mod font;

use std::{env, thread, time::Duration};

use processor::Processor;
//use winit::event_loop::{ControlFlow, EventLoop};

fn print_usage() {
    println!("Usage: Chip8Emulator [pathToGame]");
}

fn main() {
    println!("[+] Initializing emulator...");

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return;
    }

    //env_logger::init(); // WGPU will fail silently without this

    //let event_loop = EventLoop::new().unwrap();
    //event_loop.set_control_flow(ControlFlow::Wait);

    //let mut app = App::default();
    //event_loop.run_app(&app);

    let mut processor = Processor::new();

    println!("[+] Loading rom...");
    processor.load(&args[1]);
    
    println!("[+] Starting emulation cycle...");
    loop {
        let result = processor.tick();
        if result.vram_updated {
            // Draw to screen here
            for pixel in 0..result.vram.len() {
                print!("{} ", result.vram[pixel as usize]);
            }
            println!();
        }

        thread::sleep(Duration::from_millis(2));
    }
}
