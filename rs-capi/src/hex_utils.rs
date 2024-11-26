pub fn string_to_hex(text: &str, model_name: &str) -> Vec<u8> {
    let text_bytes = text.as_bytes();
    let text_length = text_bytes.len();

    // 固定常量
    const FIXED_HEADER: usize = 2;
    const SEPARATOR: usize = 1;

    let model_name_bytes = model_name.as_bytes();
    let fixed_suffix_length = 0xA3 + model_name_bytes.len();

    // 计算第一个长度字段
    let (text_length_field1, text_length_field_size1) = if text_length < 128 {
        (format!("{:02x}", text_length), 1)
    } else {
        let low_byte1 = (text_length & 0x7F) | 0x80;
        let high_byte1 = (text_length >> 7) & 0xFF;
        (format!("{:02x}{:02x}", low_byte1, high_byte1), 2)
    };

    // 计算基础长度字段
    let base_length = text_length + 0x2A;
    let (text_length_field, text_length_field_size) = if base_length < 128 {
        (format!("{:02x}", base_length), 1)
    } else {
        let low_byte = (base_length & 0x7F) | 0x80;
        let high_byte = (base_length >> 7) & 0xFF;
        (format!("{:02x}{:02x}", low_byte, high_byte), 2)
    };

    // 计算总消息长度
    let message_total_length = FIXED_HEADER
        + text_length_field_size
        + SEPARATOR
        + text_length_field_size1
        + text_length
        + fixed_suffix_length;

    // 构造十六进制字符串
    let model_name_length_hex = format!("{:02X}", model_name_bytes.len());
    let model_name_hex = hex::encode_upper(model_name_bytes);

    let hex_string = format!(
        "{:010x}\
        12{}\
        0A{}\
        {}\
        10016A2432343163636435662D393162612D343131382D393239612D3936626330313631626432612\
        2002A132F643A2F6964656150726F2F656475626F73733A1E0A\
        {}{}\
        22004A\
        2461383761396133342D323164642D343863372D623434662D616636633365636536663765\
        680070007A2436393337376535612D386332642D343835342D623564392D653062623232336163303061\
        800101B00100C00100E00100E80100",
        message_total_length,
        text_length_field,
        text_length_field1,
        hex::encode_upper(text_bytes),
        model_name_length_hex,
        model_name_hex
    )
    .to_uppercase();

    // 将十六进制字符串转换为字节数组
    hex::decode(hex_string).unwrap_or_default()
}

pub fn chunk_to_utf8_string(chunk: &[u8]) -> String {
    if chunk.len() < 2 {
        return String::new();
    }

    if chunk[0] == 0x01 || chunk[0] == 0x02 || (chunk[0] == 0x60 && chunk[1] == 0x0C) {
        return String::new();
    }

    // 尝试找到0x0A并从其后开始处理
    let chunk = match chunk.iter().position(|&x| x == 0x0A) {
        Some(pos) => &chunk[pos + 1..],
        None => chunk,
    };

    let mut filtered_chunk = Vec::new();
    let mut i = 0;

    while i < chunk.len() {
        // 检查是否有连续的0x00
        if i + 4 <= chunk.len() && chunk[i..i + 4].iter().all(|&x| x == 0x00) {
            i += 4;
            while i < chunk.len() && chunk[i] <= 0x0F {
                i += 1;
            }
            continue;
        }

        if chunk[i] == 0x0C {
            i += 1;
            while i < chunk.len() && chunk[i] == 0x0A {
                i += 1;
            }
        } else {
            filtered_chunk.push(chunk[i]);
            i += 1;
        }
    }

    // 过滤掉特定字节
    filtered_chunk.retain(|&b| b != 0x00 && b != 0x0C);

    if filtered_chunk.is_empty() {
        return String::new();
    }

    // 转换为UTF-8字符串
    String::from_utf8_lossy(&filtered_chunk).trim().to_string()
}
