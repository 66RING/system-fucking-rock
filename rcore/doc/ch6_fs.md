# 文件系统

## Pre

qemu中添加存储设备

```
-drive file=../user/target/riscv64gc-unknown-none-elf/release/fs.img,if=none,format=raw,id=x0 \
     -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0
```

- fs.img是包含程序的文件系统镜像
- 添加虚拟磁盘设备`virtio-blk-device`


## 简易文件系统

从内核中分离文件系统

**文件系统的本质是怎么管理block。怎么读写block是有磁盘驱动负责。**

- `easy-fs` crate，核心，管理文件系统磁盘布局
    * BlockDevice抽象来连接设备驱动, 即让软件软件知道怎么read/write某一block
    * 轮询访问`virtio_blk`
    * 避免直接访问进程相关的数据，从而可以独立出内核的进程管理
- `easy-fs-fuse`独立可执行程序，用于测试`easy-fs`和打包镜像

easy-fs结构

- 磁盘块设备接口层
- 块缓存层
- 磁盘数据结构层
- 磁盘块管理器层
- 索引节点

- Q: rcore中`DiskInode`, `Inode`, `OSInode`结构的区别
- A: DiskInode是dump到磁盘上的Inode结构。Inode是存内Inode。OSInode是rcore中起名的，应该是File结构

```
Inode
FileSystem
DiskInode | SuperBlock
CacheManager
BlockDevice
```

- Inode
    * 存内Inode
    * 与用户打交道, 是DiskInode的投影
- FileSystem
    * DiskInode的申请, 分配和位置查找
- DiskInode | SuperBlock
    * 磁盘数据结构
- CacheManager
    * 管理缓存, 读入/写回磁盘数据


### 块设备接口层

> `block_dev.rs`

抽象与底层硬件的接口, 通过统一的方式访问不同的底层硬件/硬件驱动

- Block Device Trait
    * read block
    * write block

### 磁盘布局

> layout.rs. 简易FAT. 相当于Memory dump
>
> 所谓的磁盘块号和内存地址是等价的。磁盘中的分配单位是块

```
| 超级块 | 索引节点bitmap | 索引节点区 | 数据块bitmap | 数据块区 |
```

- layout.rs
    * superblock: **磁盘上结构**, 记录连续区域中各个区域的块数，即可算出各自位置
    * 磁盘索引节点DiskInode: 对象都存储在了哪些块中
        + **Inode类型: file, dir**
        + **磁盘上的inode**, 最终会与系统中一个打开的文件绑定, rcore中就是`OSInode`, 不过我更愿意叫他`struct File`不如容易混淆
    * 数据块
        + 目录项: 另一个Inode的索引
    * abs: fs的元数据, 数据的索引, 数据本体
- bitmap.rs
    * **分块分组位图管理**
- bitmap管理: 
    * 分配: 遍历块空间中的每个块，遍历块中的每个bit查看分配情况
    * 释放: 通过bit的id计算出所在的块号，再计算出块内组(u64)偏移，最后计算组(u64)内偏移

- Q: 为什么layout的用到initialize接口?
- A: 因为我们是直接操作磁盘空间，相当于直接在一块内存上写: `*(fs*)addr = fs`。所以使用`initialize`这种原地修改的模式


#### Result

- Superblock是磁盘上结构, 可以直接通过"memcpy"的方式恢复到内存中
- **⭐位图管理的分块分组**
    * 分块: fs元数据位图本身也要遵循fs的操作逻辑
    * 分组: 不是一个bit一个bit的遍历, 而是通过一个u64是否MAX判断有没有空位
    * 充分利用block，bitmap与block大小一致, 每个bit又能管理一个block的分配情况
    * bitmap也是磁盘上的bitmap，需要通过缓存间接管理
- DiskInode记录了一个对象都存储在磁盘哪些块中
    * 间接索引表的长度: u32元素的个数
    * 添加文件/内容就相当于在多级索引表中添加数据
- Read/Write的抽象以Byte为单位的磁盘空间的读写
    * 因为每个文件都有自己的磁盘空间，即多级索引表表示的空间
    * 通过一个offset去所以该索引表
- 文件和目录在磁盘上的表示
    * 就是Inode, 通过一个标志来区分文件和目录，数据由多级索引表索引
    * 文件Inode的数据就是文件数据
    * 目录Inode的数据就是一个个的目录项: 另一个Inode的索引


### 块缓存层

> `block_cache.rs`

总是先过缓存在过磁盘, 所以实际与磁盘打交道的是缓存层。因此需要考虑各种block操作。然而用户能直接访问到的是缓存管理器。

- 单个缓存本身
    * 缓存应包含: 缓存数据, 对应磁盘数据, 磁盘读写方法, dirty标志
    * 在缓存看来所有数据都是`void*`，所以对于用户要对写的对象还会涉及类型转换
- 缓存管理
    * 简单实现FIFO, 或者参考DB的visitor pattern的实现


#### Result

- 缓存包括: 单个缓存本身 + 缓存管理器
- 单个缓存看到的数据都是`void*`, 可以给用户提供好用的helper函数。感谢泛型


### 磁盘块管理器

> efs.rs

即内存中的文件系统。之所以叫块管理器是因为只有文件系统能真正操作和修改磁盘。

- 0号块作为超级块
    * **从磁盘中读取设备信息以构造文件系统实例**
- 文件系统实例创建: `open(BlockDevice)`
    1. 从超级块中dump出各个区域的数据
    2. 通过底层磁盘驱动创建文件系统: 文件系统要知道怎么read/write磁盘
    3. 创建各个区域的管理器对象: Bitmap
    4. 赋值各个区域的地址
- TIPS: `EasyFileSystem::create()`是为了用户态创建文件系统用的


#### Result

- 超级块是**存外**存储的信息。fs是**存内**存储的信息通过读取超级块实现
- FS本质就是"磁盘管理器"在内存空间的映射
- FS记录的各个区域的地址，索引时既可以通过start + offset查找数据
    * 如数据就应该在数据区: `data_area_start_block + data_block_id`, 同理inode
- abs: 
    * 对磁盘`void*`的管理 = bitmap标记的设置 + 实际空间的分配
    * 方便的helper函数获取各个区域的数据
    * **一个根目录通往世界各地**, 所以提供一个访问根目录inode的接口即可


#### 索引节点

> vfs.rs

Inode结构, **内存中的Inode**, 本质就是DiskInode在内中的投影。通过`block_id`访问到DiskInode。因为其是存内结构，需要通过缓存层才能访问到DiskInode。做了一次抽象，否则用户就得直接访问缓存层然后访问DiskInode。


## 用户态测试

> easy-fs-fuse

用户态创建文件系统镜像


## 内核中接入文件系统

TODO:

- read/write抽象
    * 抽象 -> trait
- 添加系统调用
    * `open`, 打开文件，返回文件描述符。文件描述符用于所以内部文件表
    * `close`
- 文件读写
    * 修改read/write系统调用，使其更加通用以至于能支持文件的顺序读写
    * 添加seek系统调用修改读写游标


### 块设备驱动层

> os/drivers
>
> 虚拟磁盘设备virtio-blk-device

- 使用具体的硬件驱动访问底层块设备，对外暴露BlockDevice trait从而能够接入文件系统

- 通过MMIO访问qemu的virtio外设总线
- 添加MMIO区域的内核态映射, 从而在开启虚拟内存后也能访问

- 使用现成的virtio-drivers crate来使用块设备，网卡，GPU等设备
- 实现virtio-driver要求的trait以实现物理内存的管理从而可以让crate分配VirtQueue


#### Result

- MMIO就相当一段硬件寄存器, 只不过是可以通过内存访问，也可以通过dump成结构体访问
- virtio驱动给用户暴露了方便的接口，只要求用户能够管理好连续的物理内存即可


### 文件描述符层

> os/src/fs/inode.rs

- OSInode, 本质就是进程中打开的文件, 即File结构，只是rcore里叫他OSInode
    * 内部关联到内存中的Inode, 内存中Inode再关联到磁盘DiskInode这才真正拿到数据
- 接口
    * 创建一个"File结构", 即OSInode
    * 读出一段，写入一段
- 进程结构体
    * 打开文件表
    * 初始化进程时默认创建三个stdio的fd
    * fork时记得fork文件表
        + a fork in the road
- 最简单的单层根目录交互: list, open, read, write
    * read, write统一从打开表获取实现的`File trait`的对象然后读写
    * open: 申请fd, 填入打开文件表
        + 申请fd就是遍历表找第一个空的项
        + TODO: Q: rust中可以用trait如`pub fd_table: Vec<Option<Arc<dyn File + Send + Sync>>>`, c中怎么办
- 添加相关系统调用: open, close
- 修改read, write系统调用使更加通用
    * `stdio.rs`, 结合read, write系统调用分析
    * 为stdio实现`File trait`从而可以用统一的read，write接口实现读写
    * 创建基于fd的stdio: 每次r/w时都从打开文件表中获取到File结构然后读写
- 基于文件系统加载应用: `sys_exec`
    * 直接读入文件的所有内容到内存中, 创建task
- 基于文件系统打开初始进程
    * 直接读入文件的所有内容到内存中, 创建task


## Result

- DiskInode vs Inode vs OSInode
- DiskFs vs Fs
    * 磁盘上的文件系统对象: 超级块
    * 内存中的文件系统对象: 从超级块中读取进去的数据
