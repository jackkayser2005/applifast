//! Synthetic UI-side queue preparation timings, not click-to-audible playback.
use std::time::Instant;

use serde_json::json;
use spotifast::apple::{Song, State};

fn main() {
    for count in [100, 1000] {
        let mut state = State::default();
        state.songs.extend((0..count).map(|index| {
            let id = format!("i.example.{index}");
            serde_json::from_value::<Song>(json!({
                "kind":"library", "id":id,
                "playParams":{"id":id,"kind":"song","isLibrary":true},
                "catalogId":null,"title":"Example","artist":"Example",
                "album":"Example","durationMs":180000
            }))
            .unwrap()
        }));
        for run in 0..5 {
            let start = Instant::now();
            let command = state.play(count - 1).unwrap().to_string();
            std::hint::black_box(command);
            println!(
                "{count} songs / {}: {:.2} ms",
                if run == 0 { "first" } else { "switch" },
                start.elapsed().as_secs_f64() * 1000.0
            );
        }
    }
}
