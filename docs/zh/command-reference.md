# 命令覆盖

## 命令树是怎么被发现的

PhyloPanel 内部没有一份 GoTree 命令清单。首次使用时它会*反射*目标二进制：

1. 不带参数运行工具，读出 `Available Commands` 各段。
2. 递归进入每个命令组。
3. 对每个候选命令运行 `<tool> <path> --help`，解析 usage 行、参数块和简短说明。
4. 把命令分类为叶子、命令组，或**双重**（既是命令又是组——GoTree 的 `cut`、`resolve`、`stats`
   能自己运行，也有子命令）。
5. 根据占位符给参数定类型（`--height int` → 整数、`--support float`、`--labels stringSlice`、
   光秃秃的 `--circular` → 布尔），并从帮助文本里抽出默认值。
6. 给每个参数指派语义角色——`input`、`output`、`format`、`file`、`other`——角色决定了是否显示
   文件选择器，以及 `--input` 的自动注入。

之所以要引入"角色"，是因为命令行工具自己就不统一：`-i/--input` 在 `draw svg` 上属于全局参数，
在 `acr` 上属于私有参数；`reroot outgroup` 写 `--tip-file`，`prune` 却写 `--tipfile`。按*含义*
而不是按拼写来组织界面，才让一套表单构建器能同时服务多个工具。

## 缓存

反射意味着大约"每个命令一个进程"，所以结果会以 JSON 缓存，键是二进制文件名**加上它的修改时间**。
替换命令行工具后缓存键自然对不上，下次启动就会自动重新反射。

顶栏的**重新反射命令表**会强制重新反射并重写缓存。如果你是原地覆盖二进制、而文件的修改时间戳
没变（某些拷贝工具会保留原时间戳），就用这个按钮。

## GoTree 0.5.2 到底有多少命令

35 个顶层命令组、82 个可执行命令、213 个命令私有参数。下面的数量和说明取自反射出来的 schema，
不是取自上游文档。

| 命令组 | 命令数 | 说明 |
|---|---|---|
| `acr` | 1 | 最大简约法重建祖先性状 |
| `annotate` | 1 | 用给定数据标注内部分支 |
| `asr` | 1 | 最大简约法重建祖先序列 |
| `brlen` | 8 | 分支长度运算：add、clear、cut、round、scale、set、setmin、setrand |
| `collapse` | 6 | 按分类单元、支持率、深度、长度、名称、单孩子合并 |
| `comment` | 2 | 增删节点/分类单元/边的注释 |
| `compare` | 4 | edges、neighborhood、tips、trees 四种比较 |
| `compute` | 7 | bipartitiontree、consensus、edgetrees、mutations、roccurve、`support fbp`、`support tbe` |
| `cut` | 2 | 剪切树（双重命令，自己也能跑） |
| `divide` | 1 | 把输入树文件拆成多个文件 |
| `download` | 3 | itol、ncbitax、panther |
| `draw` | 4 | `draw cyjs`、`draw png`、`draw svg`、`draw text` |
| `generate` | 6 | balancedtree、caterpillartree、startree、topologies、uniformtree、yuletree |
| `graft` | 1 | 把一棵树接到另一棵树的指定分类单元上 |
| `labels` | 1 | 列出全部分类单元标签 |
| `ltt` | 1 | 计算 Lineage-Through-Time 数据 |
| `matrix` | 1 | 输出距离矩阵 |
| `merge` | 1 | 用新根合并两棵有根树 |
| `nni` | 1 | 生成全部 NNI 邻居 |
| `prune` | 1 | 移除分类单元（需要位置参数） |
| `reformat` | 3 | newick、nexus、phyloxml |
| `rename` | 1 | 重命名节点或分类单元 |
| `repopulate` | 1 | 把序列相同的分类单元补回树中 |
| `reroot` | 2 | `reroot midpoint`、`reroot outgroup` |
| `resolve` | 2 | 用 0 长度分支解开多叉节点（双重命令） |
| `rotate` | 2 | rand、sort 两种内部节点子序调整 |
| `rtt` | 1 | 根到tip 回归 |
| `sample` | 1 | 从文件里抽取部分树 |
| `shuffletips` | 1 | 打乱分类单元名 |
| `stats` | 7 | 树的统计信息，含 `stats monophyletic`（双重命令） |
| `subtree` | 1 | 按节点名取子树 |
| `support` | 4 | clear、round、scale、setrand |
| `unroot` | 1 | 去根 |
| `upload` | 1 | `upload itol`（需要位置参数） |
| `version` | 1 | 打印工具版本 |

两个容易踩的文档坑：`monophyletic` 挂在 `stats` **下面**（`gotree stats monophyletic`），
以及上表标注的四个命令读取裸参数，而 `--help` 从不提这件事。

## 驱动 GoAlign 或任意其他 cobra 工具

把**可执行文件**指向 `goalign.exe`，同一套反射就会产出 GoAlign 面板。机制是通用的，工具特有的
部分放在*工具包*里。

### 工具包

`src-tauri/toolpacks/<name>.json` 按可执行文件文件名匹配加载，负责补上反射拿不到的信息：

| 字段 | 用途 |
|---|---|
| `displayName` | 状态栏里显示的工具名 |
| `positional` | 每个命令裸参数的标签与帮助，含 `labelZh`/`helpZh` 供中文界面使用 |
| `templates` | 预设工作流，含 `name`/`nameZh` 与 `description`/`descriptionZh` |
| `imageCommands` | 哪些命令返回的是图片而不是树或表格 |

陌生的二进制照样能完整使用——所有命令和参数都会被反射出来——只是没有预设和位置参数提示。
补齐这些只需要改 JSON，不需要改代码。

### 给新工具加工具包

1. 新建 `src-tauri/toolpacks/<tool>.json`。
2. 在 `src-tauri/src/toolpack.rs` 的 `pack_for()` 里按文件名词根加载它，和 `gotree.json` 一样。
3. 重新构建。命令由反射提供，人工标注由工具包提供。

## 版本漂移

因为命令树是运行时从二进制读出来的，上游升级后面板无需任何改动即可看到新命令。如果某次升级改了
命令名而某个预设还在用它，面板会明确告诉你缺的是哪个命令，而不是跑一条坏掉的链。
