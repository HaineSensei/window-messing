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
            
            // Check current position vs stored position
            let current_pos = window.outer_position().unwrap_or_default();
            if current_pos != self.window_position {
                self.window_position = current_pos;
            }
            
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
            
            
            // Position text way off screen above the monitor
            let off_screen_x = monitor_width / 2; // Keep horizontally centered
            let off_screen_y = -(monitor_height as i32) - 1000; // Well above screen
            
            // Convert off-screen coordinates to window coordinates
            let text_x = off_screen_x - pos.x;
            let text_y = off_screen_y - pos.y;
            
            
            Self::draw_text(&mut buffer, text_x, text_y, width);
            
            buffer.present().unwrap();
        }
    }
    
    fn draw_text(buffer: &mut [u32], x: i32, y: i32, buffer_width: u32) {
        const SCALE: i32 = 3;
        let mut offset_x = 0;
        for char_data in TEXT_BITMAPS.iter() {
            Self::draw_char(buffer, x + offset_x, y, char_data, buffer_width, SCALE);
            offset_x += 6 * SCALE; // 5 pixels wide + 1 pixel spacing, scaled
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

const fn string_to_bytes<const N:usize>(s: &str) -> Option<[u8; N]> {
    if s.len() == N {
        let mut i = 0;
        let mut out: [u8; N] = [0; N];
        while i < N {
            out[i] = s.as_bytes()[i];
            i += 1;
        }
        Some(out)
    } else {
        None
    }
}

const LEN: usize = 28;
const TEXT_SOURCE: [u8;LEN] = string_to_bytes("ictf{Teeheehee_you_found_me}").unwrap();

const fn text_to_bitmap<const N:usize>(text: &[u8;N]) -> Option<[[[bool; 5]; 8]; N]> {
    let mut result = [[[false; 5]; 8]; N];
    let mut i = 0;
    while i < N {
        let char = text[i];
        if let Some(char_index) = index_u8(&LETTER_DATA, char) {
            result[i] = FONT_DATA[char_index];
        } else {
            return None; // Invalid character
        }
        i+= 1;
    }
    Some(result)
}

const fn index_u8<const N: usize>(arr: &[u8; N], element: u8) -> Option<usize> {
    let l = arr.len();
    let mut i = 0;
    while i < l {
        if arr[i] == element {
            return Some(i);
        }
        i += 1;
    }
    None
}

const TEXT_BITMAPS: [[[bool; 5]; 8]; LEN] = text_to_bitmap(&TEXT_SOURCE).unwrap();

const LETTER_DATA: [u8; 26 + 26 + 4] = [
    // Uppercase letters A-Z
    b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'J', b'K', b'L', b'M',
    b'N', b'O', b'P', b'Q', b'R', b'S', b'T', b'U', b'V', b'W', b'X', b'Y', b'Z',
    // Lowercase letters a-z
    b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'i', b'j', b'k', b'l', b'm',
    b'n', b'o', b'p', b'q', b'r', b's', b't', b'u', b'v', b'w', b'x', b'y', b'z',
    // Special characters: space, underscore, curly braces
    b'{', b'}', b'_', b' ',
];

const FONT_DATA: [[[bool; 5]; 8];26+26+4] = [
    // Uppercase letters
    [
        [false, true, true, true, false],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, true, true, true, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [false, false, false, false, false],
    ],
    [
        [true, true, true, true, false],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, true, true, true, false],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, true, true, true, false],
        [false, false, false, false, false],
    ],
    [
        [false, true, true, true, false],
        [true, false, false, false, true],
        [true, false, false, false, false],
        [true, false, false, false, false],
        [true, false, false, false, false],
        [true, false, false, false, true],
        [false, true, true, true, false],
        [false, false, false, false, false],
    ],
    [
        [true, true, true, false, false],
        [true, false, false, true, false],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, true, false],
        [true, true, true, false, false],
        [false, false, false, false, false],
    ],
    [
        [true, true, true, true, true],
        [true, false, false, false, false],
        [true, false, false, false, false],
        [true, true, true, true, false],
        [true, false, false, false, false],
        [true, false, false, false, false],
        [true, true, true, true, true],
        [false, false, false, false, false],
    ],
    [
        [true, true, true, true, true],
        [true, false, false, false, false],
        [true, false, false, false, false],
        [true, true, true, true, false],
        [true, false, false, false, false],
        [true, false, false, false, false],
        [true, false, false, false, false],
        [false, false, false, false, false],
    ],
    [
        [false, true, true, true, false],
        [true, false, false, false, true],
        [true, false, false, false, false],
        [true, false, true, true, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [false, true, true, true, false],
        [false, false, false, false, false],
    ],
    [
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, true, true, true, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [false, false, false, false, false],
    ],
    [
        [false, true, true, true, false],
        [false, false, true, false, false],
        [false, false, true, false, false],
        [false, false, true, false, false],
        [false, false, true, false, false],
        [false, false, true, false, false],
        [false, true, true, true, false],
        [false, false, false, false, false],
    ],
    [
        [false, false, false, false, true],
        [false, false, false, false, true],
        [false, false, false, false, true],
        [false, false, false, false, true],
        [false, false, false, false, true],
        [true, false, false, false, true],
        [false, true, true, true, false],
        [false, false, false, false, false],
    ],
    [
        [true, false, false, false, true],
        [true, false, false, true, false],
        [true, false, true, false, false],
        [true, true, false, false, false],
        [true, false, true, false, false],
        [true, false, false, true, false],
        [true, false, false, false, true],
        [false, false, false, false, false],
    ],
    [
        [true, false, false, false, false],
        [true, false, false, false, false],
        [true, false, false, false, false],
        [true, false, false, false, false],
        [true, false, false, false, false],
        [true, false, false, false, false],
        [true, true, true, true, true],
        [false, false, false, false, false],
    ],
    [
        [true, false, false, false, true],
        [true, true, false, true, true],
        [true, false, true, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [false, false, false, false, false],
    ],
    [
        [true, false, false, false, true],
        [true, true, false, false, true],
        [true, false, true, false, true],
        [true, false, false, true, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [false, false, false, false, false],
    ],
    [
        [false, true, true, true, false],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [false, true, true, true, false],
        [false, false, false, false, false],
    ],
    [
        [true, true, true, true, false],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, true, true, true, false],
        [true, false, false, false, false],
        [true, false, false, false, false],
        [true, false, false, false, false],
        [false, false, false, false, false],
    ],
    [
        [false, true, true, true, false],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, true, false, true],
        [true, false, false, true, false],
        [false, true, true, false, true],
        [false, false, false, false, false],
    ],
    [
        [true, true, true, true, false],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, true, true, true, false],
        [true, false, true, false, false],
        [true, false, false, true, false],
        [true, false, false, false, true],
        [false, false, false, false, false],
    ],
    [
        [false, true, true, true, false],
        [true, false, false, false, true],
        [true, false, false, false, false],
        [false, true, true, true, false],
        [false, false, false, false, true],
        [true, false, false, false, true],
        [false, true, true, true, false],
        [false, false, false, false, false],
    ],
    [
        [true, true, true, true, true],
        [false, false, true, false, false],
        [false, false, true, false, false],
        [false, false, true, false, false],
        [false, false, true, false, false],
        [false, false, true, false, false],
        [false, false, true, false, false],
        [false, false, false, false, false],
    ],
    [
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [false, true, true, true, false],
        [false, false, false, false, false],
    ],
    [
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [false, true, false, true, false],
        [false, true, false, true, false],
        [false, false, true, false, false],
        [false, false, false, false, false],
    ],
    [
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, true, false, true],
        [true, false, true, false, true],
        [true, true, false, true, true],
        [true, false, false, false, true],
        [false, false, false, false, false],
    ],
    [
        [true, false, false, false, true],
        [false, true, false, true, false],
        [false, false, true, false, false],
        [false, false, true, false, false],
        [false, false, true, false, false],
        [false, true, false, true, false],
        [true, false, false, false, true],
        [false, false, false, false, false],
    ],
    [
        [true, false, false, false, true],
        [false, true, false, true, false],
        [false, false, true, false, false],
        [false, false, true, false, false],
        [false, false, true, false, false],
        [false, false, true, false, false],
        [false, false, true, false, false],
        [false, false, false, false, false],
    ],
    [
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
    [
        [false, false, false, false, false],
        [false, false, false, false, false],
        [false, true, true, true, false],
        [false, false, false, false, true],
        [false, true, true, true, true],
        [true, false, false, false, true],
        [false, true, true, true, true],
        [false, false, false, false, false],
    ],
    [
        [true, false, false, false, false],
        [true, false, false, false, false],
        [true, true, true, true, false],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, true, true, true, false],
        [false, false, false, false, false],
    ],
    [
        [false, false, false, false, false],
        [false, false, false, false, false],
        [false, true, true, true, false],
        [true, false, false, false, true],
        [true, false, false, false, false],
        [true, false, false, false, true],
        [false, true, true, true, false],
        [false, false, false, false, false],
    ],
    [
        [false, false, false, false, true],
        [false, false, false, false, true],
        [false, true, true, true, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [false, true, true, true, true],
        [false, false, false, false, false],
    ],
    [
        [false, false, false, false, false],
        [false, false, false, false, false],
        [false, true, true, true, false],
        [true, false, false, false, true],
        [true, true, true, true, true],
        [true, false, false, false, false],
        [false, true, true, true, false],
        [false, false, false, false, false],
    ],
    [
        [false, false, true, true, false],
        [false, true, false, false, true],
        [false, true, false, false, false],
        [true, true, true, false, false],
        [false, true, false, false, false],
        [false, true, false, false, false],
        [false, true, false, false, false],
        [false, false, false, false, false],
    ],
    [
        [false, false, false, false, false],
        [false, false, false, false, false],
        [false, true, true, true, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [false, true, true, true, true],
        [false, false, false, false, true],
        [false, true, true, true, false],
    ],
    [
        [true, false, false, false, false],
        [true, false, false, false, false],
        [true, true, true, true, false],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [false, false, false, false, false],
    ],
    [
        [false, false, true, false, false],
        [false, false, false, false, false],
        [false, true, true, false, false],
        [false, false, true, false, false],
        [false, false, true, false, false],
        [false, false, true, false, false],
        [false, true, true, true, false],
        [false, false, false, false, false],
    ],
    [
        [false, false, false, true, false],
        [false, false, false, false, false],
        [false, false, true, true, false],
        [false, false, false, true, false],
        [false, false, false, true, false],
        [false, false, false, true, false],
        [true, false, false, true, false],
        [false, true, true, false, false],
    ],
    [
        [true, false, false, false, false],
        [true, false, false, false, false],
        [true, false, false, true, false],
        [true, false, true, false, false],
        [true, true, false, false, false],
        [true, false, true, false, false],
        [true, false, false, true, false],
        [false, false, false, false, false],
    ],
    [
        [false, true, true, false, false],
        [false, false, true, false, false],
        [false, false, true, false, false],
        [false, false, true, false, false],
        [false, false, true, false, false],
        [false, false, true, false, false],
        [false, true, true, true, false],
        [false, false, false, false, false],
    ],
    [
        [false, false, false, false, false],
        [false, false, false, false, false],
        [true, true, false, true, false],
        [true, false, true, false, true],
        [true, false, true, false, true],
        [true, false, true, false, true],
        [true, false, true, false, true],
        [false, false, false, false, false],
    ],
    [
        [false, false, false, false, false],
        [false, false, false, false, false],
        [true, true, true, true, false],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [false, false, false, false, false],
    ],
    [
        [false, false, false, false, false],
        [false, false, false, false, false],
        [false, true, true, true, false],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [false, true, true, true, false],
        [false, false, false, false, false],
    ],
    [
        [false, false, false, false, false],
        [false, false, false, false, false],
        [true, true, true, true, false],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, true, true, true, false],
        [true, false, false, false, false],
        [true, false, false, false, false],
    ],
    [
        [false, false, false, false, false],
        [false, false, false, false, false],
        [false, true, true, true, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [false, true, true, true, true],
        [false, false, false, false, true],
        [false, false, false, false, true],
    ],
    [
        [false, false, false, false, false],
        [false, false, false, false, false],
        [true, false, true, true, false],
        [true, true, false, false, true],
        [true, false, false, false, false],
        [true, false, false, false, false],
        [true, false, false, false, false],
        [false, false, false, false, false],
    ],
    [
        [false, false, false, false, false],
        [false, false, false, false, false],
        [false, true, true, true, false],
        [true, false, false, false, false],
        [false, true, true, true, false],
        [false, false, false, false, true],
        [true, true, true, true, false],
        [false, false, false, false, false],
    ],
    [
        [false, true, false, false, false],
        [false, true, false, false, false],
        [true, true, true, false, false],
        [false, true, false, false, false],
        [false, true, false, false, false],
        [false, true, false, false, true],
        [false, false, true, true, false],
        [false, false, false, false, false],
    ],
    [
        [false, false, false, false, false],
        [false, false, false, false, false],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [false, true, true, true, true],
        [false, false, false, false, false],
    ],
    [
        [false, false, false, false, false],
        [false, false, false, false, false],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [false, true, false, true, false],
        [false, true, false, true, false],
        [false, false, true, false, false],
        [false, false, false, false, false],
    ],
    [
        [false, false, false, false, false],
        [false, false, false, false, false],
        [true, false, false, false, true],
        [true, false, true, false, true],
        [true, false, true, false, true],
        [true, false, true, false, true],
        [false, true, false, true, false],
        [false, false, false, false, false],
    ],
    [
        [false, false, false, false, false],
        [false, false, false, false, false],
        [true, false, false, false, true],
        [false, true, false, true, false],
        [false, false, true, false, false],
        [false, true, false, true, false],
        [true, false, false, false, true],
        [false, false, false, false, false],
    ],
    [
        [false, false, false, false, false],
        [false, false, false, false, false],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],
        [false, true, true, true, true],
        [false, false, false, false, true],
        [false, true, true, true, false],
    ],
    [
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
    [
        [false, false, true, true, false],
        [false, true, false, false, false],
        [false, true, false, false, false],
        [true, false, false, false, false],
        [false, true, false, false, false],
        [false, true, false, false, false],
        [false, false, true, true, false],
        [false, false, false, false, false],
    ],
    [
        [false, true, true, false, false],
        [false, false, false, true, false],
        [false, false, false, true, false],
        [false, false, false, false, true],
        [false, false, false, true, false],
        [false, false, false, true, false],
        [false, true, true, false, false],
        [false, false, false, false, false],
    ],
    [
        [false, false, false, false, false],
        [false, false, false, false, false],
        [false, false, false, false, false],
        [false, false, false, false, false],
        [false, false, false, false, false],
        [false, false, false, false, false],
        [true, true, true, true, true],
        [false, false, false, false, false],
    ],
    [
        [false, false, false, false, false],
        [false, false, false, false, false],
        [false, false, false, false, false],
        [false, false, false, false, false],
        [false, false, false, false, false],
        [false, false, false, false, false],
        [false, false, false, false, false],
        [false, false, false, false, false],
    ],
];
