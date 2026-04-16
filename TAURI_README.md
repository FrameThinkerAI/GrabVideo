# Tauri 桌面应用构建说明

本项目已配置为使用 Tauri 打包为桌面应用程序。

## 工作原理

- **开发模式**：使用 Next.js 开发服务器（`npm run dev`），Tauri 窗口加载 `http://localhost:3000`
- **生产模式**：
  1. 构建 Next.js standalone 版本
  2. 将 standalone 输出复制到 Tauri 资源目录（通过 `prepare-tauri` 脚本）
  3. Tauri 应用启动时，会在后台启动 Node.js 服务器
  4. 窗口加载 `http://localhost:3000`

## 开发

### 前置要求

1. **Node.js** (v18+)
2. **Rust** 和 **Cargo** - 安装方法：https://www.rust-lang.org/tools/install
3. **系统依赖**：
   - **macOS**: Xcode Command Line Tools
   - **Linux**: `libwebkit2gtk-4.0-dev`, `build-essential`, `curl`, `wget`, `file`, `libssl-dev`, `libgtk-3-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`
   - **Windows**: Microsoft C++ Build Tools

### 安装依赖

```bash
npm install
```

这将安装包括 `@tauri-apps/cli` 在内的所有依赖。

### 开发模式运行

```bash
npm run tauri:dev
```

这将：
1. 启动 Next.js 开发服务器（如果未运行）
2. 启动 Tauri 开发窗口（加载 `http://localhost:3000`）

### 构建生产版本

```bash
npm run tauri:build
```

这将：
1. 构建 Next.js standalone 版本
2. 运行 `prepare-tauri` 脚本复制文件到 Tauri 资源目录
3. 构建 Tauri 应用程序

构建产物位置：
- **macOS**: `src-tauri/target/release/bundle/macos/GrabVideo.app`
- **Windows**: `src-tauri/target/release/bundle/msi/GrabVideo_0.1.0_x64_en-US.msi`
- **Linux**: `src-tauri/target/release/bundle/appimage/GrabVideo_0.1.0_amd64.AppImage`

## 文件系统路径

在生产模式下，应用使用 Tauri 的应用数据目录作为基础路径：
- **Downloads**: `{AppData}/downloads`
- **Cache**: `{AppData}/cache`

各平台的应用数据目录：
- **macOS**: `~/Library/Application Support/com.grabvideo.app`
- **Windows**: `%APPDATA%\com.grabvideo.app`
- **Linux**: `~/.local/share/com.grabvideo.app`

## 代码修改说明

为了适配 Tauri，仅做了最小化的代码修改：

1. **src/server/constants.ts**: 添加了 `BASE_PATH` 环境变量支持，允许 Tauri 设置应用数据目录路径
   ```typescript
   const BASE_PATH = process.env.BASE_PATH || (isDevelopment ? process.cwd() : '/');
   ```

2. **其他现有代码未修改**，保持向后兼容。

## 注意事项

1. **Node.js 依赖**：最终用户需要安装 Node.js 才能运行应用。服务器在后台运行，需要 Node.js 可执行文件在系统 PATH 中。

2. **端口占用**：应用使用 3000 端口。如果端口被占用，服务器启动会失败。

3. **首次启动**：应用首次启动时可能需要几秒钟来启动服务器。如果看到连接错误，请等待几秒后刷新页面。

4. **静态资源**：`prepare-tauri` 脚本会自动复制 `.next/static` 目录到资源目录，确保静态资源可以正常加载。

## 故障排除

### 服务器启动失败

如果服务器启动失败，检查：
1. Node.js 是否已安装并在 PATH 中（运行 `node --version` 验证）
2. 3000 端口是否被占用（尝试更改端口或关闭占用端口的程序）
3. standalone 构建是否成功完成（检查 `.next/standalone` 目录是否存在）
4. `prepare-tauri` 脚本是否成功运行（检查 `src-tauri/resources/.next/standalone` 目录）

### 找不到 server.js

确保：
1. 运行了 `npm run build` 成功构建 Next.js
2. 运行了 `npm run prepare-tauri` 复制文件到资源目录
3. 检查 `src-tauri/resources/.next/standalone/server.js` 是否存在

### 构建错误

如果遇到 Rust 构建错误：
1. 确保 Rust 和 Cargo 已正确安装：`cargo --version`
2. 更新 Rust 工具链：`rustup update`
3. 检查系统依赖是否已安装（见前置要求）

## 项目结构

```
GrabVideo/
├── src-tauri/          # Tauri 后端代码
│   ├── src/
│   │   └── main.rs     # Rust 主文件，负责启动 Node.js 服务器
│   ├── icons/          # 应用图标
│   ├── Cargo.toml      # Rust 依赖配置
│   └── tauri.conf.json # Tauri 配置文件
├── scripts/
│   └── prepare-tauri.js # 复制 standalone 输出到资源目录的脚本
└── ...                 # 其他项目文件
```

