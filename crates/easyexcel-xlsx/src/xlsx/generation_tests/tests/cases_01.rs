    #[test]
    fn image_generation_rejects_invalid_buffers_and_coordinates() {
        assert!(image_from_buffer(&[0_u8; 7]).is_err());
        let mut worksheet = Worksheet::new();
        assert!(
            insert_scaled_image(
                &mut worksheet,
                u32::MAX,
                0,
                &tiny_png(),
                1,
                1,
                ObjectMovement::MoveAndSizeWithCells,
                0,
                0,
            )
            .is_err()
        );
    }
