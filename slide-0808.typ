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

= Vector Register Read Ports Configuration

== Overview

This presentation demonstrates how to configure vector register read ports through configuration files and shows the implementation details in the simulator.

== Configuration File Setup

=== Vector Register Ports Configuration

In the `config.toml` file, you can configure the number of read ports available for vector registers:

```toml
[vector_register.ports]
read_ports_limit = 1  # Number of simultaneous read ports for one vector register
write_ports_limit = 1 # Number of simultaneous write ports for one vector register
```

=== Key Configuration Parameters

- `read_ports_limit`: Controls how many vector registers can be read simultaneously
- `write_ports_limit`: Controls how many vector registers can be written simultaneously
- Setting to a large number (e.g., 64) effectively removes the limitation

== Code Implementation

=== Configuration Loading

The configuration is loaded in `src/config.rs`:

```rust
#[derive(Debug, Deserialize, Clone)]
pub struct VectorRegisterPorts {
    pub read_ports_limit: u32,
    pub write_ports_limit: u32,
}

pub fn get_vector_register_read_ports_limit(&self) -> u32 {
    self.vector_register.ports.read_ports_limit
}
```

=== Port Conflict Detection

The simulator checks for port conflicts in `src/sim/register.rs`:

```rust
pub fn can_issue_vector_instruction(&self, func_inst: &FuncInst) -> bool {
    let read_ports_limit = config.get_vector_register_read_ports_limit();
    
    for operand in &func_inst.resource {
        match operand {
            RegisterType::VectorRegister(id) => {
                let current_read_count = self.vector_registers[*id as usize].get_read_count();
                if current_read_count + 1 > read_ports_limit {
                    debug!("Cannot issue: vector register {} read count would exceed limit", id);
                    return false;
                }
            }
        }
    }
    true
}
```

== Example and Demonstration

#v(2em)
=== Test Configuration

To demonstrate port conflicts, set `read_ports_limit = 1` in config.toml:

```toml
[vector_register.ports]
read_ports_limit = 1
write_ports_limit = 1
```

=== Running the Conflict Port Example

```bash
cargo run -- -i appendix/_conflict_port/bin/conflict_port.exe -c ./config.toml -s 0x1023c -e 0x10250
```
#slide[
#v(2em)
=== Assembly Code
```sh
   1023c: 07 74 05 02  	vle64.v	v8, (a0)
   10240: 87 f4 05 02  	vle64.v	v9, (a1)
   10244: d7 94 84 92  	vfmul.vv	v9, v8, v9
   10248: 57 94 84 02  	vfadd.vv	v8, v8, v9
   1024c: 27 f4 06 02  	vse64.v	v8, (a3)
```

]

#slide[
#v(2em)
=== Expected Log Output

When port conflicts occur, you'll see debug messages like:

```sh
01:46:53 [INFO] Step 3: Fetching new instructions and checking if they can be issued
01:46:53 [DEBUG] (1) ruscv_vector_sim::sim: Trying to issue instruction: Func(FuncInst { raw: VFADD_VV { vrd: 8, vrs1: 9, vrs2: 8 }, destination: VectorRegister(8), resource: [VectorRegister(9), VectorRegister(8)], func_unit_key: VectorAlu })
01:46:53 [DEBUG] (1) ruscv_vector_sim::sim: Function instruction cycles: 3
01:46:53 [DEBUG] (1) ruscv_vector_sim::sim::register: [ISSUE_CHECK_DEBUG] Checking vector instruction: VFADD_VV { vrd: 8, vrs1: 9, vrs2: 8 }
01:46:53 [DEBUG] (1) ruscv_vector_sim::sim::register: [ISSUE_CHECK_DEBUG] Port limits - read: 1, write: 1
01:46:53 [DEBUG] (1) ruscv_vector_sim::sim::register: [ISSUE_CHECK_DEBUG] Vector register 9 current read count: 1, limit: 1
01:46:53 [DEBUG] (1) ruscv_vector_sim::sim::register: [ISSUE_CHECK_DEBUG] Cannot issue: vector register 9 read count would exceed limit (1 + 1 > 1)
01:46:53 [DEBUG] (1) ruscv_vector_sim::sim: Function unit VectorAlu cannot accept new instruction yet, waiting
01:46:53 [INFO] ========== Simulation for cycle 7 completed ==========
```
]
