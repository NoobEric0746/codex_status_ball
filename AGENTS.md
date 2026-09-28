# AGENTS.md

## 项目目标

这是一个 Windows 优先的桌面悬浮球应用，使用极简方式展示 Codex 当前账户的：

- 5 小时额度剩余比例与重置倒计时
- 周额度剩余比例与重置倒计时

美术样式尚未确定。实现阶段先保证数据正确、状态清晰、悬浮球可用和 exe 可分发，不提前固化视觉方案。

## 推荐技术边界

- 桌面壳：Tauri 2
- 前端：React + TypeScript + Vite
- 原生后端：Rust
- Codex 通信：优先复用 Codex Tracker 的 `codex app-server --stdio` JSON-RPC 方案，通过已安装的 Codex CLI 获取 allowance 数据。
- 不读取、解析或复制 Codex 本地认证文件；认证应继续由 Codex 官方流程和系统浏览器管理。
- Windows 是第一目标平台；最终交付使用 Tauri Windows installer/exe。

除非后续有明确理由，不要引入 Electron 或自建 HTTP 服务来替代上述边界。参考项目：<https://github.com/heavymaskstudio/Codex-Tracker>。

## 设计与行为约束

- 悬浮球保持置顶、可拖动，并尽量不干扰正常窗口操作。
- 主界面只突出 5 小时和周额度；详细信息、刷新、设置、退出等放入展开面板或托盘菜单。
- 数据状态必须区分：正常、加载中、未登录、Codex 未安装、连接中断、数据过期。
- 连接中断时保留最近一次快照并明确标记过期，不要将临时故障显示成 0% 或空数据。
- 使用服务端返回的 allowance 窗口和重置时间，不自行猜测绝对 token 上限。
- 解析大数或 token 计数时避免 JavaScript `Number` 精度丢失，必要时使用字符串或 `BigInt` 兼容表示。
- 后台刷新应可配置或至少集中管理，避免每个组件各自启动定时器。
- 不在日志、错误提示或提交内容中暴露认证信息、令牌或完整敏感响应。

## 目录与职责

保持职责清晰：

- `src/`：React 展示、交互、格式化和视图状态
- `src-tauri/`：Rust 原生窗口/托盘、Codex 子进程监督、JSON-RPC、通知和持久化设置
- `tests/` 或前端邻近测试目录：额度解析、状态转换和关键 UI 行为
- `artifacts/`：构建产物；不要提交生成的安装包

前后端通信使用明确的、可版本化的类型；不要把 Rust 内部进程细节直接暴露给组件。

## 开发与验证

项目初始化后，优先提供并使用以下命令；若 `package.json` 中已有不同命令，以实际脚本为准：

```powershell
npm.cmd install
npm.cmd test
npm.cmd run build
cargo check --manifest-path src-tauri\Cargo.toml
npm.cmd run tauri dev
npm.cmd run build:exe
```

涉及真实 Codex 通信时，同时保留不依赖登录态的协议/响应样例测试；UI 测试不要把网络或本机 Codex 进程作为唯一数据源。Windows 发布前验证 WebView2、安装包启动、托盘菜单、置顶/拖动、登录缺失和断线恢复。

## 实现工作流

1. 先确认 Codex CLI 的可执行文件发现、`app-server` 启动和 JSON-RPC 响应形状，再设计领域类型。
2. 先写 allowance 映射与状态机测试，再接入悬浮球视图。
3. 原生能力优先放 Rust；React 只负责展示和用户交互。
4. 每次改动保持小范围，并运行受影响的最窄测试；完成前至少运行前端构建和 Rust 检查。
5. 美术方案确定后再扩展主题、动效和素材，不要在数据链路未稳定时大规模改视觉。

## 文档与依赖

新增依赖前说明它解决的具体问题，并确认许可证与 Windows 打包兼容性。对外部协议或参考实现做重要改动时，在项目文档中记录来源和验证方式。不要复制参考仓库的品牌名称、标识或素材。
