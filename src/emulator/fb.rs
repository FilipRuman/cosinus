use anyhow::Context;
use anyhow::{Result, anyhow, bail};
use log::info;
use minifb::Window;
use minifb::WindowOptions;
use tokio::sync::mpsc;

const WIDTH: usize = 640;
const HEIGHT: usize = 360;

#[derive(Clone, Copy, Debug)]
pub struct PixelOp {
    pub x: usize,
    pub y: usize,
    pub color: u32,
}

#[derive(Clone)]
pub struct FramebufferHandle {
    tx: mpsc::Sender<PixelOp>,
}

impl FramebufferHandle {
    pub fn write(&self, relative_addr: u32, color: u32) -> Result<()> {
        let x = (relative_addr as usize) % WIDTH;
        let y = (relative_addr as usize) / WIDTH;
        if x >= WIDTH || y >= HEIGHT {
            bail!("out of bounds ({}, {})", x, y)
        }

        let op = PixelOp { x, y, color };

        self.tx
            .try_send(op)
            .map_err(|e| anyhow!("channel send failed: {e}"))?;

        Ok(())
    }
}

fn create_framebuffer_channel() -> (FramebufferHandle, mpsc::Receiver<PixelOp>) {
    let (tx, rx) = mpsc::channel::<PixelOp>(10_000);

    (FramebufferHandle { tx }, rx)
}

pub async fn run_framebuffer_loop(mut rx: mpsc::Receiver<PixelOp>) -> Result<()> {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    buffer[25..31].fill(u32::MAX);

    let mut window = Window::new(
        "Test - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )?;

    // Limit to max ~60 fps update rate
    window.set_target_fps(60);
    info!("START WINDOW");

    window.update_with_buffer(&buffer, WIDTH, HEIGHT)?;

    while let Some(first) = rx.recv().await {
        apply_op(&mut buffer, WIDTH, first);

        while let Ok(op) = rx.try_recv() {
            apply_op(&mut buffer, WIDTH, op);
        }
        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window.update_with_buffer(&buffer, WIDTH, HEIGHT)?;
    }
    Ok(())
}

#[inline]
fn apply_op(buffer: &mut Vec<u32>, width: usize, op: PixelOp) {
    let idx = op.y * width + op.x;

    buffer[idx] = op.color;
}
pub fn init() -> Result<(FramebufferHandle, mpsc::Receiver<PixelOp>)> {
    let (frame_buffer_handle, rx) = create_framebuffer_channel();

    Ok((frame_buffer_handle, rx))
}
