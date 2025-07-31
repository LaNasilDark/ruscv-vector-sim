#import "@preview/touying:0.6.1": *
#import themes.university: *
#import "@preview/cetz:0.3.2"
#import "@preview/fletcher:0.5.4" as fletcher: node, edge
#import "@preview/numbly:0.1.0": numbly
#import "@preview/theorion:0.3.2": *
#import cosmos.clouds: *
#show: show-theorion

// cetz and fletcher bindings for touying
#let cetz-canvas = touying-reducer.with(reduce: cetz.canvas, cover: cetz.draw.hide.with(bounds: true))
#let fletcher-diagram = touying-reducer.with(reduce: fletcher.diagram, cover: fletcher.hide)

#show: university-theme.with(
  aspect-ratio: "16-9",
  // align: horizon,
  // config-common(handout: true),
  config-common(frozen-counters: (theorem-counter,)),  // freeze theorem counter for animation
  config-info(
    title: [Static Simulator],
    subtitle: [],
    author: [Jiaqi Si],
    date: datetime.today(),
    institution: [CUHK(SZ)],
    logo: emoji.school,
  ),
)


#set heading(numbering: numbly("{1}.", default: "1.1"))

#title-slide()

== Outline <touying:hidden>

#components.adaptive-columns(outline(title: none, indent: 1em))

= Simulator Structure

== Main Three Parts

The simulator is mainly divided into three main parts: the fetch unit, the execution section, and the register.


Among these, the fetch unit is relatively simple and only needs to queue instructions, as we do not handle jump statements.

= Execution Section

== Execution Section

The execution section consists of two main types of units:
- *Function Units*: Handle computational operations
- *Memory Units*: Handle load/store operations

Both unit types share a common buffer architecture design.

== Function Unit Types

The simulator supports multiple function unit types:

- *Vector Units*: VectorAlu, VectorMul, VectorDiv, VectorSlide
- *Float Units*: FloatAlu, FloatMul, FloatDiv  
- *Integer Units*: IntegerAlu, IntegerDiv

Each function unit operates independently and can process different instruction types concurrently.

== Memory Unit Structure

The Load/Store Unit (LSU) handles memory operations:

- *Multiple Ports*: Separate read and write ports
- *Port-based Processing*: Each port can handle one memory operation at a time
- *Direction Support*: Read (load) and Write (store) operations

```rust
pub enum Direction {
    Read,   // Load operations
    Write   // Store operations
}
```

== Buffer Architecture Overview

Each execution unit (both function and memory) uses a *BufferPair* structure:

```rust
pub struct BufferPair {
    pub input_buffer: InputBuffer,
    pub result_buffer: ResultBuffer,
    pub current_instruction: Option<Inst>,
    pub owner: BufferOwnerType,
}
```

This design provides:
- *Input buffering* for operand data
- *Output buffering* for results
- *Instruction tracking* for current operation

== Input Buffer Structure

The Input Buffer manages multiple resource types:

```rust
pub struct InputBuffer {
    pub resource: Vec<Resource>
}

pub struct Resource {
    pub resource_type: ResourceType,
    pub target_size: u32,    // Total bytes needed
    pub current_size: u32,   // Current bytes available
}
```

*Resource Types*:
- `Register(RegisterType)`: Vector/Scalar/Float registers
- `Memory`: Memory data

== Output Buffer Structure  

The Result Buffer stores computation results:

```rust
pub struct ResultBuffer {
    pub destination: Option<EnhancedResource>
}

pub struct EnhancedResource {
    pub resource_type: ResourceType,
    pub target_size: u32,      // Total bytes to produce
    pub current_size: u32,     // Current bytes stored
    pub consumed_bytes: u32    // Bytes already consumed
}
```

This tracks both *production* and *consumption* of results.

== Buffer Event System

The buffer system uses an event-driven model:

*Producer Events*: Add data to buffers
```rust
pub struct ProducerEvent {
    pub resource_index: usize,
    pub append_length: u32,
}
```

*Consumer Events*: Read data from buffers
```rust
pub struct ConsumerEvent {
    pub maximum_consume_length: u32,
}
```

== Function Unit Processing Flow

1. *Input Stage*: Operands arrive in Input Buffer
2. *Event Generation*: EventGenerator creates processing events
3. *Execution*: Function unit processes data, remove an event from the queue and change the state of the Result Buffer
4. *Output Stage*: Results stored in Result Buffer
5. *Consumption*: Register file reads results

```rust
pub struct EventGenerator {
    func_inst: FuncInst,
    cycle_per_event: u32,
    bytes_per_event: u32,
    total_bytes: u32,
    processed_bytes: u32,
}
```

== Memory Unit Processing Flow

1. *Port Allocation*: Find available read/write port
2. *Data Waiting*: Check if data is available in Input Buffer
3. *Data Transfer*: Move data between memory and buffers
4. *Port Release*: Free port for next operation

```rust
pub struct MemoryPortEventGenerator {
    index: usize,
    bytes_per_cycle: u32,
    raw_inst: MemInst,
    total_bytes: u32,
    current_pos: u32,
}
```

== Latency Calculation

Each unit type has configurable latency:

```rust
pub fn calc_func_cycle(inst: &FuncInst) -> u32 {
    match inst.get_key_type() {
        FunctionUnitKeyType::IntegerAlu => config.function_units.interger_alu.latency,
        FunctionUnitKeyType::VectorAlu => /* depends on float/int */,
        FunctionUnitKeyType::VectorDiv => /* depends on float/int */,
        // ... other unit types
    }
}
```

Latencies are defined in the configuration file and can be customized.

= Register

== RegisterFile Structure

The RegisterFile manages three types of registers:

```rust
pub struct RegisterFile {
    pub scalar_registers: [CommonRegister; 32],
    pub vector_registers: [VectorRegister; 32], 
    pub float_registers: [CommonRegister; 32],
}
```

*Register Types*:
- *Scalar Registers*: 32-bit integer values
- *Vector Registers*: Variable-length vector data
- *Float Registers*: Floating-point values

== Register Internal Structure

Each register maintains a task queue for managing read/write operations:

*CommonRegister* (for Scalar and Float):
```rust
pub struct CommonRegister {
    pub task_queue: VecDeque<RegisterTask>,
    pub current_index: usize,
}
```

*VectorRegister*:
```rust
pub struct VectorRegister {
    pub task_queue: VecDeque<RegisterTask>,
    pub current_index: usize,
}
```

== RegisterTask Structure

Each task represents a read or write operation:

```rust
pub struct RegisterTask {
    pub current_place: u32,      // Current processing position
    pub resource_index: usize,   // Buffer resource index
    pub behavior: TaskBehavior,  // Read or Write
    pub unit_key: UnitKeyType,   // Associated execution unit
}

pub enum TaskBehavior {
    Read,   // Read from register
    Write,  // Write to register
}
```

== Task Registration Process

When an instruction is issued, read/write events are registered:

*Function Unit Instructions*:
```rust
pub fn add_task(&mut self, inst: &FuncInst, unit_key: FuncKey) {
    // Add read tasks for source registers
    for src_reg in inst.get_source_registers() {
        self.add_read_task(src_reg, unit_key);
    }
    
    // Add write task for destination register
    if let Some(dst_reg) = inst.get_destination_register() {
        self.add_write_task(dst_reg, unit_key);
    }
}
```

== Memory Unit Task Registration

*Memory Instructions*:
```rust
pub fn add_mem_task(&mut self, inst: &MemInst, unit_key: MemKey) {
    match unit_key {
        MemKey::Load => {
            // Read address dependencies
            self.add_read_task(inst.get_address_register(), unit_key);
            // Write loaded data to destination
            self.add_write_task(inst.get_data_register(), unit_key);
        },
        MemKey::Store => {
            // Read address and data dependencies
            self.add_read_task(inst.get_address_register(), unit_key);
            self.add_read_task(inst.get_data_register(), unit_key);
        }
    }
}
```

== RegisterTaskHandler Trait

All register types implement the RegisterTaskHandler trait:

```rust
pub trait RegisterTaskHandler {
    fn handle_one_task(&mut self, forward_bytes: u32, update_length: u32) 
        -> Option<BufferEvent>;
    fn handle_event_result(&mut self, result: BufferEventResult);
    fn generate_event(&mut self) -> Option<BufferEvent>;
    fn task_queue(&self) -> &VecDeque<RegisterTask>;
    fn get_total_bytes(&self, task: &RegisterTask) -> u32;
}
```

== Task Processing Flow

1. *Task Creation*: When instruction issued, tasks added to register queues
2. *Event Generation*: Tasks generate BufferEvents (Producer/Consumer)
3. *Data Transfer*: Events move data between registers and execution units 
4. *Progress Tracking*: Tasks track current_place and completion status
5. *Task Completion*: Completed tasks removed from queue

== Data Forwarding Mechanism

The simulation supports data forwarding for chaining instructions

```rust
fn handle_one_task(&mut self, forward_bytes: u32, update_length: u32) 
    -> Option<BufferEvent> {
    if let Some(task) = self.task_queue().front_mut() {
        match task.behavior {
            TaskBehavior::Read => {
                // Generate Consumer event
                Some(BufferEvent::Consumer(ConsumerEvent { ... }))
            },
            TaskBehavior::Write => {
                // Generate Producer event  
                Some(BufferEvent::Producer(ProducerEvent { ... }))
            }
        }
    }
}
```
== Chaining 

The data that needs to be forwarded will be registered as a *Read Task* queued after a certain *Write Task*. In this way, subsequent instructions can obtain the results already calculated by the previous instruction through the Register's processing of events in each cycle.

= Current Limitations

== Current Limitations

Does not support mask instructions.

Since MACC instructions need to be split into microinstructions, they are also not supported.

Each instruction must wait until the Result Buffer is cleared before it can be issued. Therefore, for scalar instructions, the fetch unit will be stuck for two cycles (being modified).