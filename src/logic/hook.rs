use rust_i18n::t;
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
    let hooks_dir = get_hooks_dir().ok_or(t!("hook.no_git_dir").to_string())?;

    // 确保 hooks 目录存在
    std::fs::create_dir_all(&hooks_dir)
        .map_err(|e| t!("hook.create_hooks_failed", error = e.to_string()).to_string())?;

    let hook_path = hooks_dir.join("commit-msg");

    // 检查是否已存在 hook
    if hook_path.exists() {
        let existing = std::fs::read_to_string(&hook_path)
            .map_err(|e| t!("hook.read_hook_failed", error = e.to_string()).to_string())?;

        if existing.contains("commit-audition") {
            // 是本工具生成的，无需重复安装
            return Ok(t!("hook.already_installed").to_string());
        }

        // 不是本工具生成的，报错
        return Err(t!(
            "hook.other_hook_exists",
            path = hook_path.display().to_string()
        )
        .to_string());
    }

    // 写入 hook 脚本
    std::fs::write(&hook_path, COMMIT_MSG_HOOK)
        .map_err(|e| t!("hook.write_hook_failed", error = e.to_string()).to_string())?;

    // Unix: 设置可执行权限
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o755);
        std::fs::set_permissions(&hook_path, perms)
            .map_err(|e| t!("hook.set_perms_failed", error = e.to_string()).to_string())?;
    }

    Ok(t!("hook.installed", path = hook_path.display().to_string()).to_string())
}

/// 卸载当前仓库的 commit-msg hook
pub fn uninstall_hook() -> Result<String, String> {
    let hooks_dir = get_hooks_dir().ok_or(t!("hook.not_git_repo").to_string())?;

    let hook_path = hooks_dir.join("commit-msg");

    if !hook_path.exists() {
        return Err(t!("hook.hook_not_found").to_string());
    }

    // 安全检查：只删除本工具生成的 hook
    let content = std::fs::read_to_string(&hook_path)
        .map_err(|e| t!("hook.read_hook_failed_uninstall", error = e.to_string()).to_string())?;

    if !content.contains("commit-audition") {
        return Err(t!("hook.not_our_hook").to_string());
    }

    std::fs::remove_file(&hook_path)
        .map_err(|e| t!("hook.remove_failed", error = e.to_string()).to_string())?;

    Ok(t!("hook.uninstalled").to_string())
}
