
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
pub fn decrypt_password_dpapi(_encrypted_base64: &str) -> Result<String, String> {
    Err("非 Windows 平台不支持 DPAPI 解密".to_string())
}
