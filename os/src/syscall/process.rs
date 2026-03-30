//! Process management syscalls
use crate::{
    task::{exit_current_and_run_next, suspend_current_and_run_next,TASK_MANAGER},
    timer::get_time_us,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// get time with second and microsecond
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    unsafe {
        *ts = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}


// TODO: implement the syscall
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    match trace_request {
        // 功能 0：读取内存一个字节
        0 => {
            unsafe {
                (id as *const u8).read() as isize
            }
        }
        // 功能 1：写入内存一个字节
        1 => {
            unsafe {
                (id as *mut u8).write(data as u8);
            }
            0
        }
        // 功能 2：查询当前任务调用编号为 id 的系统调用的次数
        2 => {
            let manager = TASK_MANAGER.inner.exclusive_access();
            let current_id = manager.current_task;
                // 直接返回即可，因为进入 syscall 函数时已经 +1 了
                // 这符合题目要求：“本次调用也计入统计”
                manager.tasks[current_id].syscall_counts[id] as isize
            
        }
        _ => -1,
    }
}
/*获取任务信息
在 ch3 中，我们的系统已经能够支持多个任务分时轮流运行，我们希望引入一个新的系统调用 ``sys_trace``（ID 为 410）用来追踪当前任务系统调用的历史信息，并做对应的修改。定义如下。

fn sys_trace(_trace_request: usize, _id: usize, _data: usize) -> isize
调用规范：
这个系统调用有三种功能，根据 trace_request 的值不同，执行不同的操作：

如果 trace_request 为 0，则 id 应被视作 *const u8 ，表示读取当前任务 id 地址处一个字节的无符号整数值。此时应忽略 data 参数。返回值为 id 地址处的值。

如果 trace_request 为 1，则 id 应被视作 *mut u8 ，表示写入 data （作为 u8，即只考虑最低位的一个字节）到该用户程序 id 地址处。返回值应为0。

如果 trace_request 为 2，表示查询当前任务调用编号为 id 的系统调用的次数，返回值为这个调用次数。本次调用也计入统计 。

否则，忽略其他参数，返回值为 -1。

说明：
你可能会注意到，这个调用的读写并不安全，使用不当可能导致崩溃。这是因为在下一章节实现地址空间之前，系统中缺乏隔离机制。所以我们 不要求你实现安全检查机制，只需通过测试用例即可 。

你还可能注意到，这个系统调用读写本任务内存的功能并不是很有用。这是因为作业的灵感来源 syscall 主要依靠 trace 功能追踪其他任务的信息，但在本章节我们还没有进程、线程等概念，所以简化了操作，只要求追踪自身的信息。

 */