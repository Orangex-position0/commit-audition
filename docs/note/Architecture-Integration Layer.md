# Integration Layer

Integration 层是四层架构中的副作用边界，负责隔离所有外部依赖的交互（文件 IO、git 命令调用、网络请求等），确保 Logic 层保持纯函数特性。

## 当前状态：空层

`src/integration.rs` 当前为空文件，仅声明为模块。这是有意为之的架构决策，并非遗漏。

## 设计意图

在经典分层架构中，Integration 层（也称 Infrastructure 层）承担以下职责：

- **文件系统操作**：读写配置文件、管理临时文件
- **外部命令调用**：执行 git 命令、调用编辑器进程
- **网络请求**：远程 API 调用（如 Issue 关联查询）
- **环境检测**：操作系统特性、环境变量读取

将这些操作隔离在本层，使得：
1. Logic 层可以保持纯函数特性，便于单元测试
2. 外部依赖的替换（如从文件系统切换到数据库）不影响业务逻辑
3. 副作用边界明确，便于理解和维护

## 未来承载的职责

以下功能可从当前 Logic 层迁移至 Integration 层：

| 功能 | 当前位置 | 迁移后职责 |
|---|---|---|
| 配置文件读取 | `logic::config::load_config()` | `integration::config_io::read_config()` |
| Hook 文件操作 | `logic::hook::install_hook()` / `uninstall_hook()` | `integration::hook_io::write_hook()` / `remove_hook()` |
| Git 命令调用 | `logic::hook::get_hooks_dir()` | `integration::git_io::get_hooks_dir()` |
| 临时文件管理 | `ui::editor::edit_custom_editor()` | `integration::file_io::create_temp_file()` |

## 迁移条件

当代码满足以下任一条件时，应考虑将 IO 操作迁移至 Integration 层：

1. **依赖倒置需要**：Logic 层需要对 IO 操作进行 mock/替换（如测试场景）
2. **多 IO 源**：同一类操作需要支持多种实现（如配置从文件读取改为从数据库读取）
3. **复杂度增长**：IO 操作的逻辑变得复杂，开始侵占 Logic 层的纯业务逻辑空间

迁移方式：在 Logic 层定义 trait 接口，在 Integration 层提供具体实现，通过依赖注入传入 Logic 层。
