# Main Changes Log

This document records the major changes made to the simulator project since taking over from Jiaqi.

## 1. Bug Fixes

### 1.1 SD Instruction Infinite Loop (August 27)
- **Issue**: Fixed infinite loop caused by the `sd` instruction
- **Files Modified**: `memory_unit.rs`

### 1.2 Memory Instruction Issue Timing (October 8)
- **Issue**: Fixed incorrect timing of memory instruction issue
- **Files Modified**: `sim.rs`, `register.rs`

### 1.3 VSE Instruction Chaining (October 30)
- **Issue**: Fixed `vse` instruction unable to participate in chaining
- **Files Modified**: `register.rs`

## 2. Instruction Support

### 2.1 VSETIVLI Instruction (September 17)
- Added support for `vsetivli` instruction

### 2.2 Vector Arithmetic Instructions (October 16)
- Added support for the following instructions:
  - `vadd.vv`
  - `vmul.vx`
  - `vmul.vv`
  - `vredsum.vs`
  - `vadd.vx`
  - `vsub.vv`

## 3. Other Changes

### 3.1 GitHub Submodules Removal (September 13)
- Removed GitHub submodules functionality
- Converted `vendor` folder contents to regular folder management

### 3.2 Log Analysis Tool (September 13)
- Created Python script for log analysis: `log_parser.py`
