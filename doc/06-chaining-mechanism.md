## Current Simulator's Chaining Mechanism

---

## 1. Core Design Philosophy

### 1.1 Data Structure Design

**Queue-based task management system defined in `register.rs` file**:

```rust
pub struct VectorRegister {
    pub id: RegisterIdType,
    pub total_bytes: u32,
    pub write_count: u32,
    pub read_count: u32,
    pub current_index: usize,
    pub task_queue: VecDeque<RegisterTask>  // KeyPoint
}
```

**`task_queue` is the core of chaining**:

- Each vector register maintains a task queue
- The queue contains multiple read/write tasks
- Tasks are added from the **front** (`push_front`) and removed from the **back** (`pop_back`)
- This forms a **producer-consumer chain**

### 1.2 RegisterTask Structure in `task.rs` File

```rust
pub struct RegisterTask {
    pub current_place: u32,      // Current processing progress (bytes)
    pub resource_index: usize,   // Corresponding input buffer index
    pub behavior: UnitBehavior,  // Read or Write
    pub unit_key: UnitKeyType    // Associated functional unit
}
```

This design enables:

- **Tracking progress of each task** (`current_place`)
- **Supporting multiple dependent instructions** (multiple tasks can exist in the queue)
- **Distinguishing read/write behavior** (Read/Write)

---

## 2. Specific Implementation of Chaining

### 2.1 Element-Level/Chunk-Level Forwarding

**Key function in `register.rs` file**: `VectorRegister::handle_one_task()`

```rust
// Only showing key portions of the function
pub fn handle_one_task(&self, index: usize) -> Option<BufferEvent> {
    let forward_bytes = SimulatorConfig::get_global_config()
        .unwrap()
        .get_maximum_forward_bytes()  // Configuration: maximum_forward_bytes = 32
        .min(self.get_total_bytes() - q[index].current_place);
  
    match index == q.len() - 1 {
        true => {
            // Last task: can forward forward_bytes
            Some(q[index].generate_event(forward_bytes))
        },
        false => {
            // Intermediate task: can only forward up to next task's current position
            let mut update_length = q[index+1].current_place - q[index].current_place;
            update_length = update_length.min(forward_bytes);
  
            if update_length == 0 {
                None  // Wait for next task to consume more data
            } else {
                Some(q[index].generate_event(update_length))
            }
        }
    }
}
```

1. **Last task** (producer): Can freely forward data according to `maximum_forward_bytes`
2. **Intermediate task** (consumer): Can only forward data that has been consumed by subsequent tasks
3. **Forwarding granularity**: 32 bytes/cycle (configurable)

### 2.2 Pipelined Overlapped Execution

**Key mechanism in `function_unit.rs` file**: `EventGenerator` and `generate_next_event()` function

```rust
pub struct EventGenerator {
    func_inst: FuncInst,
    cycle_per_event: u32,
    bytes_per_event: u32,      // Bytes processed per event
    total_bytes: u32,
    processed_bytes: u32,
}
```

```rust
pub fn generate_next_event(&mut self, current_bytes: u32) -> Option<Event> {
    let bytes_this_event = self.bytes_per_event
        .min(self.total_bytes - self.processed_bytes)
        .min(current_bytes - self.processed_bytes);  // KeyPoint
  
    if bytes_this_event == 0 {
        return None;  // Wait for more input data
    }
  
    Some(Event {
        remained_cycle: self.cycle_per_event,
        target_register: self.func_inst.destination.clone(),
        result_bytes: bytes_this_event,
    })
}
```

- `current_bytes`: Number of bytes ready in the input buffer
- Only generates new events when `current_bytes > processed_bytes`
- **No need to wait for entire vector completion**, processes as much data as available

### 2.3 Dependency Management

**In `RegisterFile::add_vector_task()` in `register.rs` file**:

```rust
pub fn add_vector_task(&mut self, func_inst: &FuncInst) {
    // Add read tasks for source registers
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
  
    // Add write task for destination register
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

**Establishing dependency chains**:

```
vadd.vv v1, v2, v3  ->  v1.task_queue = [Write(vadd)]
                        v2.task_queue = [Read(vadd)]
                        v3.task_queue = [Read(vadd)]

vmul.vv v4, v1, v5  ->  v1.task_queue = [Read(vmul), Write(vadd)]  // Chained!
                        v4.task_queue = [Write(vmul)]
                        v5.task_queue = [Read(vmul)]
```

---

## 3. Execution Flow Example

Assume execution:

```assembly
vadd.vv v1, v2, v3    # v1 = v2 + v3, each vector 256 bytes
vmul.vv v4, v1, v5    # v4 = v1 * v5
```

### Timeline (maximum_forward_bytes = 32):

| Cycle | vadd Progress | v1 Write | vmul Progress | v1 Read | Description                       |
| ----- | ------------- | -------- | ------------- | ------- | --------------------------------- |
| 0     | 0/256         | 0/256    | -             | -       | vadd starts                       |
| 1     | 32/256        | 32/256   | -             | -       | vadd generates first event        |
| 2     | 64/256        | 64/256   | -             | -       | Continues processing              |
| 3     | 96/256        | 96/256   | 0/256         | 0/96    | **vmul starts! (chaining)** |
| 4     | 128/256       | 128/256  | 32/256        | 32/128  | **Pipelined overlap**       |
| 5     | 160/256       | 160/256  | 64/256        | 64/160  | Continues overlapping             |
| ...   | ...           | ...      | ...           | ...     | ...                               |

**Key observations**:

- **Cycle 3**: vmul doesn't need to wait for vadd to complete all 256 bytes
- **Cycle 4-5**: vadd and vmul **execute simultaneously** (pipelined overlap)
- **v1's read progress** is always ≤ **v1's write progress** (ensures data consistency)

---

## 4. Role of Configuration Parameters

### `maximum_forward_bytes = 32`

This controls **chaining granularity**:

- **32 bytes**: Means at most 32 bytes can be forwarded per cycle
- **Too small** (e.g., 8): Limited chaining effect, latency reduction not significant
- **Too large** (e.g., 256): May cause buffer pressure and functional unit conflicts
- **32 bytes**: A balanced choice (lane_number * sew/8 = 4 * 32/8 = 16, can process one lane in two cycles)
