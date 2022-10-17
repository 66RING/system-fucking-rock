# TODO:
# 为每个app定制链接脚本
# 修改所有的base_address相关的
import os

base_address = 0x80400000
step = 0x20000
linker = 'src/linker.ld'

app_id = 0
apps = os.listdir('src/bin')
apps.sort()

for app in apps:
    app = app[:app.find('.')]
    lines = []
    lines_before = []
    with open(linker, 'r') as f:
        # 找到linker.ld中 BASE_ADDRESS = 0x80400000的行
        for line in f.readlines():
            # 保存旧linker.ld
            lines_before.append(line)
            line = line.replace(hex(base_address), hex(base_address+step*app_id))
            lines.append(line)
    with open(linker, 'w+') as f:
        f.writelines(lines)
    # 构建当前应用, --bin构建单个
    os.system('cargo build --bin %s --release' % app)
    print('[build.py] application %s start with address %s' %(app, hex(base_address+step*app_id)))
    # 还原linker.ld
    with open(linker, 'w+') as f:
        f.writelines(lines_before)
    app_id = app_id + 1
