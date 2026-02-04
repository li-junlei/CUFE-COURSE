// CUFE 教务系统仅支持 JSON 格式，不再支持 HTML 解析
// 请使用 parse_cufe_json() 解析课表数据

use crate::models::Course;

/// 解析 HTML 格式的课表数据（已废弃）
/// CUFE 教务系统现在只返回 JSON，不再支持 HTML 解析
/// 此函数保留是为了向后兼容，但会返回错误提示
pub fn parse_html_with_parser(_html_content: &str, _parser_type: &str) -> Result<Vec<crate::models::Course>, String> {
    Err("CUFE 教务系统仅支持 JSON 格式解析。请使用「从服务器导入」功能，不要使用「从浏览器导入」".to_string())
}

/// 解析 CUFE JSON 格式的课表数据
/// CUFE 教务系统返回的是 JSON 而不是 HTML
pub fn parse_cufe_json(json_text: &str) -> Result<Vec<Course>, String> {
    use serde_json::Value;

    println!("=== 开始解析 CUFE JSON 课表 ===");
    println!("原始响应长度: {} 字节", json_text.len());

    // 打印前200个字符用于调试
    let preview = if json_text.len() > 200 {
        &json_text[..200]
    } else {
        json_text
    };
    println!("原始响应预览:\n{}", preview);

    // 尝试移除BOM和其他可能的干扰字符
    let cleaned_text = json_text.trim().trim_start_matches('\u{feff}').trim_start_matches('\u{200b}');

    // 检查是否是HTML响应（错误页面）
    if cleaned_text.starts_with("<!DOCTYPE") || cleaned_text.starts_with("<html") || cleaned_text.starts_with("<HTML") {
        return Err("服务器返回了HTML页面而不是JSON，可能是登录已失效".to_string());
    }

    // 解析 JSON
    let json: Value = serde_json::from_str(cleaned_text)
        .map_err(|e| {
            // 提供更详细的错误信息
            let error_preview = if cleaned_text.len() > 100 {
                &cleaned_text[..100]
            } else {
                cleaned_text
            };
            format!("解析JSON失败: {}\n实际内容前100字符: {}", e, error_preview)
        })?;

    // 提取 kbList 数组
    let kb_list = json.get("kbList")
        .and_then(|v| v.as_array())
        .ok_or("JSON中未找到kbList字段，可能是API返回格式已变更")?;

    println!("找到 {} 条课程记录", kb_list.len());

    let mut courses = Vec::new();

    for item in kb_list {
        // 提取课程基本信息
        let course_name = item.get("kcmc")
            .and_then(|v| v.as_str())
            .unwrap_or("未知课程")
            .to_string();

        let teacher = item.get("xm")
            .and_then(|v| v.as_str())
            .unwrap_or("未指定")
            .to_string();

        let classroom = item.get("cdmc")
            .and_then(|v| v.as_str())
            .unwrap_or("未指定")
            .to_string();

        // 提取星期信息
        let xqjmc = item.get("xqjmc")
            .and_then(|v| v.as_str())
            .unwrap_or("星期一");

        let day_of_week = match xqjmc {
            "星期一" => 1,
            "星期二" => 2,
            "星期三" => 3,
            "星期四" => 4,
            "星期五" => 5,
            "星期六" => 6,
            "星期日" => 7,
            _ => 1,
        };

        // 提取节次信息 (如 "3-4节")
        let jc = item.get("jc")
            .and_then(|v| v.as_str())
            .unwrap_or("1-2节");

        let periods = parse_period_string(jc);

        // 提取周次信息 (如 "4-5周,7-18周")
        let zcd = item.get("zcd")
            .and_then(|v| v.as_str())
            .unwrap_or("1-18周");

        let weeks = parse_week_string(zcd);

        // 检查是否有课程类型符号 (xslxbj: "★", "○" 等)
        // 如果没有，默认为讲课类型
        let _course_type_sym = item.get("xslxbj")
            .and_then(|v| v.as_str())
            .unwrap_or("★");

        // 从课程名称中移除课程类型符号
        let course_name_clean = course_name.trim_end_matches(|c: char| {
            matches!(c, '★' | '○' | '◆' | '◇' | '●')
        }).trim().to_string();

        println!("解析课程: {} - {} - {} - {}", course_name_clean, xqjmc, jc, zcd);

        courses.push(Course {
            name: course_name_clean,
            teacher,
            weeks,
            week_type: 0, // 默认为全周
            day_of_week,
            periods,
            location: classroom,
        });
    }

    println!("成功解析 {} 条课程记录", courses.len());
    Ok(courses)
}

/// 解析节次字符串 (如 "3-4节", "9-11节")
fn parse_period_string(period_str: &str) -> Vec<i32> {
    // 移除"节"字
    let clean = period_str.replace("节", "").trim().to_string();

    if clean.contains('-') {
        // 格式如 "3-4"
        let parts: Vec<&str> = clean.split('-').collect();
        if parts.len() == 2 {
            if let (Ok(start), Ok(end)) = (parts[0].parse::<i32>(), parts[1].parse::<i32>()) {
                if start <= end {
                    return (start..=end).collect();
                }
            }
        }
    }

    // 单个节次或解析失败，返回默认值
    if let Ok(p) = clean.parse::<i32>() {
        vec![p]
    } else {
        vec![1, 2] // 默认第1-2节
    }
}

/// 解析周次字符串 (如 "4-5周,7-18周", "3-18周", "2周,6周")
fn parse_week_string(week_str: &str) -> Vec<i32> {
    let mut weeks = Vec::new();

    // 移除"周"字，然后按逗号分割
    let clean = week_str.replace("周", "");
    let parts: Vec<&str> = clean.split(',').collect();

    for part in parts {
        let part = part.trim();
        if part.contains('-') {
            // 范围，如 "4-5"
            let range: Vec<&str> = part.split('-').collect();
            if range.len() == 2 {
                if let (Ok(start), Ok(end)) = (range[0].parse::<i32>(), range[1].parse::<i32>()) {
                    if start <= end {
                        for w in start..=end {
                            weeks.push(w);
                        }
                    }
                }
            }
        } else {
            // 单个周次，如 "2"
            if let Ok(w) = part.parse::<i32>() {
                weeks.push(w);
            }
        }
    }

    // 去重并排序
    weeks.sort();
    weeks.dedup();

    weeks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_course_html() {
        let html = r#"
        <!DOCTYPE html>
        <html>
        <body>
            <table class="timetable">
                <thead>
                    <tr><th colspan="9">Course Schedule</th></tr>
                    <tr><th>Section</th><th>Time</th><th>Mon</th><th>Tue</th><th>Wed</th><th>Thu</th><th>Fri</th><th>Sat</th><th>Sun</th></tr>
                </thead>
                <tbody>
                    <tr>
                        <td rowspan="4">上午</td>
                        <td>1</td>
                        <td>
                            <div class="timetable_con">
                                <span class="title">高等数学★</span>
                                <p title="教师">张三</p>
                                <p>1-16周</p>
                                <p title="上课地点">主教101</p>
                            </div>
                        </td>
                        <td>
                            <div class="timetable_con">
                                <span class="title">Test Course</span>
                                <p title="教师">Smith</p>
                                <p>2-4周</p>
                                <p title="上课地点">Room 202</p>
                            </div>
                        </td>
                        <td></td><td></td><td></td><td></td><td></td>
                    </tr>
                </tbody>
            </table>
        </body>
        </html>
        "#;

        let courses = parse_course_html(html).expect("Failed to parse");
        println!("Extracted courses: {:?}", courses);
        // Note: The parser logic for "Mon"/"Tue" depends on specific Chinese strings "周一", "周二" etc in header or assumed order.
        // My parser code: `let day_list = vec!["周一", "周二", ...];`
        // And it finds period columns based on skipping first 1 or 2 cols.
        // Wait, the parser logic iterates `cells` starting from index.
        // It assumes 3rd row is data.
        // And it maps columns to days using `col_rowspan` and `day_idx`.
        // BUT `day_list` is just a list of names to assign.
        // The parser logic does NOT look at the Table Header to determine which column is Monday.
        // It assumes standard layout: Period Col -> Mon -> Tue -> ...
        
        // In my mock HTML above, I have `<td>1</td>` (Period) then `<td>...` (Mon).
        // Let's verify parser logic:
        // `is_time_slot` check: "上午" is in first cell? Yes.
        // `start_cell_idx = 2`. Correct.
        // `cells.skip(2)` -> Mon, Tue...
        
        // So first course (Mon) should be parsed.
        // Second course (Tue) should be parsed.
        
        assert!(courses.len() >= 1);
        let c1 = &courses[0];
        assert_eq!(c1.name, "高等数学");
        assert_eq!(c1.day_of_week, 1); // Mon
        
        if courses.len() > 1 {
            let c2 = &courses[1];
            assert_eq!(c2.name, "Test Course");
            assert_eq!(c2.day_of_week, 2); // Tue
        }
    }
}
