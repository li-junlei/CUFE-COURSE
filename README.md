# 课程表桌面应用 (Course Schedule Desktop)

基于 Tauri 2.0 + Vue 3 + TypeScript 构建的跨平台课程表管理桌面应用

[![Version](https://img.shields.io/badge/version-1.7.0-blue.svg)](https://github.com/lijunlei/course-schedule-desktop)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-2.0-FFC131.svg)](https://tauri.app/)
[![Vue](https://img.shields.io/badge/Vue-3.5-4FC08D.svg)](https://vuejs.org/)

---

## 版本历史

### v1.7.0 (2026-02-02)
- ✨ 新增课表重命名功能，支持在课表编辑对话框中直接修改课表名称
- 🐛 修复中央财经大学解析器遗漏课程的问题（严格按照 Python 参考实现重写）
- 🔧 优化 Tauri 命令注册，确保所有前端调用正确路由到后端

### v1.6.0 (2026-02-02)
- 多教务系统支持与时间表管理重构
- 初始化项目架构文档

### v1.5.0 (2026-02-02)
- 多教务系统支持与时间表管理重构

---

## 功能特性

### 核心功能
- 多课表管理 - 创建、切换、重命名、删除多个课表，支持拖拽排序
- 智能导入 - 内置浏览器一键导入，剪贴板自动传输
- 自定义时间表 - 支持多套节次时间配置，灵活适配不同学校
- 动态周数 - 周次选择器根据课表设置动态显示（支持 10-30 周灵活配置）

### 界面特性
- 周次预览 - 点阵视图展示每周课程分布
- 滑动切换 - 手势滑动切换周次，自动适配周数范围
- 课程详情 - 底部抽屉展示完整课程信息
- 深色模式 - 支持手动切换和跟随系统主题

### 外观定制
- 网格辅助线 - 可开关的虚线网格，无上边和左边框
- 卡片透明度 - 可调节课程卡片不透明度（50-100%）
- 信息显示控制 - 自定义显示教师/地点信息
- 自定义背景 - 支持上传本地图片作为课表背景

### 智能提示
- 学期状态 - 自动显示"未开学"/"学期已结束"状态
- 今日高亮 - 强调显示当前日期，快速定位
- 本地时间 - 基于本地时间计算，准确无误

---

## 快速开始

### 环境要求

- Node.js 18+
- Rust 1.70+ (含 Cargo)
- npm 或 yarn

### 安装步骤

1. 克隆仓库
   ```bash
   git clone https://github.com/lijunlei/course-schedule-desktop.git
   cd course-schedule-desktop
   ```

2. 安装依赖
   ```bash
   npm install
   ```

3. 开发模式
   ```bash
   npm run tauri dev
   ```

4. 构建应用
   ```bash
   npm run tauri build
   ```

### 获取安装包

从 [Releases](https://github.com/lijunlei/course-schedule-desktop/releases) 页面下载最新版本的安装包

---

## 使用指南

### 导入课表

1. 点击左上角菜单按钮
2. 选择"导入课表"
3. 在打开的浏览器中登录教务系统
4. 导航到课表页面
5. 点击右下角"导入当前课表"浮动按钮
6. 关闭浏览器窗口，应用自动导入

### 管理多个课表

- 创建: 通过"导入课表"创建新课表
- 切换: 在"课表管理"中点击课表卡片切换
- 重命名: 点击编辑图标，在"课表信息"中修改课表名称
- 排序: 拖拽课表卡片调整顺序
- 编辑: 点击编辑图标修改学期设置和时间表方案
- 删除: 点击删除图标移除不需要的课表

### 自定义时间表

1. 打开"课表管理" -> 编辑课表
2. 在"时间表"部分点击"编辑"
3. 修改每节课的开始/结束时间
4. 可创建多个时间表并应用到不同课表
5. 点击"应用到所有课表"统一时间配置

### 修改学期设置

1. 打开"课表管理" -> 点击要编辑的课表卡片上的编辑图标
2. 在"课表信息"中修改课表名称
3. 在"学期设置"中配置：
   - **第一周第一天**：设置学期开始日期
   - **每天节数**：设置每天的课程节数（4-20节）
   - **学期周数**：设置学期总周数（10-30周）
4. 保存后，周次选择器会自动显示正确的周数范围

### 外观设置

- 网格辅助线: 在"设置" -> "课表外观设置"中开启
- 卡片透明度: 拖动滑块调整（50-100%）
- 信息显示: 开关"显示授课教师"或"显示上课地点"
- 背景图片: 点击菜单 -> "上传背景"选择本地图片

---

## 技术栈

### 前端技术
- 框架: Vue 3 (Composition API + `<script setup>`)
- 类型系统: TypeScript 5.6
- 构建工具: Vite 6.0
- UI 组件库: Element Plus 2.13
- 状态管理: Vue 3 Reactivity API
- 拖拽库: vuedraggable 4.1.0
- 样式: SCSS + CSS Variables

### 后端技术（Rust）
- 框架: Tauri 2.0
- HTTP 客户端: reqwest 0.11 (支持 Cookie 和 JSON)
- HTML 解析: scraper 0.25 + regex 1.12
- 加密: des 0.8, md5 0.7, base64 0.21
- 序列化: serde/serde_json
- 时间处理: chrono 0.4
- 文件系统: dirs 5.0, uuid 1.0

---

## 项目结构

```
course-schedule-desktop/
├── src/                    # 前端源码
│   ├── components/         # Vue 组件
│   │   ├── CourseGrid.vue       # 课程网格
│   │   ├── WeekSelector.vue     # 周次选择器
│   │   ├── CourseCard.vue       # 课程卡片
│   │   ├── ScheduleEditDialog.vue   # 课表编辑对话框
│   │   ├── ImportScheduleDialog.vue  # 导入课表对话框
│   │   └── TimeTableManager.vue  # 时间表管理器
│   ├── composables/        # 组合式函数
│   │   ├── useCourse.ts         # 课表数据管理
│   │   └── useTimeTable.ts      # 时间表逻辑
│   ├── utils/              # 工具函数
│   │   ├── color.ts             # 颜色工具
│   │   └── date.ts              # 日期工具
│   ├── types.ts            # TypeScript 类型定义
│   └── App.vue             # 根组件
├── src-tauri/             # Rust 后端
│   ├── src/
│   │   ├── lib.rs              # Tauri 命令定义与注册
│   │   ├── models.rs           # 数据模型 (Course, ScheduleMetadata, etc.)
│   │   ├── parser.rs           # HTML 课表解析器
│   │   ├── storage.rs          # 本地存储管理
│   │   ├── client.rs           # HTTP 客户端
│   │   └── crypto.rs           # 加密模块
│   ├── Cargo.toml              # Rust 依赖配置
│   └── tauri.conf.json         # Tauri 配置
├── package.json           # 前端依赖配置
├── README.md              # 项目文档
└── CLAUDE.md              # AI 上下文文档
```

---

## 开发指南

### 添加新的教务系统支持

1. 修改 `src-tauri/src/parser.rs` 中的 `parse_course_html` 函数
2. 适配目标教务系统的 HTML 表格结构
3. 处理特殊的时间格式、节次表示、周次格式
4. 测试解析结果的准确性

### 添加新的 Tauri 命令

1. 在 `src-tauri/src/lib.rs` 添加 `#[tauri::command]` 函数
2. 在 `invoke_handler!` 宏中注册命令（重要：未注册的命令无法被前端调用）
3. 前端使用 `invoke()` 调用

**Rust 端示例** (`src-tauri/src/lib.rs`):
```rust
#[tauri::command]
fn my_command(param: String) -> Result<String, String> {
    Ok(format!("Hello, {}!", param))
}

// 在 run() 函数中注册
.invoke_handler(tauri::generate_handler![
    // ... 其他命令
    my_command,  // 必须在此注册
])
```

**前端调用示例**:
```typescript
import { invoke } from '@tauri-apps/api/core';

const result = await invoke('my_command', { param: 'World' });
console.log(result); // "Hello, World!"
```

### 构建配置

```bash
# 开发模式
npm run tauri dev

# 构建
npm run tauri build

# 仅构建前端
npm run build
```

---

## 常见问题

### Q: 为什么需要剪贴板权限？

A: 由于 Tauri 2 的安全限制，外部 HTTPS 网页（教务系统）无法直接访问 Tauri API。因此使用剪贴板作为数据传输通道。点击"导入当前课表"按钮会将 HTML 源代码复制到剪贴板，前端再自动读取。

### Q: 课表数据存储在哪里？

A:
- Windows: `C:\Users\<用户>\AppData\Local\course-schedule\`
- macOS: `~/Library/Application Support/course-schedule/`
- Linux: `~/.local/share/course-schedule/`

### Q: 如何支持单双周课程？

A: 解析器会自动识别周次范围，前端会根据：
- `weeks` 数组：包含该课程的周次列表（已展开为具体周次）
- `week_type` 字段：1=单周，2=双周，0=全周

来判断是否在当前周显示课程。

### Q: 如何重命名课表？

A:
1. 打开"课表管理" -> 点击要编辑的课表卡片上的编辑图标
2. 在"课表信息"部分找到"课表名称"输入框
3. 修改名称后点击"保存"按钮

### Q: 支持哪些教务系统？

A: 目前支持的教务系统包括：
- 中央财经大学教务系统（cufe_default）
- 浙江大学教务系统（zju_default）

可以通过修改 `src-tauri/src/parser.rs` 添加对新教务系统的支持。

---

## 贡献指南

欢迎贡献代码！请遵循以下步骤：

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'feat: Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 提交 Pull Request

### 提交规范

遵循 [Conventional Commits](https://www.conventionalcommits.org/) 规范：

- `feat:` 新功能
- `fix:` 修复 bug
- `docs:` 文档更新
- `style:` 代码格式调整
- `refactor:` 代码重构
- `test:` 测试相关
- `chore:` 构建/工具链相关

---

## 许可证

本项目采用 [MIT](LICENSE) 许可证。

---

## 致谢

- [Tauri](https://tauri.app/) - 跨平台桌面应用框架
- [Vue.js](https://vuejs.org/) - 渐进式 JavaScript 框架
- [Element Plus](https://element-plus.org/) - Vue 3 UI 组件库
- [Rust](https://www.rust-lang.org/) - 系统编程语言

---

## 联系方式

- 作者: Li Junlei
- 项目地址: [https://github.com/lijunlei/course-schedule-desktop](https://github.com/lijunlei/course-schedule-desktop)
- 问题反馈: [Issues](https://github.com/lijunlei/course-schedule-desktop/issues)
