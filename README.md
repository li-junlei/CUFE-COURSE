# CUFE-COURSE

中央财经大学（CUFE）课程表桌面应用，基于 Tauri 2 + Vue 3 + TypeScript。

## 项目状态
- 当前版本：`2.5.0`
- 代码仓库：<https://github.com/li-junlei/CUFE-COURSE>
- Release 下载：<https://github.com/li-junlei/CUFE-COURSE/releases>

## 主要功能
- 教务系统登录与会话恢复
- 从教务系统导入课表（支持自动重登录）
- 多课表管理：切换、重命名、删除、排序
- 课表在线更新（差异统计：新增/删除/修改/未变）
- 考试安排导入（合并到现有课表，考试卡片高亮）
- 课表导入导出（JSON）
- 时间表方案管理（多套节次时间）
- 课表外观设置（背景图、网格、透明度、教师/地点显示）
- 上课提醒（系统通知）
- 系统托盘（关闭主窗口最小化到托盘）
- 软件更新检查（GitHub Releases）

## 技术栈
- 前端：Vue 3、TypeScript、Vite、Element Plus、vuedraggable
- 桌面端：Tauri 2
- 后端：Rust、reqwest、serde、chrono
- 通知与系统能力：tauri-plugin-notification / dialog / fs / opener
- 凭证加密：Windows DPAPI（非 Windows 下仅开发降级）

## 快速开始

### 环境要求
- Node.js 18+
- Rust 1.70+
- npm

### 本地开发
```bash
git clone https://github.com/li-junlei/CUFE-COURSE.git
cd CUFE-COURSE
npm install
npm run tauri dev
```

### 构建
```bash
npm run tauri build
```

## 使用说明

### 首次使用
1. 打开“个人中心”，输入学号密码登录。
2. 登录成功后会保存凭证，后续可自动恢复会话。
3. 在“课表管理”中新建/在线导入课表。

### 课表管理
- 在线导入：按学年学期导入。
- 文件导入：从 JSON 文件恢复课表。
- 文件导出：导出当前课表为 JSON。
- 更新课表：基于学年学期重新拉取并计算差异。
- 导入考试：将考试安排合并进课表。

### 设置与外观
- 设置：提醒开关、关闭窗口行为、手动检查更新。
- 外观：背景图、网格线、透明度、教师/地点显示、地点简化。

## 更新检查机制
应用通过以下接口读取最新发布：

`https://api.github.com/repos/li-junlei/CUFE-COURSE/releases/latest`

自动检查由 `auto_check_update` 控制，支持手动检查和跳过版本。

## 数据存储
应用数据目录为 `cufe-course`（由系统用户数据目录决定）：
- `config.json`：应用配置
- `credentials.json`：登录凭证（密码加密）
- `schedules/*.json`：课表缓存与元数据
- `backgrounds/*`：背景图

常见路径：
- Windows: `%LOCALAPPDATA%\cufe-course\`
- macOS: `~/Library/Application Support/cufe-course/`
- Linux: `~/.local/share/cufe-course/`

## 项目结构
```text
CUFE-COURSE/
├─ src/                          # Vue 前端
│  ├─ components/                # 主要界面组件
│  ├─ composables/               # 业务逻辑组合函数
│  ├─ utils/                     # 工具函数
│  ├─ types.ts                   # 前端类型定义
│  └─ App.vue                    # 主界面
├─ src-tauri/                    # Rust + Tauri
│  ├─ src/
│  │  ├─ commands/               # Tauri 命令
│  │  ├─ services/               # 业务服务（差异计算）
│  │  ├─ system/                 # 系统托盘
│  │  ├─ client.rs               # 教务系统客户端
│  │  ├─ parser.rs               # 课表/考试解析
│  │  ├─ storage.rs              # 本地存储
│  │  └─ lib.rs                  # 命令注册入口
│  └─ tauri.conf.json            # Tauri 配置
├─ package.json
└─ README.md
```

## 当前限制
- 当前实际适配目标为中央财经大学教务系统（CUFE）。
- 凭证安全能力以 Windows DPAPI 为主。

## 贡献
欢迎提交 Issue / PR：
- Issues: <https://github.com/li-junlei/CUFE-COURSE/issues>
- Pull Requests: <https://github.com/li-junlei/CUFE-COURSE/pulls>
