## 当前模拟器的 Chaining 机制

---

## 1. 核心设计理念

### 1.1 数据结构设计

**`register.rs`文件中定义的队列式的任务管理系统**:

```rust
pub struct VectorRegister {
    pub id: RegisterIdType,
    pub total_bytes: u32,
    pub write_count: u32,
    pub read_count: u32,
    pub current_index: usize,
    pub task_queue: VecDeque<RegisterTask>  // 关键!
}
```

**`task_queue` 是 chaining 的核心**:

- 每个向量寄存器维护一个任务队列
- 队列中包含多个读/写任务
- 任务从**前端添加** (`push_front`),从**后端移除** (`pop_back`)
- 这形成了一个**生产者-消费者链**

### 1.2 `task.rs` 文件中的RegisterTask 结构

```rust
pub struct RegisterTask {
    pub current_place: u32,      // 当前处理进度(字节)
    pub resource_index: usize,   // 对应的输入缓冲区索引
    pub behavior: UnitBehavior,  // Read 或 Write
    pub unit_key: UnitKeyType    // 关联的功能单元
}
```

这个设计允许:

- **追踪每个任务的进度** (`current_place`)
- **支持多个依赖指令** (队列中可以有多个任务)
- **区分读写行为** (Read/Write)

---

## 2. Chaining 的具体实现

### 2.1 元素级/Chunk 级转发

**关键函数 register.rs 文件中的**:  `VectorRegister::handle_one_task()`

```rust
//此处只截取函数部分关键内容
pub fn handle_one_task(&self, index: usize) -> Option<BufferEvent> {
    let forward_bytes = SimulatorConfig::get_global_config()
        .unwrap()
        .get_maximum_forward_bytes()  // 配置项: maximum_forward_bytes = 32
        .min(self.get_total_bytes() - q[index].current_place);
  
    match index == q.len() - 1 {
        true => {
            // 最后一个任务: 可以转发 forward_bytes
            Some(q[index].generate_event(forward_bytes))
        },
        false => {
            // 中间任务: 只能转发到下一个任务的当前位置
            let mut update_length = q[index+1].current_place - q[index].current_place;
            update_length = update_length.min(forward_bytes);
  
            if update_length == 0 {
                None  // 等待下一个任务消费更多数据
            } else {
                Some(q[index].generate_event(update_length))
            }
        }
    }
}
```

1. **最后一个任务**(生产者): 可以自由地按 `maximum_forward_bytes` 转发数据
2. **中间任务**(消费者): 只能转发已经被后续任务消费的数据
3. **转发粒度**: 32 字节/周期 (可配置)

### 2.2 流水重叠执行

**关键机制 `function_unit.rs`文件中的**: `EventGenerator` 和 `generate_next_event()`函数

```rust
pub struct EventGenerator {
    func_inst: FuncInst,
    cycle_per_event: u32,
    bytes_per_event: u32,      // 每次处理的字节数
    total_bytes: u32,
    processed_bytes: u32,
}
```

```rust
pub fn generate_next_event(&mut self, current_bytes: u32) -> Option<Event> {
    let bytes_this_event = self.bytes_per_event
        .min(self.total_bytes - self.processed_bytes)
        .min(current_bytes - self.processed_bytes);  // 关键!
  
    if bytes_this_event == 0 {
        return None;  // 等待更多输入数据
    }
  
    Some(Event {
        remained_cycle: self.cycle_per_event,
        target_register: self.func_inst.destination.clone(),
        result_bytes: bytes_this_event,
    })
}
```

- `current_bytes`: 输入缓冲区中已就绪的字节数
- 只有当 `current_bytes > processed_bytes` 时才生成新事件
- **不需要等待整个向量完成**,有多少数据处理多少

### 2.3 依赖管理

**在 `register.rs`文件的 `RegisterFile::add_vector_task()` 中**:

```rust
pub fn add_vector_task(&mut self, func_inst: &FuncInst) {
    // 为源寄存器添加读任务
    func_inst.resource.iter().enumerate().for_each(|(i, r)| {
        match r {
            RegisterType::VectorRegister(id) => {
                self.vector_registers[*id as usize]
                    .task_queue_mut()
                    .push_front(RegisterTask::new(i, UnitBehavior::Read, unit_key.clone()));
                self.vector_registers[*id as usize].increase_read_count();
            },
            _ => {}
        }
    });
  
    // 为目标寄存器添加写任务
    match &func_inst.destination {
        RegisterType::VectorRegister(id) => {
            self.vector_registers[*id as usize]
                .task_queue_mut()
                .push_front(RegisterTask::new(0, UnitBehavior::Write, unit_key.clone()));
            self.vector_registers[*id as usize].increase_write_count();
        },
        _ => {}
    }
}
```

**依赖链的建立**:

```
vadd.vv v1, v2, v3  ->  v1.task_queue = [Write(vadd)]
                        v2.task_queue = [Read(vadd)]
                        v3.task_queue = [Read(vadd)]

vmul.vv v4, v1, v5  ->  v1.task_queue = [Read(vmul), Write(vadd)]  // 链接!
                        v4.task_queue = [Write(vmul)]
                        v5.task_queue = [Read(vmul)]
```

---

## 3. 执行流程示例

假设执行:

```assembly
vadd.vv v1, v2, v3    # v1 = v2 + v3, 每个向量 256 字节
vmul.vv v4, v1, v5    # v4 = v1 * v5
```

### 时间线 (maximum_forward_bytes = 32):

| Cycle | vadd 进度 | v1 写入 | vmul 进度 | v1 读取 | 说明                            |
| ----- | --------- | ------- | --------- | ------- | ------------------------------- |
| 0     | 0/256     | 0/256   | -         | -       | vadd 开始                       |
| 1     | 32/256    | 32/256  | -         | -       | vadd 生成第一个 event           |
| 2     | 64/256    | 64/256  | -         | -       | 继续处理                        |
| 3     | 96/256    | 96/256  | 0/256     | 0/96    | **vmul 开始! (chaining)** |
| 4     | 128/256   | 128/256 | 32/256    | 32/128  | **流水重叠**              |
| 5     | 160/256   | 160/256 | 64/256    | 64/160  | 继续重叠                        |
| ...   | ...       | ...     | ...       | ...     | ...                             |

**关键观察**:

- **Cycle 3**: vmul 不需要等 vadd 完成全部 256 字节
- **Cycle 4-5**: vadd 和 vmul **同时执行** (流水重叠)
- **v1 的读取进度**总是 ≤ **v1 的写入进度** (保证数据一致性)

---

## 4. 配置参数的作用

### `maximum_forward_bytes = 32`

这是 **chaining 的粒度控制**:

- **32 字节**: 意味着每个周期最多转发 32 字节
- **太小** (如 8): chaining 效果有限,延迟降低不明显
- **太大** (如 256): 可能导致缓冲区压力,功能单元冲突
- **32 字节**: 是一个平衡点 (lane_number * sew/8 = 4 * 32/8 = 16,可以两周期处理一个 lane)

### `bytes_per_event`

在 `VectorFunctionUnit::new()` 中:

```rust
bytes_per_event = config.get_data_length()  // lane_number * sew/8 = 16 字节
```

这决定了**功能单元的吞吐量**:

- 每个 event 处理 16 字节
- 配合 `maximum_forward_bytes = 32`,大约每 2 个周期可以转发一次

---
