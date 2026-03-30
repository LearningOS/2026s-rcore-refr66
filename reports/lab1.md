各个文件的作用，

一个sys_trace的调用流程

  

1. 正确进入 U 态后，程序的特征还应有：使用 S 态特权指令，访问 S 态寄存器后会报错。 请同学们可以自行测试这些内容（运行 [三个 bad 测例 (ch2b_bad_*.rs)](https://github.com/LearningOS/rCore-Tutorial-Test/tree/master/src/bin) ）， 描述程序出错行为，同时注意注明你使用的 sbi 及其版本。

在 RustSBI 0.3.0-alpha.2 RISC-V SBI v1.0.0 环境下，
 ch2b_bad_address 
 PageFault in application, bad addr = 0x0, bad instruction = 0x804003a4, kernel killed it.
 
ch2b_bad_instructions
 IllegalInstruction in application, kernel killed it.
 
ch2b_bad_register
 IllegalInstruction in application, kernel killed it.
。

2. 深入理解 [trap.S](https://github.com/LearningOS/rCore-Tutorial-Code/blob/ch3/os/src/trap/trap.S) 中两个函数 `__alltraps` 和 `__restore` 的作用，并回答如下问题:
	1. L40：刚进入 __restore 时，sp 代表了什么值？请指出 __restore 的两种使用情景。
		sp 的值：此时 sp 指向 内核栈，并且具体指向该栈上的一个 TrapContext 结构的起始地址（即栈顶）。
		
		两种使用情景：
		从异常处理返回：当用户程序触发了系统调用（syscall）或发生异常，进入内核处理完成后，调用 restore 恢复用户态上下文，继续执行用户程序。
		启动第一个应用：在内核初始化完成后，为了运行第一个用户程序，内核会手动在内核栈上构造一个初始的 TrapContext，然后伪造一个“从异常返回”的过程，通过调用 __restore 进入该程序。
		
	2. . L43-L48：这几行汇编代码特殊处理了哪些寄存器？这些寄存器的值对于进入用户态有何意义？
		```
		ld t0, 32*8(sp)
		ld t1, 33*8(sp)
		ld t2, 2*8(sp)
		csrw sstatus, t0
		csrw sepc, t1
		csrw sscratch, t2
		```

		特殊处理的寄存器：sstatus、sepc 和 sscratch。
		sstatus (Supervisor Status)：控制 CPU 的状态。其中最关键的是 SPP 位（Previous Privilege），它记录了进入 Trap 前的特权级。sret 指令会根据此位决定返回到 U 态还是 S 态。此外还包含中断使能等配置。
		sepc (Supervisor Exception Program Counter)：记录了 Trap 发生时那一跳指令的地址。执行 sret 时，硬件会自动将 PC 指向 sepc 的值。对于系统调用，内核会将其修改为 ecall 的下一条指令地址，从而保证程序正常向下执行。
		sscratch：在此代码段中，从 TrapContext 里取出了 用户栈指针 并写入 sscratch。这是为了在最后一步通过 csrrw 交换，将 sp 切换回用户栈。

3. L50-L56：为何跳过了 x2 和 x4？
	跳过 x2 (sp)：x2 是栈指针。我们不能在这里直接 ld x2, 2*8(sp)，因为当前的 sp 正在指向内核栈以读取数据。如果直接覆盖 x2，我们就丢失了内核栈的索引，无法继续读取后续寄存器。sp 的恢复被延迟到了 sscratch 交换的那一步。
	跳过 x4 (tp)：tp 是线程指针（Thread Pointer）。在当前的 rCore 实现（简化的批处理系统）中，用户态程序不使用 tp 寄存器，或者内核将其保留自用且不希望用户态程序修改/读取它。因此无需保存和恢复。

4. L60：该指令之后，sp 和 sscratch 中的值分别有什么意义？
```
csrrw sp, sscratch, sp
```
sp：现在指向 用户栈。程序现在已经准备好回到用户态执行，且栈已经切换回去了。
sscratch：现在指向 内核栈。当下次再发生 Trap（进入 __alltraps）时，我们可以通过 sscratch 找到内核栈的位置。

5. __restore 中发生状态切换在哪一条指令？为何该指令执行之后会进入用户态？
sret,sret 是 RISC-V 定义的特权指令，硬件在执行它时会完成以下原子操作：
将当前的特权级设置为 sstatus.SPP 字段的值（如果之前是从 U 态进来的，此时就会变回 U 态）。
将程序计数器 PC 设置为 sepc 的值。
将 sstatus.SIE 设置为 sstatus.SPIE（恢复中断状态）。

6. L13：该指令之后，sp 和 sscratch 中的值分别有什么意义？
```
csrrw sp, sscratch, sp
```

sp：现在指向 内核栈。这使得后续的指令（如 addi sp, sp, -34*8）可以在内核栈上开辟空间保存上下文。
sscratch：现在指向 用户栈。后续指令会将这个值从 sscratch 读出并存入 TrapContext 中，以便将来恢复。

7. 从 U 态进入 S 态是哪一条指令发生的？

在 trap.S 中并不存在这条指令。状态切换是由 硬件 在用户态执行以下操作时自动触发的：
ecall 指令：执行环境调用（系统调用）。
硬件中断：如时钟中断。
异常：如非法指令、访存故障。
过程：当上述事件发生时，CPU 硬件会自动将特权级从 U 模式提升到 S 模式，跳转到 stvec 寄存器所指向的地址（即 __alltraps 的起始位置），并自动设置 scause、stval、sepc 和 sstatus 等寄存器。