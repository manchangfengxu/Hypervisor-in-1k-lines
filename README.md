## RISC-V Minimal Hypervisor

一个基于 RISC-V H 扩展的轻量级 Hypervisor，使用 Rust `no_std` 裸机环境实现。支持 vCPU 初始化、二阶段地址转换（VS‑stage）、Hypervisor Trap 处理，并能启动 Guest Linux 内核。

### 功能特性

- **裸机 Rust 运行时**：`#![no_std]` / `#![no_main]`，自定链接脚本与启动入口，直接运行在硬件或 QEMU 上
- **vCPU 管理**：维护完整的 Guest 寄存器上下文，配置 `hstatus`、`hgatp`、`hedeleg` 等虚拟化相关 CSR
- **二阶段页表**：建立 Guest Physical Address → Host Physical Address 的映射，基于 Sv48x4 格式
- **Guest 启动**：加载 Linux Image 与设备树，构造 Guest 入口上下文
- **Trap 路径**：实现 Hypervisor Trap 入口，完成 Guest/Host 模式切换的基本框架

### 构建与运行

#### 依赖

- Rust RISC-V bare‑metal target（`riscv64gc-unknown-none-elf`）
- LLVM 工具链（用于链接与 objcopy）
- QEMU（≥7.0，支持 RISC-V H 扩展）
- 预先准备好的 Guest Linux 镜像（`linux/Image`）

#### 运行

```sh
./run.sh
```

脚本将完成 Guest 构建、Hypervisor 编译，并在 QEMU 中启动。预期输出：

```text
Booting hypervisor...
start vcpu run
...
```