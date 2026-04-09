use std::path::PathBuf;

/// commit-msg hook 脚本模板
const COMMIT_MSG_HOOK: &str = r#"#!/bin/sh
# commit-audition commit-msg hook
# 由 `commit-audition hook install` 自动生成

commit-audition validate "$1"
"#;

/// 获取当前仓库的 .git/hooks 目录
fn get_hooks_dir() -> Option<PathBuf> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let git_dir = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Some(PathBuf::from(git_dir).join("hooks"))
}

/// 安装 commit-msg hook 到当前 git repo
pub fn install_hook() -> Result<String, String> {
    let hooks_dir = get_hooks_dir().ok_or("未找到 .git 目录")?;

    // 确保 hooks 目录存在
    std::fs::create_dir_all(&hooks_dir).map_err(|e| format!("创建 hooks 目录失败: {}", e))?;

    let hook_path = hooks_dir.join("commit-msg");

    // 检查是否已存在 hook
    if hook_path.exists() {
        let existing =
            std::fs::read_to_string(&hook_path).map_err(|e| format!("无法读取已有 hook: {}", e))?;

        if existing.contains("commit-audition") {
            // 是本工具生成的，无需重复安装
            return Ok("commit-msg hook 已安装，无需重复安装".to_string());
        }

        // 不是本工具生成的，报错
        return Err(format!(
            "已存在其他 commit-msg hook: {}\n请手动检查后重试",
            hook_path.display()
        ));
    }

    // 写入 hook 脚本
    std::fs::write(&hook_path, COMMIT_MSG_HOOK)
        .map_err(|e| format!("写入 commit-msg hook 失败: {}", e))?;

    // Unix: 设置可执行权限
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o755);
        std::fs::set_permissions(&hook_path, perms).map_err(|e| format!("无法设置权限: {}", e))?;
    }

    Ok(format!("commit-msg hook 已安装到: {}", hook_path.display()))
}

/// 卸载当前仓库的 commit-msg hook
pub fn uninstall_hook() -> Result<String, String> {
    let hooks_dir = get_hooks_dir().ok_or("当前目录不在 git 仓库中")?;

    let hook_path = hooks_dir.join("commit-msg");

    if !hook_path.exists() {
        return Err("commit-msg hook 不存在".into());
    }

    // 安全检查：只删除本工具生成的 hook
    let content =
        std::fs::read_to_string(&hook_path).map_err(|e| format!("无法读取 hook 文件: {}", e))?;

    if !content.contains("commit-audition") {
        return Err("commit-msg hook 不是由 commit-audition 生成的，请手动检查".into());
    }

    std::fs::remove_file(&hook_path).map_err(|e| format!("无法删除 hook 文件: {}", e))?;

    Ok("commit-msg hook 已卸载".into())
}
