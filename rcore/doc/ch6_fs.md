# 文件系统

## Pre

qemu中添加存储设备

```
-drive file=../user/target/riscv64gc-unknown-none-elf/release/fs.img,if=none,format=raw,id=x0 \
     -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0
```

- fs.img是包含程序的文件系统镜像
- 添加虚拟磁盘设备`virtio-blk-device`


## 文件和目录抽象

- 添加系统调用
    * `open`, 打开文件，返回文件描述符。文件描述符用于所以内部文件表
    * `close`
- 文件读写
    * 修改read/write系统调用，使其更加通用以至于能支持文件的顺序读写
    * 添加seek系统调用修改读写游标


## 简易文件系统

从内核中分离文件系统

- `easy-fs` crate，核心，管理文件系统磁盘布局
    * TODO: 什么是BlockDevice抽象来连接设备驱动
    * 轮询访问`virtio_blk`
    * 避免直接访问进程相关的数据，从而可以独立出内核的进程管理
- `easy-fs-fuse`独立可执行程序，用于测试`easy-fs`和打包镜像

easy-fs结构

- 磁盘块设备接口层
- 块缓存层
- 磁盘数据结构层
- 磁盘块管理器层
- 索引节点


### 块设备接口层

> `block_dev.rs`

- Block Device Trait
    * read block
    * write block


### 块缓存层

> `block_cache.rs`

磁盘块读到内存缓存中，结合缓存管理策略管理块缓存

TODO: revise


### 磁盘布局

> FAT. Memory dump

```
| 超级块 | 索引节点bitmap | 索引节点区 | 数据块bitmap | 数据块区 |
```

- layout.rs
    * superblock: 记录连续区域中各个区域的块数，即可算出各自位置
    * 磁盘索引节点DiskInode: 对象都存储在了哪些块中
        + **Inode类型: file, dir**
    * 数据块
        + 目录项: 分级处理?? TODO: abs + review
    * abs: fs的元数据, 数据的索引, 数据本体
- bitmap.rs
    * **分块分组位图管理**

TODO: 为什么layout的用到initialize接口?


#### Result

- **⭐位图管理的分块分组**
    * 分块: fs元数据位图本身也要遵循fs的操作逻辑
    * 分组: 不是一个bit一个bit的遍历, 而是通过一个u64是否MAX判断有没有空位
- DiskInode记录了一个对象都存储在哪些块中
    * 间接索引表的长度: u32元素的个数


### 磁盘块管理器

> efs.rs

- 0号块作为超级块
    * 从磁盘中读取设备信息以构造文件系统实例


#### Result

- 超级块是存外存储的信息。fs是存内存储的信息通过读取超级块实现


#### 索引节点

> vfs.rs


## 用户态测试


## 虚拟磁盘设备virtio-blk-device

## Result

- DiskInode vs Inode
- DiskFs vs Fs
    * 磁盘上的文件系统对象: 超级块
    * 内存中的文件系统对象: 从超级块中读取进去的数据
