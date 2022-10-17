use riscv::register::time;
use crate::config::CLOCK_FREQ;
use crate::sbi::set_timer;

const TICKS_PER_SEC: usize = 100;
const MICRO_PER_SEC: usize = 1_000_000;

// 获取mtime计数器的值
pub fn get_time() -> usize {
    time::read()
}

// 设置10ms后触发时钟中断
// 不必担心mtime溢出，当一直递增的CSR使用
pub fn set_next_trigger() {
    // CLOCK_FREQ(Hz)根据各平台的频率而定
    //  CLOCK_FREQ / TICKS_PER_SEC : 下一次中断计数器的增量
    set_timer(get_time() + CLOCK_FREQ / TICKS_PER_SEC);
}

// 返回计数器当前值，单位为微秒
pub fn get_time_ms() -> usize {
    time::read() / (CLOCK_FREQ / MICRO_PER_SEC)
}









