mod processor;
mod font;

use std::{env, time::Duration};

use processor::Processor;

use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

fn print_usage() {
    println!("Usage: Chip8-Emulator [pathToGame]");
}

fn main() {
    println!("[+] Initializing emulator...");

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return;
    }

    env_logger::init(); // WGPU will fail silently without this 
    
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let window = {
        let size = LogicalSize::new(64, 42);
        WindowBuilder::new()
            .with_title("CHIP8")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(64, 32, surface_texture).unwrap()
    };

    let mut processor = Processor::new();

    println!("[+] Loading rom...");
    processor.load(&args[1]);
    
    println!("[+] Starting emulation cycle...");

    let mut last_tick = std::time::Instant::now();
    let tick_duration = Duration::from_millis(16); 
    
    let mut latest_vram = [false; 64 * 32];

    let res = event_loop.run(|event, elwt| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                println!("[+] Shutting down emulator...");
                elwt.exit();
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                draw(pixels.frame_mut(), &latest_vram);
                
                if let Err(err) = pixels.render() {
                    eprintln!("Render error: {}", err);
                    elwt.exit();
                    return;
                }
            },
            Event::AboutToWait => {
                let now = std::time::Instant::now();
                if now.duration_since(last_tick) >= tick_duration {
                    let state = processor.tick();
                    last_tick = now;

                    if state.vram_updated {
                        latest_vram.copy_from_slice(state.vram);
                        window.request_redraw();
                    }
                }  
            },
            _ => {},
        }
    });
    let _ = res.map_err(|e| Error::UserDefined(Box::new(e)));
}

fn draw(frame: &mut [u8], vram: &[bool; 64*32]) {
    for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
        let on = vram[i];
        let rgba = if on {
            [0xFF, 0xFF, 0xFF, 0xFF] // White
        } else {
            [0x00, 0x00, 0x00, 0xFF] // Black
        };
        pixel.copy_from_slice(&rgba);
    }
}
