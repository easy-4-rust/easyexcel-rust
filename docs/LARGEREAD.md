# 大文件读取与共享字符串缓存

XLSX 工作表由 `easyexcel-xlsx` 以 SAX 事件流读取，不会先把整张工作表加载到内存。XLSX 的 `sharedStrings.xml` 仍可能很大，因此缓存后端必须与工作表事件流分开选择。

## 默认策略

`ReadCacheMode::Auto` 根据 `sharedStrings.xml` 的未压缩大小选择后端：

- 小于 5 MB：`ReadCacheMode::Memory`，使用顺序内存缓存；
- 达到或超过 5 MB：`ReadCacheMode::File`，使用临时文件和索引，保持大文件 SAX 读取的内存边界；
- 工作簿读取结束后，临时文件随缓存生命周期自动释放。

工作表 XML 始终按事件流处理。切换共享字符串缓存不会把 SAX 读取改成整本工作簿加载。

## Moka 对象缓存

`ReadCacheMode::Moka` 是显式选择的对象缓存：

- 使用 `moka::sync::Cache` 保存解码后的共享字符串对象；
- 不设置容量上限、TTL 或 TTI，读取过程中不淘汰条目；
- `put_finished` 后继续使用同一缓存读取；
- `destroy` 或缓存对象销毁时整体释放。

Moka 模式适合共享字符串规模可控、希望避免文件随机读取的场景。它不是默认的大文件策略，因为全量对象驻留会随共享字符串数量增长。

## 显式选择

```rust
use easyexcel::ReadCacheMode;

// 大文件、可预测内存占用：
let file_mode = ReadCacheMode::File;

// 数据规模可控、优先对象读取性能：
let moka_mode = ReadCacheMode::Moka;
```

也可以使用 `SimpleReadCacheSelector::with_max_use_map_cache_size_mb` 调整 `Auto` 从内存缓存切换到文件缓存的阈值。Moka 不提供运行过程中的容量淘汰参数。
