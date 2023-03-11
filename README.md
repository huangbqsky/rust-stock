# rust-stock

Terminal Stock Query

A simple terminal tool for stock query written in Rust 🦀

使用 Rust 开发的股价查询终端应用

![](images/screen.gif)


### 主要使用tui-rs 一款超好用的跨平台命令行界面库

```
使用 tui.rs 提供的以下模块进行 UI 编写(所有 UI 元素都实现了 Widget 或 StatefuWidget Trait)：

bakend 用于生成管理命令行的后端
layout 用于管理 UI 组件的布局
style 用于为 UI 添加样式
symbols 描述绘制散点图时所用点的样式
text 用于描述带样式的文本
widgets 包含预定义的 UI 组件

项目地址：https://github.com/fdehau/tui-rs
官方文档：https://docs.rs/tui/latest/tui/index.html

tui介绍：https://www.51cto.com/article/703696.html
实时股票数据： https://github.com/tarkah/tickrs
文件传输工具：https://github.com/veeso/termscp
网络监控工具：https://github.com/imsnif/bandwhich

```

# 编译/运行

```
cargo build 
cargo run
```


