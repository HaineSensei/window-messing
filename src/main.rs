use std::num::NonZeroU32;
use std::sync::Arc;
use softbuffer::{Context, Surface};
use winit::{
    dpi::PhysicalPosition,
    event::WindowEvent,
    event_loop::EventLoop,
    window::Window,
};

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.run_app(&mut App::new()).unwrap();
}

struct App {
    window: Option<Arc<Window>>,
    context: Option<Context<Arc<Window>>>,
    surface: Option<Surface<Arc<Window>, Arc<Window>>>,
    window_position: PhysicalPosition<i32>,
    monitor_size: winit::dpi::PhysicalSize<u32>,
}

impl App {
    fn new() -> Self {
        Self { 
            window: None,
            context: None,
            surface: None,
            window_position: PhysicalPosition::new(0, 0),
            monitor_size: winit::dpi::PhysicalSize::new(0, 0),
        }
    }
}

impl winit::application::ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_none() {
            let monitors: Vec<_> = event_loop.available_monitors().collect();
            let primary_monitor = monitors.first().unwrap();
            let monitor_size = primary_monitor.size();
            self.monitor_size = monitor_size;
            
            let window_size = winit::dpi::PhysicalSize::new(
                monitor_size.width / 2,
                monitor_size.height / 2,
            );
            
            let window_attributes = Window::default_attributes()
                .with_title("Boundary Window")
                .with_inner_size(window_size)
                .with_resizable(false);
            
            let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
            
            self.window_position = window.outer_position().unwrap_or_default();
            
            let context = Context::new(window.clone()).unwrap();
            let surface = Surface::new(&context, window.clone()).unwrap();
            
            self.window = Some(window);
            self.context = Some(context);
            self.surface = Some(surface);
            
            self.redraw();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                self.redraw();
            }
            WindowEvent::Moved(position) => {
                self.window_position = position;
                self.redraw();
            }
            _ => (),
        }
    }
}

impl App {
    fn redraw(&mut self) {
        if let (Some(window), Some(surface)) = (&self.window, &mut self.surface) {
            let size = window.inner_size();
            let width = size.width;
            let height = size.height;
            
            surface.resize(
                NonZeroU32::new(width).unwrap(),
                NonZeroU32::new(height).unwrap(),
            ).unwrap();
            
            let mut buffer = surface.buffer_mut().unwrap();
            
            // Fill with black background
            for pixel in buffer.iter_mut() {
                *pixel = 0xFF000000; // Black
            }
            
            // Calculate which edges are near the screen boundary (100px threshold)
            const BOUNDARY_SIZE: i32 = 100;
            let pos = self.window_position;
            let monitor_width = self.monitor_size.width as i32;
            let monitor_height = self.monitor_size.height as i32;
            
            // Draw green boundaries where appropriate
            for y in 0..height {
                for x in 0..width {
                    let idx = (y * width + x) as usize;
                    let mut draw_green = false;
                    
                    // Calculate world coordinates for this pixel
                    let world_x = pos.x + x as i32;
                    let world_y = pos.y + y as i32;
                    
                    // Left boundary (world x < BOUNDARY_SIZE)
                    if world_x < BOUNDARY_SIZE {
                        draw_green = true;
                    }
                    
                    // Right boundary (world x >= monitor_width - BOUNDARY_SIZE)
                    if world_x >= monitor_width - BOUNDARY_SIZE {
                        draw_green = true;
                    }
                    
                    // Top boundary (world y < BOUNDARY_SIZE)
                    if world_y < BOUNDARY_SIZE {
                        draw_green = true;
                    }
                    
                    // Bottom boundary (world y >= monitor_height - BOUNDARY_SIZE)
                    if world_y >= monitor_height - BOUNDARY_SIZE {
                        draw_green = true;
                    }
                    
                    if draw_green {
                        buffer[idx] = 0xFF00FF00; // Green
                    }
                }
            }
            
            // Test text rendering - position at screen center
            let screen_center_x = monitor_width / 2;
            let screen_center_y = monitor_height / 2;
            
            // Convert screen coordinates to window coordinates
            let text_x = screen_center_x - pos.x - 50; // -50 to roughly center the text
            let text_y = screen_center_y - pos.y;
            
            Self::draw_text(&mut buffer, text_x, text_y, width);
            
            buffer.present().unwrap();
        }
    }
    
    fn draw_text(buffer: &mut [u32], x: i32, y: i32, buffer_width: u32) {
        const TEXT_SOURCE: &str = "Hello_World{}";
        const SCALE: i32 = 3;
        let mut offset_x = 0;
        for byte in TEXT_SOURCE.bytes() {
            if let Some(char_data) = FONT_DATA.get(&byte) {
                Self::draw_char(buffer, x + offset_x, y, char_data, buffer_width, SCALE);
                offset_x += 6 * SCALE; // 5 pixels wide + 1 pixel spacing, scaled
            }
        }
    }
    
    fn draw_char(buffer: &mut [u32], x: i32, y: i32, char_data: &[[bool; 5]; 8], buffer_width: u32, scale: i32) {
        for (row, line) in char_data.iter().enumerate() {
            for (col, &pixel) in line.iter().enumerate() {
                if pixel {
                    // Draw a scale x scale block for each pixel
                    for dy in 0..scale {
                        for dx in 0..scale {
                            let px = x + (col as i32 * scale) + dx;
                            let py = y + (row as i32 * scale) + dy;
                            if px >= 0 && py >= 0 && px < buffer_width as i32 {
                                let idx = (py as u32 * buffer_width + px as u32) as usize;
                                if idx < buffer.len() {
                                    buffer[idx] = 0xFFFFFFFF; // White text
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}


const FONT_DATA: phf::Map<u8, [[bool; 5]; 8]> = phf::phf_map! {
    // Uppercase letters
    b'A' => [
        [false, true, true, true, false],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, true, true, true, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [false, false, false, false, false],
    ],
    b'B' => [
        [true, true, true, true, false],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, true, true, true, false],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, true, true, true, false],
        [false, false, false, false, false],
    ],
    b'C' => [
        [false, true, true, true, false],
        [true, false, false, false, true],
        [true, false, false, false, false],
        [true, false, false, false, false],
        [true, false, false, false, false],
        [true, false, false, false, true],
        [false, true, true, true, false],
        [false, false, false, false, false],
    ],
    b'D' => [
        [true, true, true, false, false],
        [true, false, false, true, false],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, true, false],
        [true, true, true, false, false],
        [false, false, false, false, false],
    ],
    b'E' => [
        [true, true, true, true, true],
        [true, false, false, false, false],
        [true, false, false, false, false],
        [true, true, true, true, false],
        [true, false, false, false, false],
        [true, false, false, false, false],
        [true, true, true, true, true],
        [false, false, false, false, false],
    ],
    b'F' => [
        [true, true, true, true, true],
        [true, false, false, false, false],
        [true, false, false, false, false],
        [true, true, true, true, false],
        [true, false, false, false, false],
        [true, false, false, false, false],
        [true, false, false, false, false],
        [false, false, false, false, false],
    ],
    b'G' => [
        [false, true, true, true, false],
        [true, false, false, false, true],
        [true, false, false, false, false],
        [true, false, true, true, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [false, true, true, true, false],
        [false, false, false, false, false],
    ],
    b'H' => [
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, true, true, true, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [false, false, false, false, false],
    ],
    b'I' => [
        [false, true, true, true, false],
        [false, false, true, false, false],
        [false, false, true, false, false],
        [false, false, true, false, false],
        [false, false, true, false, false],
        [false, false, true, false, false],
        [false, true, true, true, false],
        [false, false, false, false, false],
    ],
    b'J' => [
        [false, false, false, false, true],
        [false, false, false, false, true],
        [false, false, false, false, true],
        [false, false, false, false, true],
        [false, false, false, false, true],
        [true, false, false, false, true],
        [false, true, true, true, false],
        [false, false, false, false, false],
    ],
    b'K' => [
        [true, false, false, false, true],
        [true, false, false, true, false],
        [true, false, true, false, false],
        [true, true, false, false, false],
        [true, false, true, false, false],
        [true, false, false, true, false],
        [true, false, false, false, true],
        [false, false, false, false, false],
    ],
    b'L' => [
        [true, false, false, false, false],
        [true, false, false, false, false],
        [true, false, false, false, false],
        [true, false, false, false, false],
        [true, false, false, false, false],
        [true, false, false, false, false],
        [true, true, true, true, true],
        [false, false, false, false, false],
    ],
    b'M' => [
        [true, false, false, false, true],
        [true, true, false, true, true],
        [true, false, true, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [false, false, false, false, false],
    ],
    b'N' => [
        [true, false, false, false, true],
        [true, true, false, false, true],
        [true, false, true, false, true],
        [true, false, false, true, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [false, false, false, false, false],
    ],
    b'O' => [
        [false, true, true, true, false],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [false, true, true, true, false],
        [false, false, false, false, false],
    ],
    b'P' => [
        [true, true, true, true, false],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, true, true, true, false],
        [true, false, false, false, false],
        [true, false, false, false, false],
        [true, false, false, false, false],
        [false, false, false, false, false],
    ],
    b'Q' => [
        [false, true, true, true, false],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, true, false, true],
        [true, false, false, true, false],
        [false, true, true, false, true],
        [false, false, false, false, false],
    ],
    b'R' => [
        [true, true, true, true, false],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, true, true, true, false],
        [true, false, true, false, false],
        [true, false, false, true, false],
        [true, false, false, false, true],
        [false, false, false, false, false],
    ],
    b'S' => [
        [false, true, true, true, false],
        [true, false, false, false, true],
        [true, false, false, false, false],
        [false, true, true, true, false],
        [false, false, false, false, true],
        [true, false, false, false, true],
        [false, true, true, true, false],
        [false, false, false, false, false],
    ],
    b'T' => [
        [true, true, true, true, true],
        [false, false, true, false, false],
        [false, false, true, false, false],
        [false, false, true, false, false],
        [false, false, true, false, false],
        [false, false, true, false, false],
        [false, false, true, false, false],
        [false, false, false, false, false],
    ],
    b'U' => [
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [false, true, true, true, false],
        [false, false, false, false, false],
    ],
    b'V' => [
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [false, true, false, true, false],
        [false, true, false, true, false],
        [false, false, true, false, false],
        [false, false, false, false, false],
    ],
    b'W' => [
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, true, false, true],
        [true, false, true, false, true],
        [true, true, false, true, true],
        [true, false, false, false, true],
        [false, false, false, false, false],
    ],
    b'X' => [
        [true, false, false, false, true],
        [false, true, false, true, false],
        [false, false, true, false, false],
        [false, false, true, false, false],
        [false, false, true, false, false],
        [false, true, false, true, false],
        [true, false, false, false, true],
        [false, false, false, false, false],
    ],
    b'Y' => [
        [true, false, false, false, true],
        [false, true, false, true, false],
        [false, false, true, false, false],
        [false, false, true, false, false],
        [false, false, true, false, false],
        [false, false, true, false, false],
        [false, false, true, false, false],
        [false, false, false, false, false],
    ],
    b'Z' => [
        [true, true, true, true, true],
        [false, false, false, false, true],
        [false, false, false, true, false],
        [false, false, true, false, false],
        [false, true, false, false, false],
        [true, false, false, false, false],
        [true, true, true, true, true],
        [false, false, false, false, false],
    ],
    
    // Lowercase letters  
    b'a' => [
        [false, false, false, false, false],
        [false, false, false, false, false],
        [false, true, true, true, false],
        [false, false, false, false, true],
        [false, true, true, true, true],
        [true, false, false, false, true],
        [false, true, true, true, true],
        [false, false, false, false, false],
    ],
    b'b' => [
        [true, false, false, false, false],
        [true, false, false, false, false],
        [true, true, true, true, false],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, true, true, true, false],
        [false, false, false, false, false],
    ],
    b'c' => [
        [false, false, false, false, false],
        [false, false, false, false, false],
        [false, true, true, true, false],
        [true, false, false, false, true],
        [true, false, false, false, false],
        [true, false, false, false, true],
        [false, true, true, true, false],
        [false, false, false, false, false],
    ],
    b'd' => [
        [false, false, false, false, true],
        [false, false, false, false, true],
        [false, true, true, true, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [false, true, true, true, true],
        [false, false, false, false, false],
    ],
    b'e' => [
        [false, false, false, false, false],
        [false, false, false, false, false],
        [false, true, true, true, false],
        [true, false, false, false, true],
        [true, true, true, true, true],
        [true, false, false, false, false],
        [false, true, true, true, false],
        [false, false, false, false, false],
    ],
    b'f' => [
        [false, false, true, true, false],
        [false, true, false, false, true],
        [false, true, false, false, false],
        [true, true, true, false, false],
        [false, true, false, false, false],
        [false, true, false, false, false],
        [false, true, false, false, false],
        [false, false, false, false, false],
    ],
    b'g' => [
        [false, false, false, false, false],
        [false, false, false, false, false],
        [false, true, true, true, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [false, true, true, true, true],
        [false, false, false, false, true],
        [false, true, true, true, false],
    ],
    b'h' => [
        [true, false, false, false, false],
        [true, false, false, false, false],
        [true, true, true, true, false],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [false, false, false, false, false],
    ],
    b'i' => [
        [false, false, true, false, false],
        [false, false, false, false, false],
        [false, true, true, false, false],
        [false, false, true, false, false],
        [false, false, true, false, false],
        [false, false, true, false, false],
        [false, true, true, true, false],
        [false, false, false, false, false],
    ],
    b'j' => [
        [false, false, false, true, false],
        [false, false, false, false, false],
        [false, false, true, true, false],
        [false, false, false, true, false],
        [false, false, false, true, false],
        [false, false, false, true, false],
        [true, false, false, true, false],
        [false, true, true, false, false],
    ],
    b'k' => [
        [true, false, false, false, false],
        [true, false, false, false, false],
        [true, false, false, true, false],
        [true, false, true, false, false],
        [true, true, false, false, false],
        [true, false, true, false, false],
        [true, false, false, true, false],
        [false, false, false, false, false],
    ],
    b'l' => [
        [false, true, true, false, false],
        [false, false, true, false, false],
        [false, false, true, false, false],
        [false, false, true, false, false],
        [false, false, true, false, false],
        [false, false, true, false, false],
        [false, true, true, true, false],
        [false, false, false, false, false],
    ],
    b'm' => [
        [false, false, false, false, false],
        [false, false, false, false, false],
        [true, true, false, true, false],
        [true, false, true, false, true],
        [true, false, true, false, true],
        [true, false, true, false, true],
        [true, false, true, false, true],
        [false, false, false, false, false],
    ],
    b'n' => [
        [false, false, false, false, false],
        [false, false, false, false, false],
        [true, true, true, true, false],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [false, false, false, false, false],
    ],
    b'o' => [
        [false, false, false, false, false],
        [false, false, false, false, false],
        [false, true, true, true, false],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [false, true, true, true, false],
        [false, false, false, false, false],
    ],
    b'p' => [
        [false, false, false, false, false],
        [false, false, false, false, false],
        [true, true, true, true, false],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, true, true, true, false],
        [true, false, false, false, false],
        [true, false, false, false, false],
    ],
    b'q' => [
        [false, false, false, false, false],
        [false, false, false, false, false],
        [false, true, true, true, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [false, true, true, true, true],
        [false, false, false, false, true],
        [false, false, false, false, true],
    ],
    b'r' => [
        [false, false, false, false, false],
        [false, false, false, false, false],
        [true, false, true, true, false],
        [true, true, false, false, true],
        [true, false, false, false, false],
        [true, false, false, false, false],
        [true, false, false, false, false],
        [false, false, false, false, false],
    ],
    b's' => [
        [false, false, false, false, false],
        [false, false, false, false, false],
        [false, true, true, true, false],
        [true, false, false, false, false],
        [false, true, true, true, false],
        [false, false, false, false, true],
        [true, true, true, true, false],
        [false, false, false, false, false],
    ],
    b't' => [
        [false, true, false, false, false],
        [false, true, false, false, false],
        [true, true, true, false, false],
        [false, true, false, false, false],
        [false, true, false, false, false],
        [false, true, false, false, true],
        [false, false, true, true, false],
        [false, false, false, false, false],
    ],
    b'u' => [
        [false, false, false, false, false],
        [false, false, false, false, false],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [false, true, true, true, true],
        [false, false, false, false, false],
    ],
    b'v' => [
        [false, false, false, false, false],
        [false, false, false, false, false],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [false, true, false, true, false],
        [false, true, false, true, false],
        [false, false, true, false, false],
        [false, false, false, false, false],
    ],
    b'w' => [
        [false, false, false, false, false],
        [false, false, false, false, false],
        [true, false, false, false, true],
        [true, false, true, false, true],
        [true, false, true, false, true],
        [true, false, true, false, true],
        [false, true, false, true, false],
        [false, false, false, false, false],
    ],
    b'x' => [
        [false, false, false, false, false],
        [false, false, false, false, false],
        [true, false, false, false, true],
        [false, true, false, true, false],
        [false, false, true, false, false],
        [false, true, false, true, false],
        [true, false, false, false, true],
        [false, false, false, false, false],
    ],
    b'y' => [
        [false, false, false, false, false],
        [false, false, false, false, false],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [false, true, true, true, true],
        [false, false, false, false, true],
        [false, true, true, true, false],
    ],
    b'z' => [
        [false, false, false, false, false],
        [false, false, false, false, false],
        [true, true, true, true, true],
        [false, false, false, true, false],
        [false, false, true, false, false],
        [false, true, false, false, false],
        [true, true, true, true, true],
        [false, false, false, false, false],
    ],
    
    // Special characters
    b'{' => [
        [false, false, true, true, false],
        [false, true, false, false, false],
        [false, true, false, false, false],
        [true, false, false, false, false],
        [false, true, false, false, false],
        [false, true, false, false, false],
        [false, false, true, true, false],
        [false, false, false, false, false],
    ],
    b'}' => [
        [false, true, true, false, false],
        [false, false, false, true, false],
        [false, false, false, true, false],
        [false, false, false, false, true],
        [false, false, false, true, false],
        [false, false, false, true, false],
        [false, true, true, false, false],
        [false, false, false, false, false],
    ],
    b'_' => [
        [false, false, false, false, false],
        [false, false, false, false, false],
        [false, false, false, false, false],
        [false, false, false, false, false],
        [false, false, false, false, false],
        [false, false, false, false, false],
        [true, true, true, true, true],
        [false, false, false, false, false],
    ],
    b' ' => [
        [false, false, false, false, false],
        [false, false, false, false, false],
        [false, false, false, false, false],
        [false, false, false, false, false],
        [false, false, false, false, false],
        [false, false, false, false, false],
        [false, false, false, false, false],
        [false, false, false, false, false],
    ],
};
