use des::{cipher::generic_array::GenericArray, cipher::BlockEncrypt, cipher::KeyInit, Des};

/// 加密登录参数
/// 完全复刻 JavaScript 版本的加密逻辑
pub fn encode_login_params(
    username: &str,
    password: &str,
    sessionid: &str,
    deskey: &str,
    randnumber: &str,
    nowtime: &str,
) -> Result<String, String> {
    // 1. 计算密码策略（简化版本）
    let password_policy = check_password_policy(username, password);
    let txt_mm_expression = calculate_password_expression(password);
    let txt_mm_length = password.len();
    let txt_mm_userzh = if password.to_lowercase().contains(&username.to_lowercase()) {
        "1"
    } else {
        "0"
    };

    // 2. 密码加密: MD5(MD5(password) + MD5(randnumber))
    let password_hash = format!(
        "{}{}",
        md5_simple(password),
        md5_simple(&randnumber.to_lowercase())
    );
    let password_encrypted = md5_simple(&password_hash);

    // 3. 用户名编码: Base64(username + ";;" + sessionid)
    let username_encoded = format!("{};;{}", username, sessionid);
    let username_base64 = base64_encode(&username_encoded);

    // 4. 构建参数字符串
    let p_username = format!("_u{}", randnumber);
    let p_password = format!("_p{}", randnumber);
    let params = format!(
        "{}={}&{}={}&randnumber={}&isPasswordPolicy={}&txt_mm_expression={}&txt_mm_length={}&txt_mm_userzh={}",
        p_username, username_base64,
        p_password, password_encrypted,
        randnumber,
        password_policy,
        txt_mm_expression,
        txt_mm_length,
        txt_mm_userzh
    );

    // 5. DES 加密参数
    let encrypted = des_encrypt(&params, deskey)?;

    // 6. Base64 编码加密后的参数
    let params_base64 = base64_encode(&encrypted);

    // 7. Token 生成: MD5(MD5(params) + MD5(timestamp))
    let token_hash = format!("{}{}", md5_simple(&params), md5_simple(nowtime));
    let token = md5_simple(&token_hash);

    // 8. 最终参数
    let final_params = format!("params={}&token={}&timestamp={}", params_base64, token, nowtime);

    Ok(final_params)
}

/// 密码策略检查（简化版本）
fn check_password_policy(username: &str, password: &str) -> &'static str {
    if password.is_empty() || password == username || password.len() < 6 {
        "0"
    } else {
        "1"
    }
}

/// 计算密码表达式（字符类型位掩码）
fn calculate_password_expression(password: &str) -> u32 {
    let mut result = 0u32;
    for c in password.chars() {
        let code = c as u32;
        if code >= 48 && code <= 57 {
            // 数字
            result |= 8;
        } else if code >= 97 && code <= 122 {
            // 小写字母
            result |= 4;
        } else if code >= 65 && code <= 90 {
            // 大写字母
            result |= 2;
        } else {
            // 其他字符
            result |= 1;
        }
    }
    result
}

/// 简单的 MD5 哈希
fn md5_simple(data: &str) -> String {
    format!("{:x}", md5::compute(data))
}

/// Base64 编码
fn base64_encode(data: &str) -> String {
    use base64::prelude::*;
    BASE64_STANDARD.encode(data.as_bytes())
}

/// DES 加密（ECB 模式）
/// 这是对 JavaScript 版本 strEnc 函数的复刻
fn des_encrypt(data: &str, key: &str) -> Result<String, String> {
    // 将密钥转换为 8 字节（64 位）
    let mut key_bytes = [0u8; 8];
    for (i, byte) in key.bytes().take(8).enumerate() {
        key_bytes[i] = byte;
    }

    // 创建 DES 加密器（ECB 模式）
    let cipher = Des::new(&GenericArray::from(key_bytes));

    // PKCS#7 填充
    let block_size = 8;
    let data_bytes = data.as_bytes();
    let padding_len = block_size - (data_bytes.len() % block_size);
    let padded_len = data_bytes.len() + padding_len;

    let mut padded_data = Vec::with_capacity(padded_len);
    padded_data.extend_from_slice(data_bytes);
    for _ in 0..padding_len {
        padded_data.push(padding_len as u8);
    }

    // 加密（按 8 字节块处理 ECB 模式）
    let mut encrypted = Vec::with_capacity(padded_len);
    for chunk in padded_data.chunks(8) {
        let mut block = GenericArray::clone_from_slice(chunk);
        cipher.encrypt_block(&mut block);
        encrypted.extend_from_slice(block.as_slice());
    }

    // 转换为十六进制字符串（与 JavaScript 版本一致）
    Ok(encrypted
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_md5() {
        assert_eq!(md5_simple("abc"), "900150983cd24fb0d6963f7d28e17f72");
    }

    #[test]
    fn test_base64_encode() {
        let encoded = base64_encode("hello");
        assert_eq!(encoded, "aGVsbG8=");
    }

    #[test]
    fn test_password_expression() {
        // 纯数字
        assert_eq!(calculate_password_expression("123456"), 8);
        // 纯小写字母
        assert_eq!(calculate_password_expression("abcdef"), 4);
        // 纯大写字母
        assert_eq!(calculate_password_expression("ABCDEF"), 2);
        // 混合
        let expr = calculate_password_expression("Abc123");
        // 应该包含 数字(8) + 小写(4) + 大写(2)
        assert_eq!(expr, 8 | 4 | 2);
    }
}

/// ============================================================
/// 持久化登录密码加密模块 (DPAPI)
/// ============================================================

/// 使用 Windows DPAPI 加密密码
///
/// # 安全性
/// - 使用当前用户密钥加密
/// - 只有加密的 Windows 用户才能解密
/// - 即使文件被复制到其他机器也无法解密
///
/// # 返回
/// 加密后的 Base64 字符串
#[cfg(target_os = "windows")]
pub fn encrypt_password_dpapi(password: &str) -> Result<String, String> {
    use windows::Win32::Security::Cryptography::*;
    use windows::Win32::Foundation::HLOCAL;
    use base64::Engine;

    // 将密码转换为 UTF-16 字节数组
    let password_bytes: Vec<u16> = password.encode_utf16().collect();
    let password_len = password_bytes.len() * 2;

    // 调用 CryptProtectData (DPAPI)
    let mut blob_out = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: std::ptr::null_mut(),
    };

    let blob_in = CRYPT_INTEGER_BLOB {
        cbData: password_len as u32,
        pbData: password_bytes.as_ptr() as *mut u8,
    };

    unsafe {
        let success = CryptProtectData(
            &blob_in,
            None,  // 描述(可选)
            None,  // 可选熵
            None,  // 保留
            None,  // 提示句柄(可选)
            CRYPTPROTECT_UI_FORBIDDEN, // 禁用UI
            &mut blob_out,  // 输出 blob
        );

        if success.is_err() {
            return Err("DPAPI 加密失败".to_string());
        }

        // 获取加密后的数据
        let encrypted_bytes = std::slice::from_raw_parts(
            blob_out.pbData,
            blob_out.cbData as usize,
        );

        let base64_encoded = base64::engine::general_purpose::STANDARD.encode(encrypted_bytes);

        // 释放内存（不释放，让系统自动管理）
        // Windows DPAPI 会管理这个内存

        Ok(base64_encoded)
    }
}

/// 使用 Windows DPAPI 解密密码
#[cfg(target_os = "windows")]
pub fn decrypt_password_dpapi(encrypted_base64: &str) -> Result<String, String> {
    use windows::Win32::Security::Cryptography::*;
    use windows::Win32::Foundation::HLOCAL;
    use base64::Engine;

    // 解码 Base64
    let encrypted_bytes = base64::engine::general_purpose::STANDARD.decode(encrypted_base64)
        .map_err(|e| format!("Base64 解码失败: {}", e))?;

    let mut blob_out = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: std::ptr::null_mut(),
    };

    let blob_in = CRYPT_INTEGER_BLOB {
        cbData: encrypted_bytes.len() as u32,
        pbData: encrypted_bytes.as_ptr() as *mut u8,
    };

    unsafe {
        let success = CryptUnprotectData(
            &blob_in,
            None,  // 描述(可选)
            None,  // 可选熵
            None,  // 保留
            None,  // 提示句柄(可选)
            blob_out.cbData,  // 输出 blob 大小
            &mut blob_out,  // 输出 blob
        );

        if success.is_err() {
            return Err("DPAPI 解密失败(可能是用户不匹配)".to_string());
        }

        // 转换 UTF-16 字节数组为 String
        let u16_slice = std::slice::from_raw_parts(
            blob_out.pbData as *const u16,
            blob_out.cbData as usize / 2,
        );

        let password = String::from_utf16(u16_slice)
            .map_err(|e| format!("UTF-16 解码失败: {}", e))?;

        // 释放内存（不释放，让系统自动管理）
        // Windows DPAPI 会管理这个内存

        Ok(password)
    }
}

/// 非 Windows 平台的降级实现
/// 注意：这只是为了开发测试，生产环境应该使用平台特定的安全存储
#[cfg(not(target_os = "windows"))]
pub fn encrypt_password_dpapi(password: &str) -> Result<String, String> {
    // 非Windows平台使用简单的Base64编码（不安全，仅用于开发）
    eprintln!("警告：非Windows平台，密码仅经过Base64编码，不安全！");
    Ok(general_purpose::STANDARD.encode(password.as_bytes()))
}

#[cfg(not(target_os = "windows"))]
pub fn decrypt_password_dpapi(encrypted_base64: &str) -> Result<String, String> {
    let bytes = general_purpose::STANDARD.decode(encrypted_base64)
        .map_err(|e| format!("Base64 解码失败: {}", e))?;
    String::from_utf8(bytes)
        .map_err(|e| format!("UTF-8 解码失败: {}", e))
}

#[cfg(test)]
mod dpapi_tests {
    use super::*;

    #[test]
    fn test_dpapi_encrypt_decrypt() {
        let password = "test_password_123!@#";
        let encrypted = encrypt_password_dpapi(password).unwrap();
        println!("DPAPI 加密后: {}", encrypted);

        // 验证加密后的内容不是原文
        assert_ne!(encrypted, password);

        // 验证可以解密
        let decrypted = decrypt_password_dpapi(&encrypted).unwrap();
        assert_eq!(password, decrypted);
    }

    #[test]
    fn test_dpapi_chinese_password() {
        let password = "密码测试_中文123";
        let encrypted = encrypt_password_dpapi(password).unwrap();
        let decrypted = decrypt_password_dpapi(&encrypted).unwrap();
        assert_eq!(password, decrypted);
    }
}
