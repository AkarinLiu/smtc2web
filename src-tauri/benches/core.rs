use criterion::{black_box, criterion_group, criterion_main, Criterion};
use smtc2web_lib::{Song, format_duration};

// Large synthetic base64 cover (~400 KB). The whole point of the /api/now
// micro-benchmarks is to show how expensive per-poll cloning + (re)serialization
// of the album-art payload is — the hotspot the caching optimization targets.
const ART_LEN: usize = 400_000;

fn make_song(with_art: bool) -> Song {
    let mut song = Song {
        title: "Test Song Title - 测试歌名".into(),
        artist: "Some Artist".into(),
        album: "Some Album".into(),
        position: Some("01:23".into()),
        duration: Some("03:45".into()),
        pct: Some(37.2),
        is_playing: true,
        last_update: 1_700_000_000,
        album_art: None,
        font_family: String::new(),
    };
    if with_art {
        let mut s = String::with_capacity(ART_LEN);
        s.push_str("data:image/jpeg;base64,");
        while s.len() < ART_LEN {
            s.push('A');
        }
        song.album_art = Some(s);
    }
    song
}

fn bench_format_duration(c: &mut Criterion) {
    c.bench_function("format_duration", |b| {
        b.iter(|| format_duration(black_box(7_789)))
    });
    c.bench_function("format_duration_zero", |b| {
        b.iter(|| format_duration(black_box(0)))
    });
}

fn bench_song_serialize(c: &mut Criterion) {
    let plain = make_song(false);
    let with_art = make_song(true);

    c.bench_function("song_json_plain", |b| {
        b.iter(|| serde_json::to_vec(black_box(&plain)).unwrap())
    });

    c.bench_function("song_json_with_art", |b| {
        b.iter(|| serde_json::to_vec(black_box(&with_art)).unwrap())
    });

    // "Before": the current /api/now request path — clone the Song (deep-copies
    // the big art String) and re-serialize the whole JSON per browser poll.
    c.bench_function("api_now_clone_serialize_with_art", |b| {
        b.iter(|| {
            let mut s = black_box(&with_art).clone();
            s.font_family = "Segoe UI".into();
            serde_json::to_vec(&s).unwrap()
        })
    });

    // "After": steady-state cache hit — build a cheap fingerprint (small fields
    // + art length only) and memcpy the already-serialized bytes.
    let cached_body = serde_json::to_vec(&with_art).unwrap();
    c.bench_function("api_now_cache_hit_with_art", |b| {
        b.iter(|| {
            let _fp = format!(
                "{}\u{1}{}\u{1}{}\u{1}{}\u{1}{:?}\u{1}{}\u{1}art-len:{}",
                with_art.title,
                with_art.artist,
                with_art.album,
                with_art.is_playing,
                with_art.position,
                "Segoe UI",
                with_art.album_art.as_ref().map(|a| a.len()).unwrap_or(0),
            );
            black_box(&cached_body).clone()
        })
    });
}

criterion_group!(benches, bench_format_duration, bench_song_serialize);
criterion_main!(benches);
