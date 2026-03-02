# 回滚说明文档

本文档提供详细的回滚步骤，用于恢复到修改前的状态。

---

## 📋 回滚概览

**回滚目标**: 恢复到Microsoft Copilot应用打包前的原始状态

**回滚类型**: 完全回滚（恢复所有修改的文件）

**预计耗时**: 2-3分钟

---

## ⚠️ 重要提示

在执行回滚之前，请注意：
1. 回滚将删除所有新增的文件和配置
2. 回滚将恢复备份的配置文件
3. GitHub Actions workflow将被删除
4. 如有未提交的更改，请先备份

---

## 🔄 快速回滚（推荐）

### Windows PowerShell脚本

创建并运行以下脚本进行自动回滚：

```powershell
# rollback.ps1
# Microsoft Copilot应用回滚脚本

Write-Host "======================================" -ForegroundColor Cyan
Write-Host "  Microsoft Copilot 回滚脚本" -ForegroundColor Cyan
Write-Host "======================================" -ForegroundColor Cyan
Write-Host ""

# 定义项目根目录
$ProjectRoot = "f:\谷歌下载存放\Pake-main"

# 确认回滚操作
$Confirm = Read-Host "确认要回滚所有更改吗？(Y/N)"
if ($Confirm -ne "Y" -and $Confirm -ne "y") {
    Write-Host "回滚已取消" -ForegroundColor Yellow
    exit
}

Write-Host ""
Write-Host "开始回滚..." -ForegroundColor Green

# 1. 恢复配置文件
Write-Host ""
Write-Host "[1/5] 恢复配置文件..." -ForegroundColor Cyan

$BackupFiles = @(
    "src-tauri\pake.json",
    "src-tauri\tauri.conf.json",
    "src-tauri\tauri.windows.conf.json",
    "src-tauri\src\inject\style.js"
)

foreach ($File in $BackupFiles) {
    $BackupPath = Join-Path $ProjectRoot "$File.backup"
    $TargetPath = Join-Path $ProjectRoot $File
    
    if (Test-Path $BackupPath) {
        Copy-Item $BackupPath $TargetPath -Force
        Write-Host "  ✓ 已恢复: $File" -ForegroundColor Green
    } else {
        Write-Host "  ✗ 备份不存在: $File.backup" -ForegroundColor Yellow
    }
}

# 2. 删除新增的图标文件
Write-Host ""
Write-Host "[2/5] 删除新增的图标文件..." -ForegroundColor Cyan

$IconFiles = @(
    "src-tauri\png\copilot_256.ico",
    "src-tauri\png\copilot_32.ico",
    "src-tauri\png\copilot_512.png"
)

foreach ($File in $IconFiles) {
    $FilePath = Join-Path $ProjectRoot $File
    if (Test-Path $FilePath) {
        Remove-Item $FilePath -Force
        Write-Host "  ✓ 已删除: $File" -ForegroundColor Green
    }
}

# 3. 删除GitHub Actions workflow
Write-Host ""
Write-Host "[3/5] 删除GitHub Actions配置..." -ForegroundColor Cyan

$WorkflowPath = Join-Path $ProjectRoot ".github\workflows\build-copilot.yml"
if (Test-Path $WorkflowPath) {
    Remove-Item $WorkflowPath -Force
    Write-Host "  ✓ 已删除: .github\workflows\build-copilot.yml" -ForegroundColor Green
}

# 4. 删除文档文件
Write-Host ""
Write-Host "[4/5] 删除文档文件..." -ForegroundColor Cyan

$DocFiles = @(
    "README_COPILOT.md",
    "ROLLBACK.md",
    "check_env.ps1"
)

foreach ($File in $DocFiles) {
    $FilePath = Join-Path $ProjectRoot $File
    if (Test-Path $FilePath) {
        Remove-Item $FilePath -Force
        Write-Host "  ✓ 已删除: $File" -ForegroundColor Green
    }
}

# 5. 恢复rust-toolchain.toml
Write-Host ""
Write-Host "[5/5] 恢复Rust工具链配置..." -ForegroundColor Cyan

$RustToolchainPath = Join-Path $ProjectRoot "rust-toolchain.toml"
$RustToolchainContent = @"
[toolchain]
channel = "1.93.0"
components = ["rustfmt", "clippy"]
"@

Set-Content -Path $RustToolchainPath -Value $RustToolchainContent -NoNewline
Write-Host "  ✓ 已恢复: rust-toolchain.toml" -ForegroundColor Green

# 6. 删除备份文件（可选）
Write-Host ""
$DeleteBackups = Read-Host "是否删除备份文件？(Y/N)"
if ($DeleteBackups -eq "Y" -or $DeleteBackups -eq "y") {
    foreach ($File in $BackupFiles) {
        $BackupPath = Join-Path $ProjectRoot "$File.backup"
        if (Test-Path $BackupPath) {
            Remove-Item $BackupPath -Force
            Write-Host "  ✓ 已删除备份: $File.backup" -ForegroundColor Green
        }
    }
}

Write-Host ""
Write-Host "======================================" -ForegroundColor Cyan
Write-Host "  回滚完成！" -ForegroundColor Green
Write-Host "======================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "已恢复到修改前的状态。" -ForegroundColor Green
Write-Host "注意: .tasks目录中的任务文件已保留，如需删除请手动操作。" -ForegroundColor Yellow
```

### 执行快速回滚

```powershell
# 保存上述脚本为 rollback.ps1，然后执行
powershell -ExecutionPolicy Bypass -File rollback.ps1
```

---

## 🔧 手动回滚步骤

如果自动脚本无法执行，请按以下步骤手动回滚：

### 步骤1: 恢复配置文件

```bash
# 进入项目目录
cd "f:\谷歌下载存放\Pake-main"

# 恢复pake.json
copy "src-tauri\pake.json.backup" "src-tauri\pake.json"

# 恢复tauri.conf.json
copy "src-tauri\tauri.conf.json.backup" "src-tauri\tauri.conf.json"

# 恢复tauri.windows.conf.json
copy "src-tauri\tauri.windows.conf.json.backup" "src-tauri\tauri.windows.conf.json"

# 恢复style.js
copy "src-tauri\src\inject\style.js.backup" "src-tauri\src\inject\style.js"
```

### 步骤2: 删除新增图标文件

```bash
del "src-tauri\png\copilot_256.ico"
del "src-tauri\png\copilot_32.ico"
del "src-tauri\png\copilot_512.png"
```

### 步骤3: 删除GitHub Actions配置

```bash
del ".github\workflows\build-copilot.yml"
```

### 步骤4: 删除文档文件

```bash
del "README_COPILOT.md"
del "ROLLBACK.md"
del "check_env.ps1"
```

### 步骤5: 恢复rust-toolchain.toml

创建或编辑 `rust-toolchain.toml` 文件：

```toml
[toolchain]
channel = "1.93.0"
components = ["rustfmt", "clippy"]
```

### 步骤6: 清理构建缓存（可选）

```bash
# 删除target目录
rmdir /s /q "src-tauri\target"

# 清理node_modules（如需完全重置）
rmdir /s /q "node_modules"
```

---

## 📂 回滚验证清单

完成回滚后，请验证以下内容：

### 配置文件验证
- [ ] `src-tauri/pake.json` - URL应为 `https://weekly.tw93.fun/en`
- [ ] `src-tauri/tauri.conf.json` - productName应为 `Weekly`
- [ ] `src-tauri/tauri.windows.conf.json` - 图标路径应为 `png/weekly_*.ico`
- [ ] `src-tauri/src/inject/style.js` - 应无GPU加速CSS注入

### 文件系统验证
- [ ] `src-tauri/png/copilot_*.ico` - 文件已删除
- [ ] `src-tauri/png/copilot_*.png` - 文件已删除
- [ ] `.github/workflows/build-copilot.yml` - 文件已删除
- [ ] `README_COPILOT.md` - 文件已删除
- [ ] `ROLLBACK.md` - 文件已删除（可选）
- [ ] `check_env.ps1` - 文件已删除

### 功能验证
- [ ] 运行 `pnpm run dev` - 应启动Weekly应用
- [ ] 运行 `pnpm run build` - 应正常构建Weekly应用

---

## 🔍 Git回滚（如果已提交）

如果更改已经提交到Git仓库，可以使用Git命令回滚：

### 方案1: Git Revert（推荐）

```bash
# 查看提交历史
git log --oneline

# 找到修改前的提交哈希
# 例如: abc1234

# 创建revert提交
git revert abc1234

# 或者revert最近的提交
git revert HEAD
```

### 方案2: Git Reset（危险操作）

```bash
# ⚠️ 警告: 这将丢失所有未推送的提交

# 硬重置到指定提交
git reset --hard abc1234

# 或者重置到最近的提交（保留工作区更改）
git reset --soft HEAD~1
```

---

## 📊 回滚影响评估

### 影响范围
- ✅ 配置文件: 4个文件恢复
- ✅ 图标资源: 3个文件删除
- ✅ GitHub Actions: 1个文件删除
- ✅ 文档文件: 3个文件删除
- ✅ Rust配置: 1个文件恢复

### 不受影响的文件
- 📁 `.tasks/` - 任务文件保留
- 📁 `node_modules/` - 依赖包保留
- 📁 `src-tauri/target/` - 构建缓存保留
- 🔧 环境变量 - Rust/Node.js路径配置保留

---

## 🆘 回滚问题排查

### 问题1: 备份文件不存在

**症状**: 提示"备份文件不存在"

**解决方案**:
- 检查备份文件是否已手动删除
- 从Git历史恢复原始文件
- 重新克隆原始仓库

### 问题2: 文件被占用无法删除

**症状**: 提示"文件正在使用"

**解决方案**:
```powershell
# 强制删除（需管理员权限）
Remove-Item "文件路径" -Force
```

### 问题3: Git冲突

**症状**: Git操作失败

**解决方案**:
```bash
# 放弃本地更改
git checkout -- .

# 或强制清理
git clean -fdx
```

---

## 📞 技术支持

如遇回滚问题，请检查：
1. 是否有文件权限问题
2. 是否有进程占用文件
3. Git状态是否正常
4. 磁盘空间是否充足

---

## 📝 回滚记录

| 时间 | 操作人 | 回滚原因 | 状态 |
|------|--------|---------|------|
| - | - | - | - |

---

*最后更新时间: 2026-03-02 18:45:00*
*文档版本: 1.0*
