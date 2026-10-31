use macroquad::prelude::*;

#[macroquad::main("BasicShapes")]
async fn main() {
    let mut i: i16 = 0;
    loop {
        clear_background(RED);

        draw_line(i as f32 + 40.0, 40.0, 100.0, 200.0, 15.0, BLUE);
        draw_rectangle(screen_width() / 2.0 - 60.0, 100.0, 120.0, 60.0, GREEN);
        draw_circle(screen_width() - 30.0, screen_height() - 30.0, 15.0, YELLOW);
        draw_text("HELLO", 20.0, 20.0, 20.0, DARKGRAY);
        i += 1;
        next_frame().await
    }
}
