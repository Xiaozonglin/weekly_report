# XDSEC Weekly Report

(C) 2024-now XDSEC Developers

## 项目目录结构概览

下面是仓库的主要目录和文件的简要说明，帮助新来维护者快速理解项目布局：

```
.
├─ biome.json                # Biome / lint / formatter 配置
├─ Cargo.toml                # Workspace / Rust 根配置
├─ README.md                 # 本文件
├─ crates/
│  ├─ database/              # 数据库相关 crate（实体、查询、迁移逻辑）
│  │  ├─ Cargo.toml
│  │  └─ src/
│  │     ├─ lib.rs
│  │     └─ entities/        # ORM 实体（user, report, config 等）
│  └─ server/                # 后端服务（axum + handlers + middleware）
│     ├─ Cargo.toml
│     └─ src/
│        ├─ main.rs
│        ├─ lib.rs
│        ├─ routes/          # 所有 HTTP 路由定义
│        └─ middleware/      # 中间件（auth, data 等）
├─ docs/                     # 文档（非必需，但用于项目说明或设计文档）
├─ web/                      # 前端（SolidJS + Vite）
│  ├─ package.json
│  ├─ vite.config.mts
│  └─ src/
│     ├─ index.tsx           # 前端入口
│     ├─ lib/                # 前端库代码（api、i18n、storage、widgets 等）
│     │  ├─ api/
│     │  ├─ i18n/
│     │  └─ widgets/
│     └─ routes/             # 前端页面路由
└─ public/                   # 前端静态资源（manifest 等）

```

简要说明：
- `crates/database`：包含数据模型和数据库访问函数（sea-orm / sqlx）。
- `crates/server`：后端 HTTP 服务实现，路由在 `routes/`，鉴权与请求预处理在 `middleware/`。
- `web`：前端单页应用，使用 Vite 构建。常见修改点包括 `src/lib/api`（与后端交互的封装）和 `src/routes`（页面逻辑）。

## 环境变量配置（开发 & 生产）

项目使用后端和前端的环境变量来控制运行行为。下面给出维护者和贡献者的简要说明，包括应该设置哪些变量、示例以及这些变量会影响的功能。


### 后端（server）

- `DATABASE_URL`（必需）
	- 说明：数据库连接字符串。
	- 示例（MySQL）：`mysql://root:password@127.0.0.1/wr`

- `WR_HOST`（必需）
	- 说明：服务绑定的主机地址

- `WR_PORT`（必需）
	- 说明：服务监听的端口

- `WR_PUBLIC_URL`（可选）
	- 说明：后端用于生成站点级绝对链接（例如 RSS feed 中的 self 链接或在服务端构造的 URL）的基准公开地址。该值应设置为你的站点对外可访问的完整根 URL（例如 `https://weekly.example.com`）。
	- 默认：如果未设置，服务在代码中回退到 `http://localhost`（见 `crates/server/src/routes/mod.rs` 中的处理）。
	- 注意：请在生产环境中设置该值以确保 feed 中的链接与真实域名一致；设置时通常不需要尾部斜杠（代码会安全地去除尾斜杠）。

### 本地开发示例（PowerShell）

在启动后端和前端前，可在当前 PowerShell 会话中临时设置：

```powershell
$env:WR_HOST = '127.0.0.1'
$env:WR_PORT = '8080'
$env:DATABASE_URL = 'mysql://root:root@localhost/wr'
$env:WR_STATIC = 'C:\Users\you\path\to\weekly_report\web'
$env:RUST_LOG = 'info'
$env:WR_PUBLIC_URL = 'http://127.0.0.1:8080'
```

然后在不同终端分别启动后端与前端：

```powershell
# 启动后端
cd C:\Users\you\path\to\weekly_report
cargo run --bin wr-server

# 启动前端开发服务器
cd C:\Users\you\path\to\weekly_report\web
npm install
npm run dev
```

## API 行为说明更新

### /api/status

现在该接口仅返回“当前所有未隐藏用户中 level 最大的那一批成员”的本周提交情况，而不是返回全部成员。返回结构保持不变：

```json
{
	"submitted": ["Alice", "Bob"],
	"pending": ["Charlie"]
}
```

语义：
- `submitted`：本周（以最近的周日为边界计算出的周编号）已提交周报的、且 level == 全部用户最大 level 的成员名称列表。
- `pending`：同一批用户里尚未提交的成员名称列表。

注意：如果暂时没有用户（空列表），两个数组都会为空。为了响应稳定性，内部会按名称排序。

如果未来需要再次查看全部用户提交情况，可考虑新增一个例如 `/api/status/all` 的端点（当前未实现）。
