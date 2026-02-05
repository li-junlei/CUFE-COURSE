# CUFE 课程表桌面应用

专为中央财经大学（CUFE）学生设计的课程表管理桌面应用

基于 [Tauri 2.0](https://tauri.app/) + [Vue 3](https://vuejs.org/) + [TypeScript](https://www.typescriptlang.org/) 构建

[![Version](https://img.shields.io/badge/version-2.1.0-blue.svg)](https://github.com/lijunlei/course-schedule-desktop)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-2.0-FFC131.svg)](https://tauri.app/)
[![Vue](https://img.shields.io/badge/Vue-3.5-4FC08D.svg)](https://vuejs.org/)

---

## 功能特性

### 核心功能
- **一键课表导入** - 直接从中央财经大学教务系统导入课程数据
- **持久化登录** - 安全保存登录凭证，重启应用自动恢复登录状态
- **多课表管理** - 创建、切换、重命名、删除、排序多个课表
- **智能周数管理** - 自动根据学期设置显示周数范围（10-30周灵活配置）
- **自定义时间表** - 支持多套节次时间配置，灵活适配不同学期

### 界面特性
- **周次预览** - 点阵视图展示每周课程分布，一目了然
- **滑动切换** - 手势滑动切换周次，流畅自然
- **课程详情** - 点击课程卡片查看完整信息
- **深色模式** - 支持手动切换和跟随系统主题

### 外观定制
- **网格辅助线** - 可开关的虚线网格，方便对齐查看
- **卡片透明度** - 可调节课程卡片不透明度（50-100%）
- **信息显示控制** - 自定义显示授课教师/上课地点
- **自定义背景** - 支持上传本地图片作为课表背景

### 智能提示
- **学期状态** - 自动显示"未开学"/"学期已结束"状态
- **今日高亮** - 强调显示当前日期，快速定位
- **登录状态保持** - 应用重启后自动恢复登录，无需重复输入

---

## 版本历史

### v2.1.0 (2026-02-05)
- ✨ **新增课表在线更新功能**
  - 支持从教务系统重新获取最新课表数据
  - 智能差异统计：显示新增/删除/修改的课程数量
  - 会话失效时自动重登录，无需手动输入密码
- 💾 自动保存学年学期信息，便于后续更新
- 🎨 课表管理界面显示学年学期，新增专用的更新按钮
- ♻️ 完全向后兼容，旧课表（无学年学期）不影响使用
- 🎯 友好的用户提示和错误处理

### v2.0.0 (2026-02-04)
- ✨ **新增持久化登录功能**
  - 使用 Windows DPAPI 安全加密存储登录凭证
  - 应用重启后自动恢复登录状态
  - 导入课表时自动重登录（会话失效时）
- 🔧 优化应用启动体验，异步加载不阻塞 UI
- 📱 改进用户状态管理，组件间状态同步
- 🎯 聚焦于中央财经大学教务系统适配

### v1.7.0 (2026-02-02)
- ✨ 新增课表重命名功能，支持在课表编辑对话框中直接修改课表名称
- 🐛 修复中央财经大学解析器遗漏课程的问题
- 🔧 优化 Tauri 命令注册，确保所有前端调用正确路由到后端

---

## 快速开始

### 环境要求

- Node.js 18+
- Rust 1.70+ (含 Cargo)
- npm 或 yarn
- Windows 10+ (支持 DPAPI 加密)

### 安装步骤

1. **下载安装包**

   从 [Releases](https://github.com/lijunlei/course-schedule-desktop/releases) 页面下载最新版本的安装包

2. **安装并运行**

   双击安装包，按照提示完成安装

3. **首次使用**

   - 点击右上角"个人中心"图标
   - 输入学号和密码登录 CUFE 教务系统
   - 登录成功后凭证自动保存，下次启动无需重新登录
   - 点击"导入课表"开始导入课程数据

### 从源码构建

1. **克隆仓库**
   ```bash
   git clone https://github.com/lijunlei/course-schedule-desktop.git
   cd course-schedule-desktop
   ```

2. **安装依赖**
   ```bash
   npm install
   ```

3. **开发模式**
   ```bash
   npm run tauri dev
   ```

4. **构建生产版本**
   ```bash
   npm run tauri build
   ```

---

## 使用指南

### 导入课表

1. 点击左上角菜单按钮
2. 选择"导入课表"
3. 选择学期（如 2024-2025 学年第1学期）
4. 输入课表名称（可选）
5. 点击"导入"按钮
6. 系统自动使用保存的登录凭证导入课表

> **提示**：首次使用需要先在"个人中心"登录教务系统

### 管理多个课表

- **创建**: 通过"导入课表"创建新课表
- **切换**: 在"课表管理"中点击课表卡片切换
- **重命名**: 点击编辑图标，在"课表信息"中修改课表名称
- **排序**: 拖拽课表卡片调整顺序
- **编辑**: 点击编辑图标修改学期设置和时间表方案
- **删除**: 点击删除图标移除不需要的课表

### 自定义时间表

1. 打开"课表管理" -> 点击要编辑的课表卡片上的编辑图标
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

- **网格辅助线**: 在"设置" -> "课表外观设置"中开启
- **卡片透明度**: 拖动滑块调整（50-100%）
- **信息显示**: 开关"显示授课教师"或"显示上课地点"
- **背景图片**: 点击菜单 -> "上传背景"选择本地图片

---

## 技术栈

### 前端技术
- **框架**: Vue 3 (Composition API + `<script setup>`)
- **类型系统**: TypeScript 5.6
- **构建工具**: Vite 6.0
- **UI 组件库**: Element Plus 2.13
- **状态管理**: Vue 3 Reactivity API
- **拖拽库**: vuedraggable 4.1.0
- **样式**: SCSS + CSS Variables

### 后端技术（Rust）
- **框架**: Tauri 2.0
- **HTTP 客户端**: reqwest 0.11 (支持 Cookie 和 JSON)
- **加密**: Windows DPAPI (密码加密), DES/MD5 (登录参数)
- **序列化**: serde/serde_json
- **时间处理**: chrono 0.4
- **文件系统**: dirs 5.0, uuid 1.0

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
│   │   ├── UserProfileDialog.vue    # 用户中心/登录
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
│   │   ├── models.rs           # 数据模型 (Course, ScheduleMetadata, PersistentCredentials)
│   │   ├── parser.rs           # CUFE JSON 课表解析器
│   │   ├── storage.rs          # 本地存储管理
│   │   ├── client.rs           # HTTP 客户端与会话管理
│   │   └── crypto.rs           # 加密模块 (DPAPI, DES, MD5)
│   ├── Cargo.toml              # Rust 依赖配置
│   └── tauri.conf.json         # Tauri 配置
├── package.json           # 前端依赖配置
├── README.md              # 项目文档
└── CLAUDE.md              # AI 上下文文档
```

---

## 开发指南

### 添加新的功能

1. 在 `src/components/` 创建新的 Vue 组件
2. 在 `src-tauri/src/lib.rs` 添加对应的 Tauri 命令
3. 在 `invoke_handler!` 宏中注册命令
4. 在前端使用 `invoke()` 调用后端功能

### Tauri 命令示例

**Rust 端** (`src-tauri/src/lib.rs`):
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

**前端调用**:
```typescript
import { invoke } from '@tauri-apps/api/core';

const result = await invoke('my_command', { param: 'World' });
console.log(result); // "Hello, World!"
```

### 课表解析

修改 `src-tauri/src/parser.rs` 中的 `parse_cufe_json` 函数以适配 CUFE 教务系统的 JSON 格式变化。

---

## 常见问题

### Q: 登录凭证存储在哪里？是否安全？

A: 登录凭证存储在 `%LOCALAPPDATA%\cufe-course\credentials.json`。密码使用 **Windows DPAPI** 加密，只有当前 Windows 用户可以解密，即使文件被复制到其他计算机也无法解密。

### Q: 为什么第一次登录后，重启应用还是需要重新登录？

A: 请确保：
1. 登录成功后看到"登录成功"的提示消息
2. 凭证文件 `credentials.json` 已创建
3. 应用正常关闭（不是强制结束进程）

### Q: 课表数据存储在哪里？

A:
- **Windows**: `C:\Users\<用户>\AppData\Local\cufe-course\schedules\`
- **macOS**: `~/Library/Application Support/cufe-course/schedules/`
- **Linux**: `~/.local/share/cufe-course/schedules/`

### Q: 如何支持单双周课程？

A: 解析器会自动识别周次范围，前端会根据 `weeks` 数组和 `week_type` 字段来判断是否在当前周显示课程：
- `week_type = 1`: 单周
- `week_type = 2`: 双周
- `week_type = 0`: 全周

### Q: 导入课表失败怎么办？

A:
1. 检查网络连接是否正常
2. 确认已在"个人中心"登录教务系统
3. 尝试退出后重新登录
4. 查看控制台日志获取详细错误信息

### Q: 支持哪些学校？

A: 当前版本专注于**中央财经大学教务系统**的适配。如需添加其他学校支持，需要：
1. 在 `src-tauri/src/parser.rs` 添加新的解析函数
2. 适配目标学校的教务系统 API
3. 实现相应的登录和数据获取逻辑

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
- **中央财经大学教务系统** - 课程数据来源

---

## 联系方式

- **作者**: Li Junlei
- **项目地址**: [https://github.com/lijunlei/course-schedule-desktop](https://github.com/lijunlei/course-schedule-desktop)
- **问题反馈**: [Issues](https://github.com/lijunlei/course-schedule-desktop/issues)
