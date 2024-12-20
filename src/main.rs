extern crate sdl2;
extern crate ffmpeg_next;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::video::Window;
use sdl2::render::Canvas;
use sdl2::ttf::Font;
use ffmpeg_next::format::{self, Pixel};
use ffmpeg_next::codec::{self, decoder};
use ffmpeg_next::media::Type;
use ffmpeg_next::software::scaling::{Context, flags};
use std::time::Duration;

fn main() -> Result<(), String> {
    // 初始化 SDL2
    let sdl_context = sdl2::init().map_err(|e| e.to_string())?;
    let video_subsystem = sdl_context.video().map_err(|e| e.to_string())?;

    // 打开一个窗口
    let window = video_subsystem
        .window("MP4 Player with SDL2 and FFmpeg", 800, 600)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    canvas.set_draw_color(Color::BLACK);
    canvas.clear();
    canvas.present();

    // 打开 MP4 文件并初始化 FFmpeg 解码器
    let mut context = format::input(&"path_to_video.mp4").map_err(|e| e.to_string())?;
    let video_stream_index = context
        .streams()
        .best(Type::Video)
        .ok_or_else(|| "No video stream found".to_string())?
        .index();
    
    let mut decoder = decoder::Video::from_stream(&context.stream(video_stream_index).unwrap()).map_err(|e| e.to_string())?;
    let mut scaler = Context::get(
        decoder.format(),
        decoder.width(),
        decoder.height(),
        Pixel::YUV420P,
        decoder.width(),
        decoder.height(),
        flags::BILINEAR,
    )
    .map_err(|e| e.to_string())?;
    
    // SDL2 texture
    let mut texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::YV12, decoder.width() as u32, decoder.height() as u32)
        .map_err(|e| e.to_string())?;

    // 播放视频
    let mut frame_index = 0;
    for (stream, packet) in context.packets() {
        if stream.index() == video_stream_index {
            decoder.send_packet(&packet).map_err(|e| e.to_string())?;
            while let Ok(frame) = decoder.receive_frame() {
                // 使用 FFmpeg 转换帧
                scaler.run(&frame, &mut texture).map_err(|e| e.to_string())?;

                // 渲染到 SDL2
                canvas.clear();
                canvas.copy(&texture, None, Some(Rect::new(0, 0, decoder.width() as u32, decoder.height() as u32)))?;
                canvas.present();

                // 控制帧率（假设每帧持续 40ms）
                std::thread::sleep(Duration::from_millis(40));

                frame_index += 1;
            }
        }
    }

    Ok(())
}
