# 配置文件

配置文件路径：`~/.commit-audition/config.toml`

## 配置项一览

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `vim_mode` | `bool` | `false` | 是否启用 vim 模式（全屏 TUI 交互） |
| `editor.command` | `string?` | `null` | 自定义编辑器命令，如 `"code --wait"` |
| `editor.extension` | `string` | `"md"` | 临时文件的扩展名 |

## 示例

```toml
# 启用 vim 模式
vim_mode = true

[editor]
command = "code --wait"
extension = "md"
```

## 配置项详解

### `vim_mode`

设为 `true` 后，运行 `commit-audition` 将进入全屏 TUI 界面，使用 vim 风格快捷键操作。设为 `false` 或不配置时，使用默认的问答式交互。

### `editor.command`

自定义编辑器的启动命令。选择"自定义编辑器"模式时会使用此命令打开临时文件。

常用编辑器示例：

| 编辑器 | 命令 |
|--------|------|
| VS Code | `"code --wait"` |
| Vim | `"vim"` |
| Neovim | `"nvim"` |
| Helix | `"hx"` |
| Sublime Text | `"subl -w"` |

若不配置且选择了自定义编辑器模式，程序会提示错误并退出。

### `editor.extension`

临时文件的扩展名，影响编辑器的语法高亮。默认 `"md"`，也可设为 `"txt"` 等。
