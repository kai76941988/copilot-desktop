# 回滚说明

## 当前版本信息
- **版本**: 1.1.0
- **日期**: 2026-03-13
- **变更**: 修复托盘双图标、全屏字体重叠问题，添加 MCP 支持

## 回滚方法

### 方法一：Git 回滚（推荐）

如果您在修改前已提交代码：

```bash
# 查看最近的提交
git log --oneline -5

# 回滚到指定提交
git reset --hard <commit_hash>

# 或者使用 revert 创建新提交
git revert <commit_hash>
```

### 方法二：手动恢复

#### 需要删除的新增文件
```
src-tauri/src/app/mcp.rs
src-tauri/src/app/mcp_tools.rs
src-tauri/src/app/mcp_resources.rs
src-tauri/src/app/mcp_prompts.rs
.tasks/log.md
```

#### 需要恢复的文件

**1. src-tauri/tauri.conf.json**
恢复 trayIcon 配置：
```json
{
  "app": {
    "trayIcon": {
      "iconPath": "png/copilot_32.ico",
      "iconAsTemplate": false,
      "id": "copilot-tray"
    }
  }
}
```

**2. src-tauri/tauri.windows.conf.json**
恢复原始图标配置：
```json
{
  "bundle": {
    "icon": ["png/copilot_256.ico", "png/copilot_32.ico"],
    "resources": ["png/copilot_32.ico"]
  }
}
```

**3. src-tauri/pake.json**
移除 MCP 配置，恢复原始图标路径：
```json
{
  "system_tray_path": "png/copilot_32.ico"
}
```

**4. src-tauri/Cargo.toml**
移除 MCP 依赖：
```toml
# 移除以下依赖
axum = "0.7"
tower = "0.4"
tower-http = { version = "0.5", features = ["cors"] }
tokio-stream = "0.1"
futures = "0.3"
base64 = "0.22"
```

**5. src-tauri/src/app/config.rs**
移除 McpConfig 结构体和 PakeConfig 中的 mcp 字段

**6. src-tauri/src/app/setup.rs**
恢复原始的 tray 删除代码：
```rust
app.app_handle().remove_tray_by_id("pake-tray");
```

**7. src-tauri/src/inject/event.js**
恢复原始的 setZoom 函数（移除全屏检测）

**8. src-tauri/src/lib.rs**
移除 MCP 服务启动代码

## 验证回滚成功

1. 编译项目：`pnpm run build`
2. 运行开发模式：`pnpm run dev`
3. 检查：
   - 托盘显示单图标
   - 应用使用原始图标
   - 无 MCP 服务启动

## 注意事项

- 回滚后需要重新编译项目
- 如有数据库或配置变更，可能需要手动处理
- 建议在回滚前备份当前工作目录


## ?????2026-03-13 20:13:31?
- ????/????????? Git ??
- ?????src-tauri/src/inject/event.js, src-tauri/src/inject/style.js, README.md, .tasks/log.md, ROLLBACK.md

### ?????Git?
???? Git ??????? `git reset` / `git revert`?

### ??????
1. ?????????????????????
   src-tauri/src/inject/event.js
   src-tauri/src/inject/style.js
   README.md
   .tasks/log.md
   ROLLBACK.md
2. ?? Copilot ??????????????????

### ??????
- Copilot ??????????
- ???????????
- Ctrl+?????????????????
