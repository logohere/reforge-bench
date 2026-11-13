use super::*;

#[test]
fn decoded_images_and_decode_errors_invalidate_the_loading_view() {
    for bytes in [
        include_bytes!("../../../../../web/assets/brand/rea-app.png").to_vec(),
        b"damaged PNG".to_vec(),
    ] {
        let (sender, receiver) = mpsc::channel();
        let wake: rune::Waker = std::sync::Arc::new(move || {
            let _ = sender.send(());
        });
        let cache = ImageCache::new(wake.clone());
        let mut timeline = Timeline {
            image: Some(cache.load(bytes)),
            ..Default::default()
        };
        receiver
            .recv_timeout(std::time::Duration::from_secs(5))
            .unwrap();
        assert!(timeline.poll(1, Some(wake.clone()), 0.0, &mut Tx::default()));
        assert!(timeline.image_ready);
        assert!(!timeline.poll(1, Some(wake), 0.0, &mut Tx::default()));
    }
}

#[test]
fn clustering_preserves_every_event_and_nonoverlapping_hit_targets() {
    let times: Vec<_> = (0..120).map(|i| if i < 50 { i } else { i * 8 }).collect();
    let indices: Vec<_> = (0..times.len()).collect();
    for width in [240.0, 400.0, 720.0, 1100.0] {
        let markers = model::markers(&indices, &times, 0.0, 1000.0, width);
        assert_eq!(
            markers
                .iter()
                .flat_map(|marker| marker.indices.clone())
                .collect::<Vec<_>>(),
            indices
        );
        for marker in &markers {
            assert!(marker.x - marker.width / 2.0 >= 0.0 && marker.x + marker.width / 2.0 <= width);
        }
        for pair in markers.windows(2) {
            assert!(pair[0].x + pair[0].width / 2.0 + 12.0 <= pair[1].x - pair[1].width / 2.0);
        }
    }
}
