fn write_comments(out: &mut Vec<u8>, comments: &[Biff8Comment]) {
    if comments.is_empty() {
        return;
    }
    for (index, comment) in comments.iter().enumerate() {
        let shape_id = 1_025u32.saturating_add(u32::try_from(index).unwrap_or(u32::MAX));
        let drawing = if index == 0 {
            first_comment_drawing(comments.len(), comment, shape_id)
        } else {
            comment_shape(comment, shape_id)
        };
        record(out, MSODRAWING, &drawing);
        record(out, OBJ, &comment_obj(shape_id));
        record(out, MSODRAWING, &[0x00, 0x00, 0x0D, 0xF0, 0, 0, 0, 0]);
        record(out, TXO, &comment_txo(&comment.text));
        write_comment_text_continue(out, &comment.text);
        record(out, CONTINUE, &comment_formatting_continue(&comment.text));
    }
    for (index, comment) in comments.iter().enumerate() {
        let shape_id = 1_025u16.saturating_add(u16::try_from(index).unwrap_or(u16::MAX));
        record(out, NOTE, &comment_note(comment, shape_id));
    }
}

fn comment_drawing_group(comment_count: usize) -> Vec<u8> {
    let mut data = vec![
        0x0F, 0x00, 0x00, 0xF0, 0x52, 0x00, 0x00, 0x00, 0x00, 0x00, 0x06, 0xF0, 0x18, 0x00,
        0x00, 0x00, 0x02, 0x04, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00,
        0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x33, 0x00,
        0x0B, 0xF0, 0x12, 0x00, 0x00, 0x00, 0xBF, 0x00, 0x08, 0x00, 0x08, 0x00, 0x81, 0x01,
        0x41, 0x00, 0x00, 0x08, 0xC0, 0x01, 0x40, 0x00, 0x00, 0x08, 0x40, 0x00, 0x1E, 0xF1,
        0x10, 0x00, 0x00, 0x00, 0x0D, 0x00, 0x00, 0x08, 0x0C, 0x00, 0x00, 0x08, 0x17, 0x00,
        0x00, 0x08, 0xF7, 0x00, 0x00, 0x10,
    ];
    let count = u32::try_from(comment_count).unwrap_or(u32::MAX);
    data[16..20].copy_from_slice(&1_025u32.saturating_add(count).to_le_bytes());
    data[24..28].copy_from_slice(&count.saturating_add(1).to_le_bytes());
    data[36..40].copy_from_slice(&count.saturating_add(1).to_le_bytes());
    data
}

fn first_comment_drawing(count: usize, comment: &Biff8Comment, shape_id: u32) -> Vec<u8> {
    let count = u32::try_from(count).unwrap_or(u32::MAX);
    let mut dg_payload = Vec::new();
    dg_payload.extend_from_slice(&count.saturating_add(1).to_le_bytes());
    dg_payload.extend_from_slice(&1_024u32.saturating_add(count).to_le_bytes());
    let dg = escher_record(0x0010, 0xF008, &dg_payload, None);

    let mut root_payload = Vec::new();
    let mut spgr_payload = Vec::new();
    spgr_payload.extend_from_slice(&0i32.to_le_bytes());
    spgr_payload.extend_from_slice(&0i32.to_le_bytes());
    spgr_payload.extend_from_slice(&1_023i32.to_le_bytes());
    spgr_payload.extend_from_slice(&255i32.to_le_bytes());
    root_payload.extend_from_slice(&escher_record(0x0001, 0xF009, &spgr_payload, None));
    let mut root_sp = Vec::new();
    root_sp.extend_from_slice(&1_024u32.to_le_bytes());
    root_sp.extend_from_slice(&5u32.to_le_bytes());
    root_payload.extend_from_slice(&escher_record(0x0002, 0xF00A, &root_sp, None));
    let root = escher_record(0x000F, 0xF004, &root_payload, None);

    let mut spgr = Vec::new();
    spgr.extend_from_slice(&root);
    spgr.extend_from_slice(&comment_shape(comment, shape_id));
    let declared_spgr = 48u32.saturating_add(count.saturating_mul(134));
    let spgr = escher_record(0x000F, 0xF003, &spgr, Some(declared_spgr));

    let mut container = Vec::new();
    container.extend_from_slice(&dg);
    container.extend_from_slice(&spgr);
    let declared_dg = 72u32.saturating_add(count.saturating_mul(134));
    escher_record(0x000F, 0xF002, &container, Some(declared_dg))
}

fn comment_shape(comment: &Biff8Comment, shape_id: u32) -> Vec<u8> {
    let mut opt: [u8; 60] = [
        0x80, 0x00, 0x2E, 0x0C, 0x00, 0x00, 0x85, 0x00, 0x00, 0x00, 0x00, 0x00, 0x87, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x81, 0x01, 0x50, 0x00, 0x00, 0x08, 0xBF, 0x01, 0x00, 0x00,
        0x01, 0x00, 0xC0, 0x01, 0x40, 0x00, 0x00, 0x08, 0xCB, 0x01, 0x35, 0x25, 0x00, 0x00,
        0xCE, 0x01, 0x00, 0x00, 0x00, 0x00, 0xFF, 0x01, 0x08, 0x00, 0x08, 0x00, 0xBF, 0x03,
        0x02, 0x00, 0x0A, 0x01,
    ];
    let text_id = 0x0C2Cu32.saturating_add(shape_id.saturating_sub(1_024));
    opt[2..6].copy_from_slice(&text_id.to_le_bytes());
    let mut payload = Vec::new();
    let mut sp = Vec::new();
    sp.extend_from_slice(&shape_id.to_le_bytes());
    sp.extend_from_slice(&0x0A00u32.to_le_bytes());
    payload.extend_from_slice(&escher_record(0x0CA2, 0xF00A, &sp, None));
    payload.extend_from_slice(&escher_record(0x00A3, 0xF00B, &opt, None));
    let mut anchor = Vec::with_capacity(18);
    anchor.extend_from_slice(&0u16.to_le_bytes());
    let first_col = u16::from(comment.col.saturating_add(1).min(252));
    let last_col = first_col.saturating_add(3).min(255);
    let first_row = comment.row.saturating_add(1);
    let last_row = first_row.saturating_add(4);
    for value in [first_col, 0, first_row, 0, last_col, 0, last_row, 0] {
        anchor.extend_from_slice(&value.to_le_bytes());
    }
    payload.extend_from_slice(&escher_record(0, 0xF010, &anchor, None));
    payload.extend_from_slice(&escher_record(0, 0xF011, &[], None));
    escher_record(0x000F, 0xF004, &payload, Some(126))
}

fn escher_record(
    options: u16,
    record_type: u16,
    payload: &[u8],
    declared_length: Option<u32>,
) -> Vec<u8> {
    let mut output = Vec::with_capacity(8 + payload.len());
    output.extend_from_slice(&options.to_le_bytes());
    output.extend_from_slice(&record_type.to_le_bytes());
    output.extend_from_slice(
        &declared_length
            .unwrap_or_else(|| u32::try_from(payload.len()).unwrap_or(u32::MAX))
            .to_le_bytes(),
    );
    output.extend_from_slice(payload);
    output
}

fn comment_obj(shape_id: u32) -> Vec<u8> {
    let object_id = u16::try_from(shape_id).unwrap_or(u16::MAX);
    let mut data = Vec::with_capacity(52);
    data.extend_from_slice(&0x0015u16.to_le_bytes());
    data.extend_from_slice(&0x0012u16.to_le_bytes());
    data.extend_from_slice(&0x0019u16.to_le_bytes());
    data.extend_from_slice(&object_id.to_le_bytes());
    data.extend_from_slice(&0x4011u16.to_le_bytes());
    data.extend_from_slice(&[0; 12]);
    data.extend_from_slice(&0x000Du16.to_le_bytes());
    data.extend_from_slice(&0x0016u16.to_le_bytes());
    data.extend_from_slice(&[0; 22]);
    data.extend_from_slice(&0u16.to_le_bytes());
    data.extend_from_slice(&0u16.to_le_bytes());
    data
}

fn comment_txo(text: &str) -> Vec<u8> {
    let length = u16::try_from(text.encode_utf16().count()).unwrap_or(u16::MAX);
    let mut data = Vec::with_capacity(18);
    data.extend_from_slice(&0x0212u16.to_le_bytes());
    data.extend_from_slice(&[0; 8]);
    data.extend_from_slice(&length.to_le_bytes());
    data.extend_from_slice(&0x0010u16.to_le_bytes());
    data.extend_from_slice(&[0; 4]);
    data
}

fn write_comment_text_continue(out: &mut Vec<u8>, text: &str) {
    let wide = text.chars().any(|character| u32::from(character) > 0xFF);
    let mut current = vec![u8::from(wide)];
    for character in text.chars() {
        let encoded = if wide {
            character
                .encode_utf16(&mut [0; 2])
                .iter()
                .flat_map(|unit| unit.to_le_bytes())
                .collect::<Vec<_>>()
        } else {
            vec![u8::try_from(u32::from(character)).unwrap_or(b'?')]
        };
        if current.len() + encoded.len() > MAX_RECORD_DATA {
            record(out, CONTINUE, &current);
            current.clear();
            current.push(u8::from(wide));
        }
        current.extend_from_slice(&encoded);
    }
    record(out, CONTINUE, &current);
}

fn comment_formatting_continue(text: &str) -> Vec<u8> {
    let length = u16::try_from(text.encode_utf16().count()).unwrap_or(u16::MAX);
    let mut data = vec![0; 16];
    data[8..10].copy_from_slice(&length.to_le_bytes());
    data
}

fn comment_note(comment: &Biff8Comment, shape_id: u16) -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(&comment.row.to_le_bytes());
    data.extend_from_slice(&u16::from(comment.col).to_le_bytes());
    data.extend_from_slice(&0u16.to_le_bytes());
    data.extend_from_slice(&shape_id.to_le_bytes());
    let units = comment.author.encode_utf16().collect::<Vec<_>>();
    data.extend_from_slice(&u16::try_from(units.len()).unwrap_or(u16::MAX).to_le_bytes());
    let compressed = units.iter().all(|unit| *unit <= 0xFF);
    data.push(u8::from(!compressed));
    if compressed {
        data.extend(units.into_iter().map(|unit| u8::try_from(unit).unwrap_or(b'?')));
    } else {
        for unit in units {
            data.extend_from_slice(&unit.to_le_bytes());
        }
    }
    data.push(0);
    data
}
