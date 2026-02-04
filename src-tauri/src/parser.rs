use scraper::{Html, Selector, ElementRef};
use regex::Regex;
use std::collections::HashMap;
use crate::models::Course;

/// 课程类型映射（Python: TYPE_MAP）
/// ★: 讲课, ○: 实验, ◆: 讨论, ◇: 上机, ●: 实践
fn map_course_type(symbol: &str) -> &'static str {
    match symbol {
        "★" => "讲课",
        "○" => "实验",
        "◆" => "讨论",
        "◇" => "上机",
        "●" => "实践",
        _ => "未知",
    }
}

/// 循环去除校区前缀（Python: while循环去除校区前缀）
/// 支持多次去除（因为校区前缀可能重复出现）
fn remove_campus_prefix(text: &str) -> String {
    let campus_re = Regex::new(r"^(学院南路校区|沙河校区)\s*").unwrap();
    let mut result = text.to_string();

    // 循环去除，直到无法再去除为止
    loop {
        let new_result = campus_re.replace(&result, "").to_string();
        if new_result == result {
            break;
        }
        result = new_result;
    }

    result.trim().to_string()
}

/// 星期映射表（Python: DAY_MAP）
fn map_day_name(chinese_name: &str) -> Option<&'static str> {
    match chinese_name {
        "星期一" => Some("周一"),
        "星期二" => Some("周二"),
        "星期三" => Some("周三"),
        "星期四" => Some("周四"),
        "星期五" => Some("周五"),
        "星期六" => Some("周六"),
        "星期日" => Some("周日"),
        _ => None,
    }
}

/// 尝试从cell ID推断星期（Python: _try_infer_day_from_cell_id）
/// term1/term3的TD ID格式：X-Y，其中X是星期编号（1=周一，2=周二，...）
fn try_infer_day_from_cell_id(cell_id: &str) -> Option<usize> {
    if !cell_id.contains('-') {
        return None;
    }

    let parts: Vec<&str> = cell_id.split('-').collect();
    if parts.is_empty() {
        return None;
    }

    let first_num = parts[0];
    match first_num.parse::<usize>() {
        Ok(day_num) if (1..=7).contains(&day_num) => Some(day_num),
        _ => None,
    }
}

/// 根据节次号和rowspan计算节次范围（Python: _calculate_period_range）
///
/// # Arguments
/// * `period` - 节次号 (如 "1", "2", "3"...)
/// * `rowspan` - 单元格占的行数（默认2表示占两节课）
///
/// # Returns
/// 节次范围字符串（如 "1-2节", "第3节", "9-11节"）
fn calculate_period_range(period: &str, rowspan: usize) -> String {
    // 节次映射表（Python: PERIOD_MAP）
    // 将节次号映射到起始节次
    let period_map: std::collections::HashMap<&str, i32> = [
        ("1", 1), ("2", 1),
        ("3", 3), ("4", 3),
        ("5", 5), ("6", 5),
        ("7", 7), ("8", 7),
        ("9", 9), ("10", 9),
        ("11", 11), ("12", 11),
    ].iter().cloned().collect();

    // 计算起始节次
    let start_period = period_map.get(period)
        .copied()
        .unwrap_or_else(|| period.parse().unwrap_or(1));

    // 根据rowspan计算结束节次
    let end_period = start_period + rowspan as i32 - 1;

    if start_period == end_period {
        format!("第{}节", start_period)
    } else {
        format!("{}-{}节", start_period, end_period)
    }
}

/// 解析HTML课程表（中央财经大学）
/// 严格按照 C:\project\course_extractor\CUFE\course_extractor.py 的逻辑实现
pub fn parse_course_html(html_content: &str) -> Result<Vec<Course>, String> {
    use std::collections::HashMap;

    println!("=== 开始解析课表 HTML ===");
    println!("HTML 内容长度: {} 字符", html_content.len());

    // 打印 HTML 内容的前 500 个字符用于调试
    let preview = if html_content.len() > 500 {
        &html_content[..500]
    } else {
        html_content
    };
    println!("HTML 内容预览:\n{}", preview);
    println!("========================");

    let document = Html::parse_document(html_content);

    // Python: soup.find('table', class_=re.compile(r'timetable'))
    let mut course_table = None;
    let table_sel = Selector::parse("table").unwrap();

    // 先列出所有找到的 table
    println!("查找所有 table 元素...");
    let mut table_count = 0;
    for table in document.select(&table_sel) {
        table_count += 1;
        if let Some(class_attr) = table.value().attr("class") {
            println!("  Table {}: class='{}'", table_count, class_attr);
            if class_attr.contains("timetable") {
                course_table = Some(table);
                println!("    ✓ 找到课表表格！");
                break;
            }
        } else {
            println!("  Table {}: 无 class 属性", table_count);
        }
    }

    let table = course_table.ok_or("未找到课程表表格 (table with class 'timetable')")?;

    let tr_selector = Selector::parse("tr").unwrap();
    let td_selector = Selector::parse("td").unwrap();
    let div_con_selector = Selector::parse("div.timetable_con").unwrap();

    // Python: rows = table.find_all('tr')
    let rows: Vec<ElementRef> = table.select(&tr_selector).collect();

    // Python: header_row = table.find('tr')
    let header_row = table.select(&tr_selector).next()
        .ok_or("表格没有行")?;

    // Python: 获取表头，建立列索引到星期的映射
    let mut _day_columns: HashMap<usize, &str> = HashMap::new();
    for (idx, td) in header_row.select(&td_selector).enumerate() {
        let text = td.text().collect::<Vec<_>>().join("").trim().to_string();
        if let Some(mapped) = map_day_name(&text) {
            _day_columns.insert(idx, mapped);
        }
    }

    let mut courses = Vec::new();
    let mut current_period: Option<String> = None;
    let mut _time_slot: Option<String> = None;
    let mut col_rowspan: HashMap<usize, usize> = HashMap::new(); // Python: 追踪每列的rowspan剩余行数

    let day_list = ["周一", "周二", "周三", "周四", "周五", "周六", "周日"];

    // Python: 从第3行开始（跳过标题行和表头行）
    // Python: for row_idx, row in enumerate(rows[2:], start=2):
    for row in rows.iter().skip(2) {
        let cells: Vec<ElementRef> = row.select(&td_selector).collect();
        if cells.is_empty() {
            continue;
        }

        // Python: first_cell_text = cells[0].get_text(strip=True)
        let first_cell_text = cells[0].text().collect::<Vec<_>>().join("").trim().to_string();

        // Python: 情况1：第一列是时间段（上午/下午/晚上/中午）
        if ["上午", "下午", "晚上", "中午"].contains(&first_cell_text.as_str()) {
            _time_slot = Some(first_cell_text.clone());

            // Python: if len(cells) >= 2:
            if cells.len() >= 2 {
                let period_text = cells[1].text().collect::<Vec<_>>().join("").trim().to_string();
                // Python: if period_text.isdigit():
                if period_text.chars().all(char::is_numeric) {
                    current_period = Some(period_text);
                }
            }

            // Python: 从第3列开始遍历（跳过时间段和节次列）
            // Python: day_idx = 0
            let mut day_idx = 0;

            // Python: for cell in cells[2:]:
            for cell in cells.iter().skip(2) {
                // Python: 跳过被rowspan占据的列
                // Python: while day_idx in col_rowspan and col_rowspan[day_idx] > 0:
                while col_rowspan.contains_key(&day_idx) {
                    let count = col_rowspan.get(&day_idx).copied().unwrap_or(0);
                    if count > 0 {
                        let new_count = count - 1;
                        if new_count > 0 {
                            col_rowspan.insert(day_idx, new_count);
                        } else {
                            col_rowspan.remove(&day_idx);
                        }
                        day_idx += 1;
                    } else {
                        // count == 0，移除并继续到下一个索引
                        col_rowspan.remove(&day_idx);
                        day_idx += 1;
                    }
                }

                // Python: if day_idx < len(day_list):
                if day_idx < day_list.len() {
                    let day = day_list[day_idx];

                    // Python: 检查并记录rowspan
                    // Python: cell_rowspan = cell.get('rowspan')
                    let cell_rowspan = cell.value().attr("rowspan")
                        .and_then(|s| s.parse::<usize>().ok())
                        .unwrap_or(1);

                    // Python: if cell_rowspan > 1:
                    // Python:     col_rowspan[day_idx] = cell_rowspan - 1
                    if cell_rowspan > 1 {
                        col_rowspan.insert(day_idx, cell_rowspan - 1);
                    }

                    // Python: 处理同一单元格中的多个课程块
                    // Python: course_divs = cell.find_all('div', class_='timetable_con')
                    for course_div in cell.select(&div_con_selector) {
                        // Python: course = self._parse_course_cell(course_div, day, current_period, cell_rowspan)
                        if let Some(period_str) = &current_period {
                            if let Some(course) = parse_course_cell(&course_div, day, period_str, cell_rowspan) {
                                courses.push(course);
                            }
                        }
                    }
                }

                // Python: day_idx += 1
                day_idx += 1;
            }
        }
        // Python: 情况2：第一列是节次号（rowspan导致的跨行）
        // Python: elif first_cell_text.isdigit():
        else if first_cell_text.chars().all(char::is_numeric) {
            current_period = Some(first_cell_text.clone());

            // Python: 尝试从cell ID推断星期（适用于term1/term3格式）
            // Python: day_idx = 0
            let mut day_idx = 0;

            // Python: for cell in cells[1:]:  # 从列1开始
            for cell in cells.iter().skip(1) {
                // Python: cell_id = cell.get('id', '')
                let cell_id = cell.value().attr("id").unwrap_or("");

                // Python: 首先检查并跳过被col_rowspan占据的列
                // Python: while day_idx in col_rowspan and col_rowspan[day_idx] > 0:
                while col_rowspan.contains_key(&day_idx) {
                    let count = col_rowspan.get(&day_idx).copied().unwrap_or(0);
                    if count > 0 {
                        let new_count = count - 1;
                        if new_count > 0 {
                            col_rowspan.insert(day_idx, new_count);
                        } else {
                            col_rowspan.remove(&day_idx);
                        }
                        day_idx += 1;
                    } else {
                        // count == 0，移除并继续到下一个索引
                        col_rowspan.remove(&day_idx);
                        day_idx += 1;
                    }
                }

                // Python: 获取rowspan（所有单元格都需要）
                let cell_rowspan = cell.value().attr("rowspan")
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(1);

                // Python: 尝试从cell ID推断星期
                // Python: inferred_day = self._try_infer_day_from_cell_id(cell_id, day_list)
                let day = if let Some(day_num) = try_infer_day_from_cell_id(cell_id) {
                    // day_num 是1-7，转换为索引0-6
                    day_list.get(day_num - 1).copied().unwrap_or("周一")
                } else {
                    // Python: fallback: 使用day_idx
                    // Python: if day_idx < len(day_list):
                    if day_idx < day_list.len() {
                        day_list[day_idx]
                    } else {
                        day_idx += 1;
                        continue;
                    }
                };

                // Python: 记录rowspan到col_rowspan（用于后续行跳过）
                // Python: if cell_rowspan > 1:
                // Python:     col_rowspan[day_idx] = cell_rowspan - 1
                if cell_rowspan > 1 {
                    col_rowspan.insert(day_idx, cell_rowspan - 1);
                }

                // Python: 处理同一单元格中的多个课程块
                for course_div in cell.select(&div_con_selector) {
                    if let Some(period_str) = &current_period {
                        if let Some(course) = parse_course_cell(&course_div, day, period_str, cell_rowspan) {
                            courses.push(course);
                        }
                    }
                }

                // Python: day_idx += 1
                day_idx += 1;
            }
        }
        // Python: 其他情况：跳过该行
    }

    Ok(courses)
}

/// 解析单个课程单元格（Python: _parse_course_cell）
/// 严格按照 C:\project\course_extractor\CUFE\course_extractor.py 的逻辑实现
fn parse_course_cell(
    div: &ElementRef,
    day: &str,
    period: &str,
    rowspan: usize,
) -> Option<Course> {
    let span_title_selector = Selector::parse("span.title, u.title").unwrap();
    let p_selector = Selector::parse("p").unwrap();

    // Python: title_elem = div.find(['span', 'u'], class_='title')
    // Python: if not title_elem: return None
    let title_elem = div.select(&span_title_selector).next()?;

    // Python: title_text = title_elem.get_text(strip=True)
    let mut title_text = title_elem.text().collect::<Vec<_>>().join("").trim().to_string();

    // Python: title_text = title_text.replace('【调】', '').strip()
    title_text = title_text.replace("【调】", "").trim().to_string();

    // Python: 尝试从标题中提取节次信息（优先级高于rowspan计算）
    // Python: title_period_match = re.search(r'\((\d+)-(\d+)节\)', title_text)
    let title_period_re = Regex::new(r"\((\d+)-(\d+)节\)").unwrap();
    let mut period_range = String::new();

    if let Some(cap) = title_period_re.captures(&title_text) {
        // Python: start = title_period_match.group(1)
        // Python: end = title_period_match.group(2)
        if let (Some(start), Some(end)) = (cap.get(1), cap.get(2)) {
            let start_str = start.as_str();
            let end_str = end.as_str();
            // Python: 如果节次范围是X-X，显示为"第X节"
            if start_str == end_str {
                period_range = format!("第{}节", start_str);
            } else {
                period_range = format!("{}-{}节", start_str, end_str);
            }
            // Python: 从标题中移除节次信息
            // Python: title_text = re.sub(r'\s*\(\d+-\d+节\)', '', title_text)
            let title_period_remove_re = Regex::new(r"\s*\(\d+-\d+节\)").unwrap();
            title_text = title_period_remove_re.replace(&title_text, "").trim().to_string();
        }
    } else {
        // Python: 如果标题中没有节次信息，使用rowspan计算
        // Python: period_range = self._calculate_period_range(period, rowspan)
        period_range = calculate_period_range(period, rowspan);
    }

    // Python: course_type_symbol = None
    // Python: for symbol in self.TYPE_MAP.keys():
    let type_symbols = ['★', '○', '◆', '◇', '●'];
    let mut course_type_symbol = None;

    for symbol in &type_symbols {
        // Python: if symbol in title_text:
        if title_text.contains(*symbol) {
            course_type_symbol = Some(*symbol);
            break;
        }
    }

    // Python: course_name = title_text.rstrip(''.join(self.TYPE_MAP.keys())).strip()
    let symbols_str: String = type_symbols.iter().collect();
    let course_name = title_text.trim_end_matches(|c| symbols_str.contains(c)).trim().to_string();

    // Python: 提取详细信息
    // Python: all_p = div.find_all('p')
    // Python: weeks = classroom = teacher = class_name = credits = total_hours = exam_type = None
    let mut weeks_vec: Vec<i32> = Vec::new();
    let mut classroom: Option<String> = None;
    let mut teacher: Option<String> = None;
    let mut period_from_p: Option<String> = None;

    // Python: 第一个循环：提取节次、周次、教室、教师
    // Python: for p in all_p:
    for p in div.select(&p_selector) {
        // Python: text = p.get_text(strip=True)
        let text = p.text().collect::<Vec<_>>().join("").trim().to_string();

        // Python: span = p.find('span')
        // Python: title_attr = span.get('title', '') if span else p.get('title', '')
        let span_sel = Selector::parse("span").unwrap();
        let title_attr = if let Some(span) = p.select(&span_sel).next() {
            span.value().attr("title").unwrap_or("").to_string()
        } else {
            p.value().attr("title").unwrap_or("").to_string()
        };

        // Python: if '节' in text and '周' in text:
        if text.contains("节") && text.contains("周") {
            // Python: 从p标签中提取节次信息
            // Python: period_match = re.search(r'\((\d+)-(\d+)节\)', text)
            let period_re = Regex::new(r"\((\d+)-(\d+)节\)").unwrap();
            // Python: if period_match and not period_from_p:
            if period_from_p.is_none() {
                if let Some(cap) = period_re.captures(&text) {
                    if let (Some(start), Some(end)) = (cap.get(1), cap.get(2)) {
                        let start_str = start.as_str();
                        let end_str = end.as_str();
                        if start_str == end_str {
                            period_from_p = Some(format!("第{}节", start_str));
                        } else {
                            period_from_p = Some(format!("{}-{}节", start_str, end_str));
                        }
                    }
                }
            }

            // Python: 提取周次信息
            // Python: week_matches = re.findall(r'(\d+(?:-\d+)?\s*)周', text)
            // Python: if week_matches: weeks = '、'.join(week_matches) + '周'
            let week_re = Regex::new(r"(\d+(?:-\d+)?)\s*周").unwrap();
            for cap in week_re.captures_iter(&text) {
                if let Some(m) = cap.get(1) {
                    let range_str = m.as_str();
                    if range_str.contains('-') {
                        let parts: Vec<&str> = range_str.split('-').collect();
                        if parts.len() == 2 {
                            if let (Ok(start), Ok(end)) = (parts[0].parse::<i32>(), parts[1].parse::<i32>()) {
                                if start <= end {
                                    for w in start..=end {
                                        weeks_vec.push(w);
                                    }
                                }
                            }
                        }
                    } else if let Ok(w) = range_str.parse::<i32>() {
                        weeks_vec.push(w);
                    }
                }
            }
        }
        // Python: elif '上课地点' in title_attr or '主教' in text or '校区' in text or '楼' in text:
        else if title_attr.contains("上课地点") || text.contains("主教") || text.contains("校区") || text.contains("楼") {
            // Python: classroom = text
            // Python: 循环去除所有校区前缀
            classroom = Some(remove_campus_prefix(&text));
        }
        // Python: elif '教师' in title_attr:
        else if title_attr.contains("教师") {
            // Python: teacher = text.strip()
            teacher = Some(text.trim().to_string());
        }
    }

    // Python: 如果从p标签中提取到了节次信息，使用它（优先级高于rowspan计算）
    // Python: if period_from_p: period_range = period_from_p
    if let Some(p_period) = period_from_p {
        period_range = p_period;
    }

    // 将节次范围字符串转换为 Vec<i32>
    let periods = parse_period_range_string(&period_range);

    // 处理周次向量并排序去重
    weeks_vec.sort();
    weeks_vec.dedup();

    // 解析星期 str -> i32
    let day_of_week = match day {
        "周一" => 1,
        "周二" => 2,
        "周三" => 3,
        "周四" => 4,
        "周五" => 5,
        "周六" => 6,
        "周日" => 7,
        _ => 1,
    };

    let week_type = 0;

    Some(Course {
        name: course_name,
        teacher: teacher.unwrap_or_else(|| "未指定".to_string()),
        weeks: weeks_vec,
        week_type,
        day_of_week,
        periods,
        location: classroom.unwrap_or_else(|| "未指定".to_string()),
    })
}

/// 将节次范围字符串解析为Vec<i32>
/// 支持格式："1-2节", "第3节", "9-11节" 等
fn parse_period_range_string(period_str: &str) -> Vec<i32> {
    // 移除"节"字和"第"字
    let clean_str = period_str.replace("节", "").replace("第", "").trim().to_string();

    if clean_str.contains('-') {
        // 格式如 "1-2"
        let parts: Vec<&str> = clean_str.split('-').collect();
        if parts.len() == 2 {
            if let (Ok(start), Ok(end)) = (parts[0].parse::<i32>(), parts[1].parse::<i32>()) {
                if start <= end {
                    return (start..=end).collect();
                }
            }
        }
    }

    // 单个节次
    if let Ok(p) = clean_str.parse::<i32>() {
        return vec![p];
    }

    // Fallback: 尝试旧的映射逻辑（兼容性）
    match period_str {
        "1" | "2" => vec![1, 2],
        "3" | "4" => vec![3, 4],
        "5" | "6" => vec![5, 6],
        "7" | "8" => vec![7, 8],
        "9" | "10" => vec![9, 10],
        "11" | "12" => vec![11, 12],
        _ => vec![],
    }
}

/// 解析浙江大学课程表 HTML
/// 严格按照 C:\project\zjucourses\extract_schedule.py 的逻辑实现
pub fn parse_zju_html(html_content: &str) -> Result<Vec<Course>, String> {
    let document = Html::parse_document(html_content);

    // 查找课表表格 ID="kbgrid_table"
    let table_sel = Selector::parse("table#kbgrid_table").unwrap();
    let table = document
        .select(&table_sel)
        .next()
        .ok_or("未找到浙江大学课表表格 (table#kbgrid_table)")?;

    let td_selector = Selector::parse("td[id]").unwrap();
    let a_selector = Selector::parse("a[onclick*='showCourseInfo2']").unwrap();
    let font_selector = Selector::parse("font[color='blue']").unwrap();

    // 存储原始课程记录（包含单双周）
    let mut raw_courses: Vec<RawZJUCourse> = Vec::new();
    let mut processed_cells: HashMap<String, bool> = HashMap::new();

    // 遍历所有包含课程的单元格
    for td in table.select(&td_selector) {
        let cell_id = td.value().attr("id").unwrap_or("");

        // 跳过已处理的单元格
        if processed_cells.contains_key(cell_id) {
            continue;
        }

        // 解析单元格 ID：格式为 {星期}-{单双周}-{节次}
        // 例如：2-1-1 表示 周二-双周-第1节
        let parts: Vec<&str> = cell_id.split('-').collect();
        if parts.len() != 3 {
            continue;
        }

        let weekday: i32 = parts[0].parse().unwrap_or(0);
        let parity_code: i32 = parts[1].parse().unwrap_or(0);
        let period: i32 = parts[2].parse().unwrap_or(0);

        if weekday == 0 || period == 0 {
            continue;
        }

        // 0=单周, 1=双周
        let parity_str = if parity_code == 0 { "单" } else { "双" };

        // 查找课程链接
        if let Some(course_link) = td.select(&a_selector).next() {
            if let Some(font_tag) = course_link.select(&font_selector).next() {
                // 获取课程信息文本
                let text_lines: Vec<String> = font_tag
                    .text()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();

                if text_lines.len() < 2 {
                    continue;
                }

                let course_name = text_lines[0].clone();
                let week_info = &text_lines[1];
                let teacher = if text_lines.len() > 2 {
                    text_lines[2].clone()
                } else {
                    "未指定".to_string()
                };
                let location = if text_lines.len() > 3 {
                    text_lines[3].clone()
                } else {
                    "未指定".to_string()
                };

                // 解析周次信息，如 "秋冬{第1-8周|2节/双周}"
                let (weeks_str, frequency) = parse_zju_week_info(week_info);

                // 检查 colspan 和 rowspan
                let colspan = td
                    .value()
                    .attr("colspan")
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(1);
                let rowspan = td
                    .value()
                    .attr("rowspan")
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(1);

                // 【关键】确定需要生成的单双周列表（严格按照 Python 逻辑）
                // Python: if colspan == 2 and '/周' in frequency and '/双周' not in frequency:
                let parity_list: Vec<&str> =
                    if colspan == 2 && frequency.contains("节/周") && !frequency.contains("双周") {
                        // 每周都上，生成单周和双周两条记录
                        vec!["单", "双"]
                    } else {
                        // 只按实际单双周生成
                        vec![parity_str]
                    };

                // 确定节次范围（处理 rowspan）
                let mut period_list: Vec<i32> = Vec::new();
                for r in 0..rowspan {
                    period_list.push(period + r as i32);
                }

                // 为每个(单双周, 节次)组合生成记录（Python: for p in parity_list: for per in period_list）
                for p in &parity_list {
                    for per in &period_list {
                        raw_courses.push(RawZJUCourse {
                            course_name: course_name.clone(),
                            weekday,
                            period: *per,
                            weeks: weeks_str.clone(),
                            parity: p.to_string(),
                            _frequency: frequency.clone(),
                            teacher: teacher.clone(),
                            location: location.clone(),
                        });
                    }
                }

                // 标记已处理的单元格
                processed_cells.insert(cell_id.to_string(), true);

                // 标记被合并的单元格（Python: merged_id = f"{base_parts[0]}-{int(base_parts[1]) + c}-{int(base_parts[2]) + r}"）
                if rowspan > 1 || colspan > 1 {
                    let base_parts: Vec<&str> = cell_id.split('-').collect();
                    if let (Ok(w), Ok(p_code)) = (
                        base_parts[0].parse::<i32>(),
                        base_parts[1].parse::<i32>(),
                    ) {
                        for r in 0..rowspan {
                            for c in 0..colspan {
                                if r == 0 && c == 0 {
                                    continue;
                                }
                                let merged_id = format!("{}-{}-{}", w, p_code + c as i32, period + r as i32);
                                processed_cells.insert(merged_id, true);
                            }
                        }
                    }
                }
            }
        }
    }

    // 合并记录并格式化（Python: merge_and_format）
    let merged_courses = merge_and_format_zju(raw_courses)?;

    Ok(merged_courses)
}

/// 浙江大学原始课程记录（对应 Python 脚本中的提取阶段）
#[derive(Debug, Clone)]
struct RawZJUCourse {
    course_name: String,
    weekday: i32,
    period: i32,
    weeks: String,
    parity: String,
    _frequency: String, // 保留用于逻辑判断，但不直接使用
    teacher: String,
    location: String,
}

/// 合并并格式化浙江大学课程（Python: merge_and_format）
fn merge_and_format_zju(raw_courses: Vec<RawZJUCourse>) -> Result<Vec<Course>, String> {
    // 第一步：按课程名称、星期、地点、教师分组（不含节次）
    // Python: key = (course['course_name'], course['weekday'], course['location'], course['teacher'])
    let mut course_groups: HashMap<String, Vec<&RawZJUCourse>> = HashMap::new();

    for course in &raw_courses {
        let key = format!(
            "{}|{}|{}|{}",
            course.course_name, course.weekday, course.location, course.teacher
        );

        course_groups
            .entry(key)
            .or_insert_with(Vec::new)
            .push(course);
    }

    // 第二步：对每个课程组进行处理
    let mut result = Vec::new();
    for (_key, courses) in course_groups.iter() {
        // 展开周数并合并（Python: groups[key]['weeks_list'].extend(WeekExpander.expand_weeks(...))）
        let mut weeks_list: Vec<i32> = Vec::new();
        for course in courses {
            let expanded = expand_zju_weeks(&course.weeks, &course.parity);
            weeks_list.extend(expanded);
        }
        weeks_list.sort();
        weeks_list.dedup();

        // 计算 week_type（如果周数是连续的则设为0=全周）
        let week_type = if weeks_list.len() > 1 {
            let is_continuous = weeks_list.windows(2).all(|w| w[1] == w[0] + 1);
            if is_continuous {
                0 // 全周
            } else {
                // 根据第一个课程的 parity 判断
                if courses[0].parity == "单" {
                    1
                } else {
                    2
                }
            }
        } else {
            0
        };

        // 【关键】查找相同课程的所有节次（Python: all_periods = [...]）
        let mut all_periods: Vec<i32> = courses.iter().map(|c| c.period).collect();
        all_periods.sort();
        all_periods.dedup();

        // 计算节次范围（Python: period_range）
        // Python: if len(all_periods) == 1: period_range = f"{all_periods[0]}-{all_periods[0] + 1}节"
        //       else: period_range = f"{all_periods[0]}-{all_periods[-1]}节"
        let periods = if all_periods.len() == 1 {
            // 单个节次，一节课通常是连续两小节
            vec![all_periods[0], all_periods[0] + 1]
        } else {
            // 多个节次，直接取最小到最大
            (all_periods[0]..=all_periods[all_periods.len() - 1]).collect()
        };

        let course = Course {
            name: courses[0].course_name.clone(),
            teacher: courses[0].teacher.clone(),
            weeks: weeks_list,
            week_type,
            day_of_week: courses[0].weekday,
            periods,
            location: courses[0].location.clone(),
        };
        result.push(course);
    }

    // 去重（Python: unique_result）
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut unique_result = Vec::new();
    for course in result {
        let key = format!(
            "{}|{}|{:?}|{}",
            course.day_of_week, course.name, course.periods, course.location
        );
        if !seen.contains(&key) {
            seen.insert(key);
            unique_result.push(course);
        }
    }

    Ok(unique_result)
}

/// 解析浙江大学周次信息
/// 输入如："秋冬{第1-8周|2节/双周}"
/// 返回：("第1-8周", "2节/双周")
fn parse_zju_week_info(text: &str) -> (String, String) {
    let week_re = Regex::new(r"第(\d+)-?(\d*)周").unwrap();
    let freq_re = Regex::new(r"(\d+)节/(周|双周)").unwrap();

    let weeks = week_re
        .captures(text)
        .map(|cap| {
            let start = cap.get(1).map(|m| m.as_str()).unwrap_or("1");
            let end = cap.get(2).map(|m| m.as_str()).filter(|s| !s.is_empty());
            match end {
                Some(e) => format!("第{}-{}周", start, e),
                None => format!("第{}周", start),
            }
        })
        .unwrap_or_default();

    let frequency = freq_re
        .captures(text)
        .map(|cap| {
            let count = cap.get(1).map(|m| m.as_str()).unwrap_or("2");
            let unit = cap.get(2).map(|m| m.as_str()).unwrap_or("周");
            format!("{}节/{}", count, unit)
        })
        .unwrap_or_default();

    (weeks, frequency)
}

/// 展开浙江大学周次字符串
/// 输入如："第1-8周"，parity="单"
/// 返回：[1, 3, 5, 7]
fn expand_zju_weeks(weeks_str: &str, parity: &str) -> Vec<i32> {
    let clean_str = weeks_str.replace("第", "").replace("周", "").trim().to_string();

    let (start, end) = if clean_str.contains('-') {
        let parts: Vec<&str> = clean_str.split('-').collect();
        let start_val = parts[0].parse().unwrap_or(1);
        let end_val = if parts.len() > 1 {
            parts[1].parse().unwrap_or(start_val)
        } else {
            start_val
        };
        (start_val, end_val)
    } else {
        let val = clean_str.parse().unwrap_or(1);
        (val, val)
    };

    let mut weeks = Vec::new();

    if parity == "单" {
        // 单周：1, 3, 5, 7...
        let mut w = if start % 2 == 1 { start } else { start + 1 };
        while w <= end {
            weeks.push(w);
            w += 2;
        }
    } else if parity == "双" {
        // 双周：2, 4, 6, 8...
        let mut w = if start % 2 == 0 { start } else { start + 1 };
        while w <= end {
            weeks.push(w);
            w += 2;
        }
    } else {
        // 全周
        for w in start..=end {
            weeks.push(w);
        }
    }

    weeks
}

/// 根据解析器类型解析 HTML
pub fn parse_html_with_parser(html_content: &str, parser_type: &str) -> Result<Vec<Course>, String> {
    match parser_type {
        "zju_default" => parse_zju_html(html_content),
        _ => parse_course_html(html_content), // 默认使用中央财经大学解析器
    }
}

/// 解析 CUFE JSON 格式的课表数据
/// CUFE 教务系统返回的是 JSON 而不是 HTML
pub fn parse_cufe_json(json_text: &str) -> Result<Vec<Course>, String> {
    use serde_json::Value;

    println!("=== 开始解析 CUFE JSON 课表 ===");

    // 解析 JSON
    let json: Value = serde_json::from_str(json_text)
        .map_err(|e| format!("解析JSON失败: {}", e))?;

    // 提取 kbList 数组
    let kb_list = json.get("kbList")
        .and_then(|v| v.as_array())
        .ok_or("JSON中未找到kbList字段")?;

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
