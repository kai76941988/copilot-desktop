# 变更日志

## [2026-03-13]

### 变更摘要
修复多个问题并添加 MCP 支持

### 修改文件列表
1. `src-tauri/tauri.conf.json` - 移除 trayIcon 配置块
2. `src-tauri/tauri.windows.conf.json` - 更改 bundle 图标为 copilot_1.ico
3. `src-tauri/pake.json` - 更改图标路径，添加 MCP 配置
4. `src-tauri/src/inject/event.js` - 修复全屏缩放问题，添加全屏监听器
5. `src-tauri/src/app/setup.rs` - 移除冗余 tray 删除代码
6. `src-tauri/Cargo.toml` - 添加 MCP 依赖
7. `src-tauri/src/app/config.rs` - 添加 MCP 配置结构
8. `src-tauri/src/app/mcp.rs` - 【新增】MCP 服务器核心实现
9. `src-tauri/src/app/mcp_tools.rs` - 【新增】MCP 工具定义
10. `src-tauri/src/app/mcp_resources.rs` - 【新增】MCP 资源定义
11. `src-tauri/src/app/mcp_prompts.rs` - 【新增】MCP 提示模板
12. `src-tauri/src/lib.rs` - 初始化 MCP 服务
13. `README.md` - 添加 MCP 使用说明
14. `README_CN.md` - 添加 MCP 使用说明
15. `README_COPILOT.md` - 更新功能说明和变更日志

### 修改原因
1. **图标更改**: 用户要求更改图标为 copilot_1.ico
2. **双图标修复**: tauri.conf.json 中的 trayIcon 配置与代码创建的托盘冲突
3. **全屏字体修复**: Windows 平台全屏时 transform scale 导致字体重叠
4. **MCP 支持**: 用户要求支持 MCP 协议，便于 AI 助手调用

### 执行模式
执行

### 状态
待审查

---

## 回滚说明

### 回滚方式
使用 Git 回滚到修改前的提交：

```bash
git checkout HEAD~1 -- .
```

或手动恢复以下文件：
1. 从备份目录恢复原始配置文件
2. 删除新增的 MCP 相关文件：
   - `src-tauri/src/app/mcp.rs`
   - `src-tauri/src/app/mcp_tools.rs`
   - `src-tauri/src/app/mcp_resources.rs`
   - `src-tauri/src/app/mcp_prompts.rs`
3. 恢复 Cargo.toml 中的原始依赖

### 验证步骤
1. 运行 `pnpm run dev` 验证应用启动正常
2. 检查托盘图标是否显示正确（单图标）
3. 验证全屏模式字体显示正常
4. 测试 MCP 服务是否正常响应

[2026-03-13 20:11:34]
* ??????? Copilot ?????/??????? Ctrl+??????
* ?????src-tauri/src/inject/event.js, src-tauri/src/inject/style.js, README.md, .tasks/log.md, ROLLBACK.md
* ??????? 1366x768 ??????????????????/????? GitHub ??????
* ???????
* ??????
