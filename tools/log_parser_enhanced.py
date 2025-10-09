#!/usr/bin/env python3
import re
from collections import defaultdict, namedtuple
from typing import Dict, List, Optional, Tuple, Set
from dataclasses import dataclass, asdict, field
import argparse
import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
from pathlib import Path

# Data structure definitions
@dataclass
class Instruction:
    """Instruction information"""
    name: str
    rd: Optional[int] = None
    rs1: Optional[int] = None
    rs2: Optional[int] = None
    vrd: Optional[int] = None
    vrs1: Optional[int] = None
    vrs2: Optional[int] = None
    imm: Optional[int] = None
    shamt: Optional[int] = None
    width: Optional[int] = None

@dataclass
class InstructionEvent:
    """Instruction event"""
    cycle: int
    pc: int
    instruction: Instruction
    event_type: str  # 'issued', 'started', 'completed'
    unit: Optional[str] = None

@dataclass
class UnitStatus:
    """Functional unit status"""
    cycle: int
    unit_name: str
    occupied: bool
    event_queue_size: int
    current_instruction: Optional[Instruction] = None

@dataclass 
class MemoryEvent:
    """Memory access event"""
    cycle: int
    event_type: str  # 'load', 'store'
    address: Optional[str] = None
    instruction: Optional[Instruction] = None

@dataclass
class FunctionUnitConfig:
    """Function unit configuration"""
    interger_alu: int = 0
    interger_multiplier: int = 0
    float_alu: int = 0
    float_multiplier: int = 0
    interger_divider: int = 0
    float_divider: int = 0
    branch_unit: int = 0

@dataclass
class MemoryUnitConfig:
    """Memory unit configuration"""
    latency: int = 0
    max_access_width: int = 0
    read_ports_limit: int = 0
    write_ports_limit: int = 0

@dataclass
class VectorSoftwareConfig:
    """Vector software configuration"""
    vl: int = 0          # Vector Length
    sew: int = 0         # Scalar Element Width
    lmul: int = 0        # Lane Multiplier

@dataclass
class VectorHardwareConfig:
    """Vector hardware configuration"""
    vlen: int = 0        # Vector Register Length
    lane_number: int = 0 # Vector Lane Number

@dataclass
class VectorRegisterConfig:
    """Vector register configuration"""
    read_ports_limit: int = 0
    write_ports_limit: int = 0

@dataclass
class BufferConfig:
    """Buffer configuration"""
    input_maximum_size: int = 0
    result_maximum_size: int = 0

@dataclass
class RegisterConfig:
    """Register configuration"""
    maximum_forward_bytes: int = 0

@dataclass
class SimulatorConfig:
    """Simulator configuration"""
    # Legacy fields for backward compatibility
    function_units: Optional[Dict[str, int]] = None  # unit name -> latency
    vector_length: int = 0
    element_width: int = 0
    vector_register_length: int = 0
    lane_number: int = 0
    
    # Detailed configuration - using field(default_factory=...) for proper initialization
    function_unit_config: FunctionUnitConfig = field(default_factory=FunctionUnitConfig)
    memory_unit_config: MemoryUnitConfig = field(default_factory=MemoryUnitConfig)
    vector_software_config: VectorSoftwareConfig = field(default_factory=VectorSoftwareConfig)
    vector_hardware_config: VectorHardwareConfig = field(default_factory=VectorHardwareConfig)
    vector_register_config: VectorRegisterConfig = field(default_factory=VectorRegisterConfig)
    buffer_config: BufferConfig = field(default_factory=BufferConfig)
    register_config: RegisterConfig = field(default_factory=RegisterConfig)
    
    # Derived values
    vector_register_size_bytes: int = 0
    element_size_bytes: int = 0
    total_vector_operation_size_bytes: int = 0

class LogParser:
    """Main log parser class"""
    
    def __init__(self):
        self.instructions = []  # all instruction list
        self.instruction_events = []  # instruction event list
        self.unit_statuses = []  # functional unit status list
        self.memory_events = []  # memory event list
        self.current_cycle = 0
        self.current_pc = 0
        self.config = SimulatorConfig()  # This will auto-initialize all nested configs
        self.total_cycles = 0  # total cycle count field
        
        # Statistical data
        self.instruction_stats = defaultdict(dict)  # instruction ID -> statistics
        self.unit_utilization = defaultdict(list)  # unit name -> occupied cycle list
        self.function_units = set()  # all functional unit names
        
        # Parse mode
        self.verbose = False
        
        # Output directory for images
        self.output_dir = None  # Can be None, False (disabled), or Path object
        self.skip_images = False  # Flag to control image generation
        
    def setup_output_directory(self, log_file_path: str):
        """Setup output directory for parser reports"""
        log_file = Path(log_file_path)
        # Remove file extension to get log name
        log_name = log_file.stem
        
        # Create output directory in analysis/parser_report folder with log name subfolder
        script_dir = Path(__file__).parent.parent  # Go up to project root
        self.output_dir = script_dir / "analysis" / "parser_report" / log_name
        
        # Create directory if it doesn't exist
        self.output_dir.mkdir(parents=True, exist_ok=True)
        print(f"Output directory created: {self.output_dir}")
        
        # Set matplotlib backend and style
        plt.style.use('default')
        plt.rcParams['font.size'] = 10
        plt.rcParams['figure.dpi'] = 150
        
    def parse_instruction(self, inst_str: str) -> Optional[Instruction]:
        """Parse instruction string - enhanced version"""
        # Remove extra spaces and brackets
        inst_str = inst_str.strip()
        
        # Match different types of instructions - comprehensive patterns
        patterns = {
            # Basic arithmetic instructions
            r'ADDI \{ rd: (\d+), rs1: (\d+), imm: (-?\d+) \}': 
                lambda m: Instruction('ADDI', rd=int(m.group(1)), rs1=int(m.group(2)), imm=int(m.group(3))),
            r'ADD \{ rd: (\d+), rs1: (\d+), rs2: (\d+) \}': 
                lambda m: Instruction('ADD', rd=int(m.group(1)), rs1=int(m.group(2)), rs2=int(m.group(3))),
            r'SUB \{ rd: (\d+), rs1: (\d+), rs2: (\d+) \}': 
                lambda m: Instruction('SUB', rd=int(m.group(1)), rs1=int(m.group(2)), rs2=int(m.group(3))),
            r'MUL \{ rd: (\d+), rs1: (\d+), rs2: (\d+) \}':
                lambda m: Instruction('MUL', rd=int(m.group(1)), rs1=int(m.group(2)), rs2=int(m.group(3))),
            r'SLLI \{ rd: (\d+), rs1: (\d+), shamt: (\d+) \}': 
                lambda m: Instruction('SLLI', rd=int(m.group(1)), rs1=int(m.group(2)), shamt=int(m.group(3))),
            
            # Jump instructions
            r'JALR \{ rd: (\d+), rs1: (\d+), offset: (-?\d+) \}':
                lambda m: Instruction('JALR', rd=int(m.group(1)), rs1=int(m.group(2)), imm=int(m.group(3))),
                
            # Vector instructions - adjusted according to actual log format
            r'VLE \{ vrd: (\d+), rs1: (\d+), width: (\d+) \}': 
                lambda m: Instruction('VLE', vrd=int(m.group(1)), rs1=int(m.group(2)), width=int(m.group(3))),
            r'VSE \{ vrd: (\d+), rs1: (\d+), width: (\d+) \}': 
                lambda m: Instruction('VSE', vrd=int(m.group(1)), rs1=int(m.group(2)), width=int(m.group(3))),
            r'VFADD_VV \{ vrd: (\d+), vrs1: (\d+), vrs2: (\d+) \}': 
                lambda m: Instruction('VFADD_VV', vrd=int(m.group(1)), vrs1=int(m.group(2)), vrs2=int(m.group(3))),
            r'VFMUL_VV \{ vrd: (\d+), vrs1: (\d+), vrs2: (\d+) \}': 
                lambda m: Instruction('VFMUL_VV', vrd=int(m.group(1)), vrs1=int(m.group(2)), vrs2=int(m.group(3))),
            r'VFMACC_VV \{ vrd: (\d+), vrs1: (\d+), vrs2: (\d+) \}': 
                lambda m: Instruction('VFMACC_VV', vrd=int(m.group(1)), vrs1=int(m.group(2)), vrs2=int(m.group(3))),
            r'VADD_VV \{ vrd: (\d+), vrs1: (\d+), vrs2: (\d+) \}': 
                lambda m: Instruction('VADD_VV', vrd=int(m.group(1)), vrs1=int(m.group(2)), vrs2=int(m.group(3))),
                
            # Vector configuration instructions
            r'VSETVLI \{ rd: (\d+), rs1: (\d+), sew: (\d+), .+ \}': 
                lambda m: Instruction('VSETVLI', rd=int(m.group(1)), rs1=int(m.group(2))),
        }
        
        for pattern, creator in patterns.items():
            match = re.search(pattern, inst_str)
            if match:
                return creator(match)
        
        # If no match found, extract instruction name
        name_match = re.search(r'^(\w+)', inst_str)
        if name_match:
            return Instruction(name_match.group(1))
        
        return None
    
    def parse_config(self, line: str):
        """Parse simulator configuration information"""
        # Parse main simulator configuration block
        if "Simulator config: SimulatorConfig" in line:
            self._parse_full_config(line)
            return
            
        # Parse individual configuration lines (legacy support)
        if "Vector Length (vl):" in line:
            match = re.search(r'Vector Length \(vl\): (\d+)', line)
            if match:
                vl = int(match.group(1))
                self.config.vector_length = vl
                self.config.vector_software_config.vl = vl
        elif "Scalar Element Width (sew):" in line:
            match = re.search(r'Scalar Element Width \(sew\): (\d+) bits', line)
            if match:
                sew = int(match.group(1))
                self.config.element_width = sew
                self.config.vector_software_config.sew = sew
        elif "Lane Multiplier (lmul):" in line:
            match = re.search(r'Lane Multiplier \(lmul\): (\d+)', line)
            if match:
                self.config.vector_software_config.lmul = int(match.group(1))
        elif "Vector Register Length (vlen):" in line:
            match = re.search(r'Vector Register Length \(vlen\): (\d+) bits', line)
            if match:
                vlen = int(match.group(1))
                self.config.vector_register_length = vlen
                self.config.vector_hardware_config.vlen = vlen
        elif "Vector Lane Number:" in line:
            match = re.search(r'Vector Lane Number: (\d+)', line)
            if match:
                lanes = int(match.group(1))
                self.config.lane_number = lanes
                self.config.vector_hardware_config.lane_number = lanes
        elif "Vector Register Size:" in line:
            match = re.search(r'Vector Register Size: (\d+) bytes', line)
            if match:
                self.config.vector_register_size_bytes = int(match.group(1))
        elif "Element Size:" in line:
            match = re.search(r'Element Size: (\d+) bytes', line)
            if match:
                self.config.element_size_bytes = int(match.group(1))
        elif "Total Vector Operation Size:" in line:
            match = re.search(r'Total Vector Operation Size: (\d+) bytes', line)
            if match:
                self.config.total_vector_operation_size_bytes = int(match.group(1))
    
    def _parse_full_config(self, line: str):
        """Parse the full simulator configuration from the main config line"""
        if self.verbose:
            print(f"Parsing full config from line: {line[:100]}...")
            
        # Extract function units configuration - use a more greedy approach
        func_units_match = re.search(r'function_units:\s*FunctionUnits\s*\{(.*?)\},\s*memory_units:', line)
        if func_units_match:
            func_units_str = func_units_match.group(1)
            self._parse_function_units(func_units_str)
            if self.verbose:
                print("  Parsed function units configuration")
        
        # Extract memory units configuration - made more flexible  
        memory_units_match = re.search(r'memory_units:\s*MemoryUnits\s*\{\s*load_store_unit:\s*LoadStoreUnit\s*\{([^}]+)\}\s*\}', line)
        if memory_units_match:
            memory_units_str = memory_units_match.group(1)
            self._parse_memory_unit_direct(memory_units_str)
            if self.verbose:
                print("  Parsed memory units configuration")
            
        # Extract vector configuration - use greedy matching
        vector_config_match = re.search(r'vector_config:\s*VectorConfig\s*\{(.*?)\},\s*vector_register:', line)
        if vector_config_match:
            vector_config_str = vector_config_match.group(1)
            self._parse_vector_config(vector_config_str)
            if self.verbose:
                print("  Parsed vector configuration")
            
        # Extract vector register configuration
        vector_register_match = re.search(r'vector_register:\s*VectorRegister\s*\{\s*ports:\s*VectorRegisterPorts\s*\{([^}]+)\}\s*\}', line)
        if vector_register_match:
            vector_register_str = vector_register_match.group(1)
            self._parse_vector_register_config_direct(vector_register_str)
            if self.verbose:
                print("  Parsed vector register configuration")
            
        # Extract buffer configuration  
        buffer_match = re.search(r'buffer:\s*BufferConfig\s*\{([^}]+)\}', line)
        if buffer_match:
            buffer_str = buffer_match.group(1)
            self._parse_buffer_config(buffer_str)
            if self.verbose:
                print("  Parsed buffer configuration")
            
        # Extract register configuration
        register_match = re.search(r'register:\s*RegisterConfig\s*\{([^}]+)\}', line)
        if register_match:
            register_str = register_match.group(1)
            self._parse_register_config(register_str)
            if self.verbose:
                print("  Parsed register configuration")
                
    def _parse_memory_unit_direct(self, memory_str: str):
        """Parse memory unit configuration directly from string"""
        latency_match = re.search(r'latency:\s*(\d+)', memory_str)
        if latency_match:
            self.config.memory_unit_config.latency = int(latency_match.group(1))
            
        max_width_match = re.search(r'max_access_width:\s*(\d+)', memory_str)
        if max_width_match:
            self.config.memory_unit_config.max_access_width = int(max_width_match.group(1))
            
        read_ports_match = re.search(r'read_ports_limit:\s*(\d+)', memory_str)
        if read_ports_match:
            self.config.memory_unit_config.read_ports_limit = int(read_ports_match.group(1))
            
        write_ports_match = re.search(r'write_ports_limit:\s*(\d+)', memory_str)
        if write_ports_match:
            self.config.memory_unit_config.write_ports_limit = int(write_ports_match.group(1))
    
    def _parse_vector_register_config_direct(self, register_str: str):
        """Parse vector register configuration directly from string"""
        read_ports_match = re.search(r'read_ports_limit:\s*(\d+)', register_str)
        if read_ports_match:
            self.config.vector_register_config.read_ports_limit = int(read_ports_match.group(1))
            
        write_ports_match = re.search(r'write_ports_limit:\s*(\d+)', register_str)
        if write_ports_match:
            self.config.vector_register_config.write_ports_limit = int(write_ports_match.group(1))
    
    def _parse_function_units(self, func_units_str: str):
        """Parse function units configuration"""
        if self.verbose:
            print(f"    Function units string: {func_units_str}")
            
        # Parse individual unit latencies
        units = {
            'interger_alu': r'interger_alu:\s*Unit\s*\{\s*latency:\s*(\d+)\s*\}',
            'interger_multiplier': r'interger_multiplier:\s*Unit\s*\{\s*latency:\s*(\d+)\s*\}',
            'float_alu': r'float_alu:\s*Unit\s*\{\s*latency:\s*(\d+)\s*\}',
            'float_multiplier': r'float_multiplier:\s*Unit\s*\{\s*latency:\s*(\d+)\s*\}',
            'interger_divider': r'interger_divider:\s*Unit\s*\{\s*latency:\s*(\d+)\s*\}',
            'float_divider': r'float_divider:\s*Unit\s*\{\s*latency:\s*(\d+)\s*\}',
            'branch_unit': r'branch_unit:\s*Unit\s*\{\s*latency:\s*(\d+)\s*\}'
        }
        
        for unit_name, pattern in units.items():
            match = re.search(pattern, func_units_str)
            if match:
                latency = int(match.group(1))
                setattr(self.config.function_unit_config, unit_name, latency)
                if self.verbose:
                    print(f"    Found {unit_name}: {latency}")
            elif self.verbose:
                print(f"    No match for {unit_name} with pattern: {pattern}")
    
    def _parse_memory_units(self, memory_units_str: str):
        """Parse memory units configuration"""
        # Parse load store unit configuration
        lsu_match = re.search(r'load_store_unit: LoadStoreUnit \{([^}]+)\}', memory_units_str)
        if lsu_match:
            lsu_str = lsu_match.group(1)
            
            latency_match = re.search(r'latency: (\d+)', lsu_str)
            if latency_match:
                self.config.memory_unit_config.latency = int(latency_match.group(1))
                
            max_width_match = re.search(r'max_access_width: (\d+)', lsu_str)
            if max_width_match:
                self.config.memory_unit_config.max_access_width = int(max_width_match.group(1))
                
            read_ports_match = re.search(r'read_ports_limit: (\d+)', lsu_str)
            if read_ports_match:
                self.config.memory_unit_config.read_ports_limit = int(read_ports_match.group(1))
                
            write_ports_match = re.search(r'write_ports_limit: (\d+)', lsu_str)
            if write_ports_match:
                self.config.memory_unit_config.write_ports_limit = int(write_ports_match.group(1))
    
    def _parse_vector_config(self, vector_config_str: str):
        """Parse vector configuration"""
        # Parse software configuration
        software_match = re.search(r'software: SoftwareConfig \{([^}]+)\}', vector_config_str)
        if software_match:
            software_str = software_match.group(1)
            
            vl_match = re.search(r'vl: (\d+)', software_str)
            if vl_match:
                vl = int(vl_match.group(1))
                self.config.vector_software_config.vl = vl
                self.config.vector_length = vl  # legacy support
                
            sew_match = re.search(r'sew: (\d+)', software_str)
            if sew_match:
                sew = int(sew_match.group(1))
                self.config.vector_software_config.sew = sew
                self.config.element_width = sew  # legacy support
                
            lmul_match = re.search(r'lmul: (\d+)', software_str)
            if lmul_match:
                self.config.vector_software_config.lmul = int(lmul_match.group(1))
        
        # Parse hardware configuration
        hardware_match = re.search(r'hardware: HardwareConfig \{([^}]+)\}', vector_config_str)
        if hardware_match:
            hardware_str = hardware_match.group(1)
            
            vlen_match = re.search(r'vlen: (\d+)', hardware_str)
            if vlen_match:
                vlen = int(vlen_match.group(1))
                self.config.vector_hardware_config.vlen = vlen
                self.config.vector_register_length = vlen  # legacy support
                
            lane_match = re.search(r'lane_number: (\d+)', hardware_str)
            if lane_match:
                lanes = int(lane_match.group(1))
                self.config.vector_hardware_config.lane_number = lanes
                self.config.lane_number = lanes  # legacy support
    
    def _parse_vector_register_config(self, vector_register_str: str):
        """Parse vector register configuration"""
        ports_match = re.search(r'ports: VectorRegisterPorts \{([^}]+)\}', vector_register_str)
        if ports_match:
            ports_str = ports_match.group(1)
            
            read_ports_match = re.search(r'read_ports_limit: (\d+)', ports_str)
            if read_ports_match:
                self.config.vector_register_config.read_ports_limit = int(read_ports_match.group(1))
                
            write_ports_match = re.search(r'write_ports_limit: (\d+)', ports_str)
            if write_ports_match:
                self.config.vector_register_config.write_ports_limit = int(write_ports_match.group(1))
    
    def _parse_buffer_config(self, buffer_str: str):
        """Parse buffer configuration"""
        input_match = re.search(r'input_maximum_size: (\d+)', buffer_str)
        if input_match:
            self.config.buffer_config.input_maximum_size = int(input_match.group(1))
            
        result_match = re.search(r'result_maximum_size: (\d+)', buffer_str)
        if result_match:
            self.config.buffer_config.result_maximum_size = int(result_match.group(1))
    
    def _parse_register_config(self, register_str: str):
        """Parse register configuration"""
        forward_match = re.search(r'maximum_forward_bytes: (\d+)', register_str)
        if forward_match:
            self.config.register_config.maximum_forward_bytes = int(forward_match.group(1))
    
    def extract_function_unit_name(self, unit_str: str) -> str:
        """Extract functional unit name"""
        # Extract "IntegerAlu" from "FuncKey(IntegerAlu)"
        match = re.search(r'FuncKey\((\w+)\)', unit_str)
        if match:
            return match.group(1)
        
        # Extract unit name from brackets
        match = re.search(r'\[(\w+)\]', unit_str)
        if match:
            return match.group(1)
        
        return unit_str
    
    def parse_log_file(self, filepath: str):
        """Parse log file"""
        print(f"Parsing log file: {filepath}")
        
        # Setup output directory for images (only if image generation is not skipped)
        if not self.skip_images:
            self.setup_output_directory(filepath)
        
        with open(filepath, 'r', encoding='utf-8', errors='ignore') as f:
            lines = f.readlines()
        
        # First check total cycle count
        total_cycles = None
        for line in reversed(lines):
            line = line.strip()
            total_cycles_match = re.search(r'Main simulation loop ended, total cycles: (\d+)', line)
            if total_cycles_match:
                total_cycles = int(total_cycles_match.group(1))
                self.total_cycles = total_cycles
                break
        
        if total_cycles is None:
            print("\n" + "!" * 20)
            print("WARNING: Total cycle count not found, log may be incomplete or corrupted!")
            print("!" * 20 + "\n")
        else:
            print(f"Total cycles detected: {total_cycles}")
        
        for line_num, line in enumerate(lines):
            line = line.strip()
            if not line:
                continue
            
            # Parse configuration information
            self.parse_config(line)
            
            # Extract cycle information - corrected regex
            cycle_match = re.search(r'========== Starting simulation for cycle (\d+) ==========', line)
            if cycle_match:
                self.current_cycle = int(cycle_match.group(1))
                if self.verbose:
                    print(f"  Processing cycle {self.current_cycle}")
                continue
            
            # Parse instruction list
            if "the instructions are" in line:
                self._parse_instruction_list(line)
                continue
            
            # Parse detailed instruction issue (new format): "Function unit IntegerAlu issued instruction: SUB { rd: 30, rs1: 15, rs2: 28 } at cycle 0, PC advanced"
            # Also handles: "Memory unit issued instruction: VLE { vrd: 10, rs1: 8, width: 64 } at cycle 4, PC advanced"
            if ("Function unit" in line and "issued instruction:" in line and "at cycle" in line) or \
               ("Memory unit issued instruction:" in line and "at cycle" in line):
                self._parse_detailed_instruction_issue(line)
                continue
            
            # Parse PC advancement
            pc_match = re.search(r'Advancing PC from (\d+) to (\d+)', line)
            if pc_match:
                self.current_pc = int(pc_match.group(2))
                continue
            
            # Parse functional unit status
            if "Starting handle_event:" in line:
                self._parse_unit_status(line)
                continue
            
            # Parse ResultBuffer fully consumed (vector instruction completion)
            if "ResultBuffer is fully consumed, freeing unit" in line:
                self._parse_result_buffer_completed(line)
                continue
            
            # Parse instruction completion events (fallback for scalar or old logs)
            if "Event completed:" in line:
                self._parse_event_completed(line)
                continue
            
            # Parse memory unit completion events
            if "task completed:" in line:
                self._parse_memory_task_completed(line)
                continue
            
            # Parse functional unit write results
            if "write result to register" in line:
                self._parse_write_result(line)
                continue
                
            # Parse memory operations (if exist)
            if "memory access" in line or "load" in line or "store" in line:
                self._parse_memory_event(line)
                continue
    
    def _parse_instruction_list(self, line: str):
        """Parse instruction list"""
        # Extract content from square brackets - corrected regex
        match = re.search(r'the instructions are \[([^\]]+)\]', line)
        if match:
            inst_content = match.group(1)
            # Use smarter way to split instructions, considering internal commas and braces
            instructions = []
            brace_depth = 0
            current_inst = ""
            
            for char in inst_content:
                if char == '{':
                    brace_depth += 1
                elif char == '}':
                    brace_depth -= 1
                elif char == ',' and brace_depth == 0:
                    # Only commas outside braces are separators
                    if current_inst.strip():
                        instructions.append(current_inst.strip())
                    current_inst = ""
                    continue
                
                current_inst += char
            
            # Add the last instruction
            if current_inst.strip():
                instructions.append(current_inst.strip())
            
            # Parse each instruction
            for inst_str in instructions:
                inst = self.parse_instruction(inst_str)
                if inst:
                    self.instructions.append(inst)
            
            if self.verbose:
                print(f"  Parsed {len(instructions)} instructions")
    
    def _parse_detailed_instruction_issue(self, line: str):
        """Parse detailed instruction issue format: 'ruscv_vector_sim::sim: Function unit IntegerAlu issued instruction: SUB { rd: 30, rs1: 15, rs2: 28 } at cycle 0, PC advanced'"""
        # Parse function unit instruction issue format
        func_unit_match = re.search(r'Function unit (\w+) issued instruction: ([^}]+\}) at cycle (\d+)(, PC advanced)?', line)
        if func_unit_match:
            unit = func_unit_match.group(1)
            inst_str = func_unit_match.group(2)
            cycle = int(func_unit_match.group(3))
            pc_advanced = func_unit_match.group(4) is not None
            
            # If PC advanced, the current_pc is already updated to next instruction
            # So the actual PC of this instruction is current_pc - 1
            actual_pc = (self.current_pc - 1) if pc_advanced else self.current_pc
            
            inst = self.parse_instruction(inst_str)
            if inst:
                event = InstructionEvent(
                    cycle=cycle,
                    pc=actual_pc,
                    instruction=inst,
                    event_type='issued',
                    unit=unit
                )
                self.instruction_events.append(event)
                self.function_units.add(unit)
                if self.verbose:
                    print(f"    Parsed functional unit instruction issue: {inst.name} -> {unit} at cycle {cycle}, PC={actual_pc}")
            return
            
        # Parse memory unit instruction issue format: "Memory unit issued instruction: VLE { vrd: 10, rs1: 8, width: 64 } at cycle 4, PC advanced"
        mem_unit_match = re.search(r'Memory unit issued instruction: ([^}]+\}) at cycle (\d+)(, PC advanced)?', line)
        if mem_unit_match:
            inst_str = mem_unit_match.group(1)
            cycle = int(mem_unit_match.group(2))
            pc_advanced = mem_unit_match.group(3) is not None
            unit = "MemoryUnit"
            
            # If PC advanced, the current_pc is already updated to next instruction
            # So the actual PC of this instruction is current_pc - 1
            actual_pc = (self.current_pc - 1) if pc_advanced else self.current_pc
            
            inst = self.parse_instruction(inst_str)
            if inst:
                event = InstructionEvent(
                    cycle=cycle,
                    pc=actual_pc,
                    instruction=inst,
                    event_type='issued',
                    unit=unit
                )
                self.instruction_events.append(event)
                self.function_units.add(unit)
                if self.verbose:
                    print(f"    Parsed memory unit instruction issue: {inst.name} -> {unit} at cycle {cycle}, PC={actual_pc}")
            return
    
    def _parse_unit_status(self, line: str):
        """Parse functional unit status"""
        match = re.search(r'\[(\w+)\] Starting handle_event: event_queue_size=(\d+), occupied=(\w+)', line)
        if match:
            unit_name = match.group(1)
            queue_size = int(match.group(2))
            occupied = match.group(3) == 'true'
            
            status = UnitStatus(
                cycle=self.current_cycle,
                unit_name=unit_name,
                occupied=occupied,
                event_queue_size=queue_size
            )
            self.unit_statuses.append(status)
            self.function_units.add(unit_name)
    
    def _parse_result_buffer_completed(self, line: str):
        """Parse ResultBuffer fully consumed event (vector instruction completion)"""
        # Example: [VectorAlu] ResultBuffer is fully consumed, freeing unit - Instruction: VFADD_VV { vrd: 8, vrs1: 9, vrs2: 8 }, PC: 9, Cycle: 25
        match = re.search(r'\[(\w+)\] ResultBuffer is fully consumed, freeing unit - Instruction: (\w+)', line)
        if match:
            unit_name = match.group(1)
            inst_name = match.group(2)
            
            # Extract PC and Cycle if available
            pc_match = re.search(r'PC: (\d+)', line)
            cycle_match = re.search(r'Cycle: (\d+)', line)
            
            completion_cycle = int(cycle_match.group(1)) if cycle_match else self.current_cycle
            pc = int(pc_match.group(1)) if pc_match else None
            
            if self.verbose:
                print(f"    Found ResultBuffer completion: {inst_name} at {unit_name} in cycle {completion_cycle}")
            
            # Record instruction completion event - match by instruction name and unit
            # ResultBuffer completion is MORE ACCURATE than Event completed for vector instructions
            # so we need to update/replace any existing completion event
            if self.instruction_events:
                for event in reversed(self.instruction_events):
                    if (event.event_type == 'issued' and 
                        event.unit == unit_name and 
                        event.instruction.name == inst_name and
                        event.cycle < completion_cycle):
                        
                        # Check if completion event already exists - if so, remove it
                        # because ResultBuffer completion is more accurate
                        existing_completion = None
                        for idx, e in enumerate(self.instruction_events):
                            if (e.event_type == 'completed' and 
                                e.instruction.name == event.instruction.name and
                                e.pc == event.pc and
                                e.unit == event.unit):
                                existing_completion = idx
                                break
                        
                        if existing_completion is not None:
                            if self.verbose:
                                print(f"    Replacing existing completion event (cycle {self.instruction_events[existing_completion].cycle}) with ResultBuffer completion (cycle {completion_cycle})")
                            del self.instruction_events[existing_completion]
                        
                        # Add the new completion event with accurate cycle
                        completion_event = InstructionEvent(
                            cycle=completion_cycle,
                            pc=event.pc if pc is None else pc,
                            instruction=event.instruction,
                            event_type='completed',
                            unit=event.unit
                        )
                        self.instruction_events.append(completion_event)
                        if self.verbose:
                            print(f"    Added ResultBuffer completion event for {event.instruction.name} in cycle {completion_cycle}")
                        break
    
    def _parse_event_completed(self, line: str):
        """Parse event completion (fallback for scalar instructions or old logs)"""
        # Fixed regex to properly match ScalarRegister and VectorRegister formats
        match = re.search(r'\[(\w+)\] Event completed: target_register=(?:Scalar|Vector)Register\((\d+)\)', line)
        if match:
            unit_name = match.group(1)
            register_id = int(match.group(2))
            
            if self.verbose:
                print(f"    Found event completion: {unit_name} register {register_id} in cycle {self.current_cycle}")
            
            # Record instruction completion event - match by register and unit
            if self.instruction_events:
                for event in reversed(self.instruction_events):
                    if (event.event_type == 'issued' and 
                        event.unit == unit_name and 
                        event.cycle < self.current_cycle):
                        
                        # Match by target register to ensure correct instruction completion
                        instruction_matches = False
                        if event.instruction.rd == register_id:  # Scalar register destination
                            instruction_matches = True
                        elif event.instruction.vrd == register_id:  # Vector register destination
                            instruction_matches = True
                            
                        if instruction_matches:
                            # Check if completion event already exists
                            already_completed = any(
                                e.event_type == 'completed' and 
                                e.instruction.name == event.instruction.name and
                                e.pc == event.pc and
                                e.unit == event.unit
                                for e in self.instruction_events
                            )
                            
                            if not already_completed:
                                completion_event = InstructionEvent(
                                    cycle=self.current_cycle,
                                    pc=event.pc,
                                    instruction=event.instruction,
                                    event_type='completed',
                                    unit=event.unit
                                )
                                self.instruction_events.append(completion_event)
                                if self.verbose:
                                    print(f"    Added completion event for {event.instruction.name} in cycle {self.current_cycle}")
                            break
    
    def _parse_write_result(self, line: str):
        """Parse write result"""
        match = re.search(r'\[(\w+)\] Function unit \w+ write result to register', line)
        if match:
            unit_name = match.group(1)
            # Write-back event can be recorded here
            pass
    
    def _parse_memory_event(self, line: str):
        """Parse memory event"""
        # Memory access event parsing can be added here
        pass
    
    def _parse_memory_task_completed(self, line: str):
        """Parse memory unit task completion"""
        # Match new memory task completion format with instruction information
        # Format: "Read port X task completed: current_pos=Y/Z bytes, result buffer completed=true, instruction: <inst_info>"
        match = re.search(r'(Read|Write) port (\d+) task completed:.*?instruction:\s*(.+)', line)
        if match:
            port_type = match.group(1)
            port_id = int(match.group(2))
            instruction_info = match.group(3).strip()
            
            if self.verbose:
                print(f"    Found memory task completion: {port_type} port {port_id} in cycle {self.current_cycle}")
                print(f"    Instruction info: {instruction_info}")
            
            # Try to extract instruction details from the instruction info string
            # Example: "MemInst { dir: Read, reg: VectorRegister(1), raw: vle32.v v1, (a0), ... }"
            inst_name = None
            target_register = None
            
            # Extract instruction name from raw field if available
            raw_match = re.search(r'raw:\s*([^,}]+)', instruction_info)
            if raw_match:
                raw_inst = raw_match.group(1).strip()
                # Extract instruction name (first word)
                inst_name_match = re.match(r'(\S+)', raw_inst)
                if inst_name_match:
                    inst_name = inst_name_match.group(1)
            
            # Extract register information
            reg_match = re.search(r'reg:\s*(\w+)\((\d+)\)', instruction_info)
            if reg_match:
                reg_type = reg_match.group(1)
                reg_id = int(reg_match.group(2))
                if reg_type == "VectorRegister":
                    target_register = f"v{reg_id}"
                elif reg_type == "ScalarRegister":
                    target_register = f"x{reg_id}"
                elif reg_type == "FloatRegister":
                    target_register = f"f{reg_id}"
            
            if self.verbose and inst_name:
                print(f"    Parsed instruction: {inst_name}, register: {target_register}")
            
            # Find matching memory instruction based on the specific port and instruction info
            if self.instruction_events:
                memory_instructions = []
                # Collect all issued memory instructions that haven't been completed yet
                for event in self.instruction_events:
                    if (event.event_type == 'issued' and 
                        event.unit == 'MemoryUnit' and 
                        event.cycle < self.current_cycle):
                        
                        # Check if completion event already exists
                        already_completed = any(
                            e.event_type == 'completed' and 
                            e.instruction.name == event.instruction.name and
                            e.pc == event.pc and
                            e.unit == event.unit
                            for e in self.instruction_events
                        )
                        
                        if not already_completed:
                            # Try to match by instruction name if available
                            if inst_name and event.instruction.name == inst_name:
                                memory_instructions.insert(0, event)  # Prioritize exact matches
                            elif not inst_name:
                                memory_instructions.append(event)
                
                # Sort by issue cycle to process in order (exact matches first)
                memory_instructions.sort(key=lambda x: x.cycle)
                
                if self.verbose:
                    print(f"    Found {len(memory_instructions)} uncompleted memory instructions:")
                    for i, inst in enumerate(memory_instructions):
                        print(f"      [{i}] {inst.instruction.name} (PC={inst.pc}, cycle={inst.cycle})")
                
                # Select the best matching instruction
                if len(memory_instructions) > 0:
                    target_event = memory_instructions[0]  # Take the best match (or oldest)
                    
                    completion_event = InstructionEvent(
                        cycle=self.current_cycle,
                        pc=target_event.pc,
                        instruction=target_event.instruction,
                        event_type='completed',
                        unit=target_event.unit
                    )
                    self.instruction_events.append(completion_event)
                    if self.verbose:
                        print(f"    Added completion event for {target_event.instruction.name} (PC={target_event.pc}) in cycle {self.current_cycle}")
                else:
                    if self.verbose:
                        print(f"    ERROR: Port {port_id} completion but no matching uncompleted instructions available")
        else:
            # Fallback to old format for compatibility
            old_match = re.search(r'(Read|Write) port (\d+) task completed:', line)
            if old_match:
                port_type = old_match.group(1)
                port_id = int(old_match.group(2))
                
                if self.verbose:
                    print(f"    Found memory task completion (old format): {port_type} port {port_id} in cycle {self.current_cycle}")
                
                # Use original logic for old format
                if self.instruction_events:
                    memory_instructions = []
                    for event in self.instruction_events:
                        if (event.event_type == 'issued' and 
                            event.unit == 'MemoryUnit' and 
                            event.cycle < self.current_cycle):
                            
                            already_completed = any(
                                e.event_type == 'completed' and 
                                e.instruction.name == event.instruction.name and
                                e.pc == event.pc and
                                e.unit == event.unit
                                for e in self.instruction_events
                            )
                            
                            if not already_completed:
                                memory_instructions.append(event)
                    
                    memory_instructions.sort(key=lambda x: x.cycle)
                    
                    if len(memory_instructions) > 0:
                        target_event = memory_instructions[0]
                        
                        completion_event = InstructionEvent(
                            cycle=self.current_cycle,
                            pc=target_event.pc,
                            instruction=target_event.instruction,
                            event_type='completed',
                            unit=target_event.unit
                        )
                        self.instruction_events.append(completion_event)
                        if self.verbose:
                            print(f"    Added completion event for {target_event.instruction.name} (PC={target_event.pc}) in cycle {self.current_cycle}")
                    else:
                        if self.verbose:
                            print(f"    ERROR: Port {port_id} completion but no uncompleted instructions available")
    
    def analyze_instruction_lifecycle(self):
        """Analyze instruction lifecycle"""
        print("\n" + "="*60)
        print("Instruction Lifecycle Analysis")
        print("="*60)
        
        # Group events by instruction
        instruction_lifecycle = defaultdict(list)
        
        for event in self.instruction_events:
            key = f"{event.instruction.name}_{event.pc}"
            instruction_lifecycle[key].append(event)
        
        # Statistical data
        total_latency = 0
        completed_instructions = 0
        latency_by_type = defaultdict(list)
        
        # Analyze each instruction's lifecycle
        for inst_key, events in instruction_lifecycle.items():
            events.sort(key=lambda x: x.cycle)
            
            print(f"\nInstruction: {inst_key}")
            issue_cycle = None
            complete_cycle = None
            unit = None
            
            for event in events:
                if event.event_type == 'issued':
                    issue_cycle = event.cycle
                    unit = event.unit
                    print(f"  Issue cycle: {event.cycle} (Functional unit: {event.unit})")
                elif event.event_type == 'completed':
                    complete_cycle = event.cycle
            
            if issue_cycle is not None and complete_cycle is not None:
                latency = complete_cycle - issue_cycle
                print(f"  Execution latency: {latency} cycles")
                
                # Statistics
                total_latency += latency
                completed_instructions += 1
                if unit:
                    latency_by_type[unit].append(latency)
        
        # Print statistics
        if completed_instructions > 0:
            avg_latency = total_latency / completed_instructions
            print(f"\nInstruction Latency Statistics:")
            print(f"  Completed instructions: {completed_instructions}")
            print(f"  Average latency: {avg_latency:.2f} cycles")
            
            print(f"\nLatency Statistics by Functional Unit:")
            for unit, latencies in latency_by_type.items():
                avg_unit_latency = sum(latencies) / len(latencies)
                print(f"  {unit}: Average {avg_unit_latency:.2f} cycles (Sample count: {len(latencies)})")
        
        # Generate instruction lifecycle Gantt chart
        self.generate_instruction_lifecycle_gantt()
    
    def generate_instruction_lifecycle_gantt(self):
        """Generate instruction lifecycle Gantt chart"""
        if not self.output_dir:
            return
            
        # Group events by instruction
        instruction_lifecycle = defaultdict(list)
        
        for event in self.instruction_events:
            key = f"{event.instruction.name}@{event.pc}"
            instruction_lifecycle[key].append(event)
        
        # Prepare data for Gantt chart
        gantt_data = []
        
        for inst_key, events in instruction_lifecycle.items():
            events.sort(key=lambda x: x.cycle)
            
            issue_cycle = None
            complete_cycle = None
            unit = None
            
            for event in events:
                if event.event_type == 'issued':
                    issue_cycle = event.cycle
                    unit = event.unit
                elif event.event_type == 'completed':
                    complete_cycle = event.cycle
            
            if issue_cycle is not None and complete_cycle is not None:
                duration = complete_cycle - issue_cycle
                gantt_data.append({
                    'instruction': inst_key,
                    'unit': unit,
                    'start': issue_cycle,
                    'duration': duration,
                    'end': complete_cycle
                })
        
        if not gantt_data:
            print("No completed instructions found for Gantt chart")
            return
        
        # Create Gantt chart
        fig, ax = plt.subplots(figsize=(14, max(8, len(gantt_data) * 0.5)))
        
        # Color mapping for different units
        units = list(set(item['unit'] for item in gantt_data))
        colors = ['#FF6B6B', '#4ECDC4', '#45B7D1', '#96CEB4', '#FFEAA7', '#DDA0DD', '#98D8C8', '#F7DC6F']
        unit_colors = {unit: colors[i % len(colors)] for i, unit in enumerate(units)}
        
        # Plot bars
        y_pos = range(len(gantt_data))
        for i, item in enumerate(gantt_data):
            # For vector instructions (especially VFMACC_VV), try to distinguish compute vs writeback phases
            if 'VFMACC_VV' in item['instruction'] and item['duration'] > 7:
                # Split into compute phase (first 7 cycles) and writeback phase (remaining)
                compute_duration = 7
                writeback_duration = item['duration'] - compute_duration
                
                # Plot compute phase (solid color)
                ax.barh(i, compute_duration, left=item['start'], 
                       color=unit_colors[item['unit']], alpha=0.9, 
                       edgecolor='black', linewidth=0.5, label='Compute' if i == 0 else "")
                
                # Plot writeback phase (striped pattern)
                ax.barh(i, writeback_duration, left=item['start'] + compute_duration, 
                       color=unit_colors[item['unit']], alpha=0.5, 
                       edgecolor='black', linewidth=0.5, hatch='///', label='Writeback' if i == 0 else "")
                
                # Add text labels for both phases
                ax.text(item['start'] + compute_duration/2, i, 
                       f"{compute_duration}c", 
                       ha='center', va='center', fontweight='bold', fontsize=7)
                ax.text(item['start'] + compute_duration + writeback_duration/2, i, 
                       f"{writeback_duration}c", 
                       ha='center', va='center', fontweight='bold', fontsize=7)
            else:
                # Regular instruction display
                ax.barh(i, item['duration'], left=item['start'], 
                       color=unit_colors[item['unit']], alpha=0.8, 
                       edgecolor='black', linewidth=0.5)
                
                # Add text labels
                ax.text(item['start'] + item['duration']/2, i, 
                       f"{item['duration']}c", 
                       ha='center', va='center', fontweight='bold', fontsize=8)
        
        # Customize the chart
        ax.set_yticks(y_pos)
        ax.set_yticklabels([item['instruction'] for item in gantt_data])
        ax.set_xlabel('Cycles')
        ax.set_ylabel('Instructions')
        ax.set_title('Instruction Lifecycle Gantt Chart', fontweight='bold')
        
        # Set X-axis range to show the full simulation period
        if self.total_cycles > 0:
            ax.set_xlim(0, self.total_cycles)
        else:
            # Fallback: use the maximum completion cycle + some padding
            max_end = max(item['end'] for item in gantt_data) if gantt_data else 20
            ax.set_xlim(0, max_end + 5)
        
        # Add legend
        legend_patches = [mpatches.Patch(color=unit_colors[unit], label=unit) for unit in units]
        ax.legend(handles=legend_patches, loc='upper right', bbox_to_anchor=(1.15, 1))
        
        # Grid for better readability
        ax.grid(True, alpha=0.3, axis='x')
        
        plt.tight_layout()
        
        # Save the image
        output_path = self.output_dir / 'instruction_lifecycle_gantt.png'
        plt.savefig(output_path, bbox_inches='tight', dpi=150, facecolor='white')
        plt.close()
        
        print(f"Instruction lifecycle Gantt chart saved to: {output_path}")
    
    def analyze_unit_utilization(self):
        """Analyze functional unit utilization"""
        print("\n" + "="*60)
        print("Functional Unit Utilization Analysis")
        print("="*60)
        
        # Group status by unit name
        unit_cycles = defaultdict(list)
        
        for status in self.unit_statuses:
            unit_cycles[status.unit_name].append((status.cycle, status.occupied, status.event_queue_size))
        
        # Calculate utilization statistics
        for unit_name in sorted(unit_cycles.keys()):
            cycles = unit_cycles[unit_name]
            cycles.sort(key=lambda x: x[0])
            
            total_cycles = len(cycles)
            occupied_cycles = sum(1 for _, occupied, _ in cycles if occupied)
            
            if total_cycles > 0:
                utilization = (occupied_cycles / total_cycles) * 100
                
                # Calculate average queue length
                avg_queue_size = sum(queue_size for _, _, queue_size in cycles) / total_cycles
                
                print(f"\n{unit_name}:")
                print(f"  Total cycles: {total_cycles}")
                print(f"  Occupied cycles: {occupied_cycles}")
                print(f"  Utilization: {utilization:.2f}%")
                print(f"  Average queue length: {avg_queue_size:.2f}")
                
                # Show utilization timeline
                print("  Utilization timeline:", end=" ")
                timeline_length = min(40, total_cycles)
                for i in range(timeline_length):
                    cycle, occupied, queue_size = cycles[i]
                    if occupied:
                        print("█", end="")
                    elif queue_size > 0:
                        print("▓", end="")  # Has queue but not occupied
                    else:
                        print("□", end="")
                        
                if total_cycles > timeline_length:
                    print("...")
                else:
                    print()
        
        # Generate unit utilization Gantt chart
        self.generate_unit_utilization_gantt()
    
    def generate_unit_utilization_gantt(self):
        """Generate functional unit utilization Gantt chart"""
        if not self.output_dir:
            return
            
        # Group status by unit name
        unit_cycles = defaultdict(list)
        
        for status in self.unit_statuses:
            unit_cycles[status.unit_name].append((status.cycle, status.occupied, status.event_queue_size))
        
        if not unit_cycles:
            print("No unit status data found for utilization Gantt chart")
            return
        
        # Create figure
        fig, ax = plt.subplots(figsize=(16, max(8, len(unit_cycles) * 0.8)))
        
        # Colors for different states
        occupied_color = '#FF4444'      # Red for occupied
        queued_color = '#FFAA44'        # Orange for queued but not occupied
        idle_color = '#CCCCCC'          # Light gray for idle
        
        # Plot utilization timeline for each unit
        y_positions = {}
        unit_names = sorted(unit_cycles.keys())
        
        for i, unit_name in enumerate(unit_names):
            y_positions[unit_name] = i
            cycles = unit_cycles[unit_name]
            cycles.sort(key=lambda x: x[0])
            
            # Create timeline bars
            for cycle, occupied, queue_size in cycles:
                if occupied:
                    color = occupied_color
                    label = 'Occupied'
                elif queue_size > 0:
                    color = queued_color
                    label = 'Queued'
                else:
                    color = idle_color
                    label = 'Idle'
                
                ax.barh(i, 1, left=cycle, color=color, alpha=0.8, 
                       edgecolor='white', linewidth=0.1)
        
        # Customize the chart
        ax.set_yticks(range(len(unit_names)))
        ax.set_yticklabels(unit_names)
        ax.set_xlabel('Cycles')
        ax.set_ylabel('Functional Units')
        ax.set_title('Functional Unit Utilization Timeline', fontweight='bold')
        
        # Create custom legend
        legend_elements = [
            mpatches.Patch(color=occupied_color, label='Occupied'),
            mpatches.Patch(color=queued_color, label='Queued'),
            mpatches.Patch(color=idle_color, label='Idle')
        ]
        ax.legend(handles=legend_elements, loc='upper right', bbox_to_anchor=(1.1, 1))
        
        # Add grid for better readability
        ax.grid(True, alpha=0.3, axis='x')
        
        # Set x-axis limits to show the full simulation period
        if self.total_cycles > 0:
            ax.set_xlim(0, self.total_cycles)
        elif unit_cycles:
            # Fallback: use the maximum recorded cycle + some padding
            max_cycle = max(max(cycle for cycle, _, _ in cycles) for cycles in unit_cycles.values())
            ax.set_xlim(0, max_cycle + 1)
        else:
            ax.set_xlim(0, 20)  # Default fallback
        
        plt.tight_layout()
        
        # Save the image
        output_path = self.output_dir / 'unit_utilization_gantt.png'
        plt.savefig(output_path, bbox_inches='tight', dpi=150, facecolor='white')
        plt.close()
        
        print(f"Unit utilization Gantt chart saved to: {output_path}")
        
        # Also generate utilization statistics bar chart
        self.generate_unit_utilization_stats()
    
    def generate_unit_utilization_stats(self):
        """Generate unit utilization statistics bar chart"""
        if not self.output_dir:
            return
            
        # Group status by unit name and calculate statistics
        unit_stats = {}
        unit_cycles = defaultdict(list)
        
        for status in self.unit_statuses:
            unit_cycles[status.unit_name].append((status.cycle, status.occupied, status.event_queue_size))
        
        for unit_name, cycles in unit_cycles.items():
            total_cycles = len(cycles)
            occupied_cycles = sum(1 for _, occupied, _ in cycles if occupied)
            avg_queue_size = sum(queue_size for _, _, queue_size in cycles) / total_cycles if total_cycles > 0 else 0
            utilization = (occupied_cycles / total_cycles) * 100 if total_cycles > 0 else 0
            
            unit_stats[unit_name] = {
                'utilization': utilization,
                'avg_queue_size': avg_queue_size,
                'occupied_cycles': occupied_cycles,
                'total_cycles': total_cycles
            }
        
        if not unit_stats:
            return
        
        # Create figure with subplots
        fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(15, 6))
        fig.suptitle('Functional Unit Utilization Statistics', fontsize=14, fontweight='bold')
        
        units = list(unit_stats.keys())
        utilizations = [unit_stats[unit]['utilization'] for unit in units]
        queue_sizes = [unit_stats[unit]['avg_queue_size'] for unit in units]
        
        # Utilization bar chart
        bars1 = ax1.bar(units, utilizations, color='#4CAF50', alpha=0.8)
        ax1.set_title('Unit Utilization Percentage', fontweight='bold')
        ax1.set_ylabel('Utilization (%)')
        ax1.tick_params(axis='x', rotation=45)
        
        # Add value labels on bars
        for bar, util in zip(bars1, utilizations):
            ax1.text(bar.get_x() + bar.get_width()/2., bar.get_height() + 1,
                    f'{util:.1f}%', ha='center', va='bottom', fontsize=9)
        
        # Average queue size bar chart
        bars2 = ax2.bar(units, queue_sizes, color='#FF9800', alpha=0.8)
        ax2.set_title('Average Queue Size', fontweight='bold')
        ax2.set_ylabel('Average Queue Size')
        ax2.tick_params(axis='x', rotation=45)
        
        # Add value labels on bars
        for bar, queue in zip(bars2, queue_sizes):
            ax2.text(bar.get_x() + bar.get_width()/2., bar.get_height() + 0.05,
                    f'{queue:.2f}', ha='center', va='bottom', fontsize=9)
        
        plt.tight_layout()
        
        # Save the image
        output_path = self.output_dir / 'unit_utilization_stats.png'
        plt.savefig(output_path, bbox_inches='tight', dpi=150, facecolor='white')
        plt.close()
        
        print(f"Unit utilization statistics saved to: {output_path}")
    
    def analyze_pipeline_performance(self):
        """Analyze pipeline performance"""
        print("\n" + "="*60)
        print("Pipeline Performance Analysis")
        print("="*60)
        
        # Count instructions issued per cycle
        instructions_per_cycle = defaultdict(int)
        completed_per_cycle = defaultdict(int)
        
        for event in self.instruction_events:
            if event.event_type == 'issued':
                instructions_per_cycle[event.cycle] += 1
            elif event.event_type == 'completed':
                completed_per_cycle[event.cycle] += 1
        
        if instructions_per_cycle:
            max_cycle = max(max(instructions_per_cycle.keys()), 
                          max(completed_per_cycle.keys()) if completed_per_cycle else 0)
            total_instructions = sum(instructions_per_cycle.values())
            total_completed = sum(completed_per_cycle.values())
            total_cycles = max_cycle + 1
            
            print(f"Total instructions: {total_instructions}")
            print(f"Completed instructions: {total_completed}")
            print(f"Total cycles: {total_cycles}")
            print(f"Average IPC (Instructions Per Cycle): {total_instructions/total_cycles:.3f}")
            
            if total_completed > 0:
                print(f"Average completion rate: {total_completed/total_cycles:.3f} instructions/cycle")
                throughput = total_completed / total_cycles
                print(f"Throughput: {throughput:.3f} instructions/cycle")
            
            # Display per-cycle instruction issue statistics
            print(f"\nPer-cycle instruction issue and completion:")
            display_cycles = min(30, max_cycle + 1)
            for cycle in range(display_cycles):
                issued = instructions_per_cycle.get(cycle, 0)
                completed = completed_per_cycle.get(cycle, 0)
                print(f"  Cycle {cycle:2d}: issued {issued}, completed {completed}")
                
            if max_cycle + 1 > display_cycles:
                print(f"  ... (omitting remaining {max_cycle + 1 - display_cycles} cycles)")
    
    def analyze_instruction_types(self):
        """Analyze instruction type distribution"""
        print("\n" + "="*60)
        print("Instruction Type Analysis")
        print("="*60)
        
        # Count instruction types
        instruction_types = defaultdict(int)
        vector_instructions = 0
        scalar_instructions = 0
        
        for inst in self.instructions:
            instruction_types[inst.name] += 1
            
            # Classify vector and scalar instructions
            if inst.name.startswith('V') or inst.vrd is not None or inst.vrs1 is not None:
                vector_instructions += 1
            else:
                scalar_instructions += 1
        
        print(f"Instruction Type Distribution:")
        for inst_type, count in sorted(instruction_types.items()):
            percentage = (count / len(self.instructions)) * 100 if self.instructions else 0
            print(f"  {inst_type}: {count} instructions ({percentage:.1f}%)")
        
        print(f"\nInstruction Classification:")
        print(f"  Scalar instructions: {scalar_instructions} instructions")
        print(f"  Vector instructions: {vector_instructions} instructions")
    
    def print_config(self):
        """Print complete simulator configuration"""
        print("\n" + "="*70)
        print("Simulator Configuration")
        print("="*70)
        
        # Function Units Configuration
        print("\nFunction Units Configuration:")
        config = self.config.function_unit_config
        if any([config.interger_alu, config.interger_multiplier, config.float_alu, 
               config.float_multiplier, config.interger_divider, config.float_divider, config.branch_unit]):
            print(f"  Integer ALU latency: {config.interger_alu} cycles")
            print(f"  Integer Multiplier latency: {config.interger_multiplier} cycles")
            print(f"  Float ALU latency: {config.float_alu} cycles")
            print(f"  Float Multiplier latency: {config.float_multiplier} cycles")
            print(f"  Integer Divider latency: {config.interger_divider} cycles")
            print(f"  Float Divider latency: {config.float_divider} cycles")
            print(f"  Branch Unit latency: {config.branch_unit} cycles")
        else:
            print("  No function unit configuration found")
        
        # Memory Units Configuration
        print("\nMemory Units Configuration:")
        mem_config = self.config.memory_unit_config
        if any([mem_config.latency, mem_config.max_access_width, mem_config.read_ports_limit, mem_config.write_ports_limit]):
            print(f"  Load/Store Unit latency: {mem_config.latency} cycles")
            print(f"  Maximum access width: {mem_config.max_access_width} bits")
            print(f"  Read ports limit: {mem_config.read_ports_limit}")
            print(f"  Write ports limit: {mem_config.write_ports_limit}")
        else:
            print("  No memory unit configuration found")
        
        # Vector Configuration
        print("\nVector Configuration:")
        vec_soft = self.config.vector_software_config
        vec_hard = self.config.vector_hardware_config
        
        if any([vec_soft.vl, vec_soft.sew, vec_soft.lmul]):
            print("  Software settings:")
            print(f"    Vector Length (vl): {vec_soft.vl}")
            print(f"    Scalar Element Width (sew): {vec_soft.sew} bits")
            print(f"    Lane Multiplier (lmul): {vec_soft.lmul}")
        else:
            print("  Software settings: No configuration found")
            
        if any([vec_hard.vlen, vec_hard.lane_number]):
            print("  Hardware settings:")
            print(f"    Vector Register Length (vlen): {vec_hard.vlen} bits")
            print(f"    Vector Lane Number: {vec_hard.lane_number}")
        else:
            print("  Hardware settings: No configuration found")
        
        # Derived Values
        if any([self.config.vector_register_size_bytes, self.config.element_size_bytes, 
               self.config.total_vector_operation_size_bytes]):
            print("  Derived values:")
            if self.config.vector_register_size_bytes:
                print(f"    Vector Register Size: {self.config.vector_register_size_bytes} bytes")
            if self.config.element_size_bytes:
                print(f"    Element Size: {self.config.element_size_bytes} bytes")
            if self.config.total_vector_operation_size_bytes:
                print(f"    Total Vector Operation Size: {self.config.total_vector_operation_size_bytes} bytes")
        
        # Vector Register Configuration
        print("\nVector Register Configuration:")
        vec_reg = self.config.vector_register_config
        if any([vec_reg.read_ports_limit, vec_reg.write_ports_limit]):
            print(f"  Read ports limit: {vec_reg.read_ports_limit}")
            print(f"  Write ports limit: {vec_reg.write_ports_limit}")
        else:
            print("  No vector register configuration found")
        
        # Buffer Configuration
        print("\nBuffer Configuration:")
        buf_config = self.config.buffer_config
        if any([buf_config.input_maximum_size, buf_config.result_maximum_size]):
            print(f"  Input buffer maximum size: {buf_config.input_maximum_size}")
            print(f"  Result buffer maximum size: {buf_config.result_maximum_size}")
        else:
            print("  No buffer configuration found")
        
        # Register Configuration
        print("\nRegister Configuration:")
        reg_config = self.config.register_config
        if reg_config.maximum_forward_bytes:
            print(f"  Maximum forward bytes: {reg_config.maximum_forward_bytes}")
        else:
            print("  No register configuration found")
        
        # Generate configuration table image
        self.generate_config_table_image()
    
    def generate_config_table_image(self):
        """Generate simulator configuration table as image"""
        if not self.output_dir:
            return
            
        # Prepare configuration data
        config_data = []
        
        # Function Units Configuration
        config_data.append(['Configuration Category', 'Item', 'Value'])
        config_data.append(['Function Units', 'Integer ALU Latency', f'{self.config.function_unit_config.interger_alu} cycles'])
        config_data.append(['', 'Integer Multiplier Latency', f'{self.config.function_unit_config.interger_multiplier} cycles'])
        config_data.append(['', 'Float ALU Latency', f'{self.config.function_unit_config.float_alu} cycles'])
        config_data.append(['', 'Float Multiplier Latency', f'{self.config.function_unit_config.float_multiplier} cycles'])
        config_data.append(['', 'Integer Divider Latency', f'{self.config.function_unit_config.interger_divider} cycles'])
        config_data.append(['', 'Float Divider Latency', f'{self.config.function_unit_config.float_divider} cycles'])
        config_data.append(['', 'Branch Unit Latency', f'{self.config.function_unit_config.branch_unit} cycles'])
        
        # Memory Units Configuration
        mem_config = self.config.memory_unit_config
        config_data.append(['Memory Units', 'Load/Store Unit Latency', f'{mem_config.latency} cycles'])
        config_data.append(['', 'Maximum Access Width', f'{mem_config.max_access_width} bits'])
        config_data.append(['', 'Read Ports Limit', f'{mem_config.read_ports_limit}'])
        config_data.append(['', 'Write Ports Limit', f'{mem_config.write_ports_limit}'])
        
        # Vector Configuration
        vec_soft = self.config.vector_software_config
        vec_hard = self.config.vector_hardware_config
        config_data.append(['Vector Software', 'Vector Length (vl)', f'{vec_soft.vl}'])
        config_data.append(['', 'Scalar Element Width (sew)', f'{vec_soft.sew} bits'])
        config_data.append(['', 'Lane Multiplier (lmul)', f'{vec_soft.lmul}'])
        config_data.append(['Vector Hardware', 'Vector Register Length (vlen)', f'{vec_hard.vlen} bits'])
        config_data.append(['', 'Vector Lane Number', f'{vec_hard.lane_number}'])
        
        # Vector Register Configuration
        vec_reg = self.config.vector_register_config
        config_data.append(['Vector Register', 'Read Ports Limit', f'{vec_reg.read_ports_limit}'])
        config_data.append(['', 'Write Ports Limit', f'{vec_reg.write_ports_limit}'])
        
        # Buffer Configuration
        buf_config = self.config.buffer_config
        config_data.append(['Buffer', 'Input Buffer Maximum Size', f'{buf_config.input_maximum_size}'])
        config_data.append(['', 'Result Buffer Maximum Size', f'{buf_config.result_maximum_size}'])
        
        # Register Configuration
        reg_config = self.config.register_config
        config_data.append(['Register', 'Maximum Forward Bytes', f'{reg_config.maximum_forward_bytes}'])
        
        # Derived Values
        if any([self.config.vector_register_size_bytes, self.config.element_size_bytes, 
               self.config.total_vector_operation_size_bytes]):
            config_data.append(['Derived Values', 'Vector Register Size', f'{self.config.vector_register_size_bytes} bytes'])
            config_data.append(['', 'Element Size', f'{self.config.element_size_bytes} bytes'])
            config_data.append(['', 'Total Vector Operation Size', f'{self.config.total_vector_operation_size_bytes} bytes'])
        
        # Create figure and table
        fig, ax = plt.subplots(figsize=(12, 8))
        ax.axis('tight')
        ax.axis('off')
        
        # Create table
        table = ax.table(cellText=config_data[1:], 
                        colLabels=config_data[0],
                        cellLoc='left',
                        loc='center')
        
        # Style the table
        table.auto_set_font_size(False)
        table.set_fontsize(9)
        table.scale(1.2, 1.5)
        
        # Style header
        for i in range(len(config_data[0])):
            table[(0, i)].set_facecolor('#4CAF50')
            table[(0, i)].set_text_props(weight='bold', color='white')
        
        # Style category rows
        current_category = ''
        for i, row in enumerate(config_data[1:], 1):
            if row[0] and row[0] != current_category:
                current_category = row[0]
                table[(i, 0)].set_facecolor('#E8F5E8')
                table[(i, 0)].set_text_props(weight='bold')
            elif not row[0]:  # Empty category cell
                table[(i, 0)].set_facecolor('#F5F5F5')
        
        plt.title('Simulator Configuration', fontsize=16, fontweight='bold', pad=20)
        
        # Save the image
        output_path = self.output_dir / 'simulator_configuration.png'
        plt.savefig(output_path, bbox_inches='tight', dpi=150, facecolor='white')
        plt.close()
        
        print(f"Configuration table saved to: {output_path}")
    
    def print_summary(self):
        """Print summary information"""
        print("\n" + "="*60)
        print("Parsing Summary")
        print("="*60)
        
        print(f"Total instructions parsed: {len(self.instructions)}")
        print(f"Total instruction events: {len(self.instruction_events)}")
        print(f"Functional unit status records: {len(self.unit_statuses)}")
        print(f"Detected functional units: {', '.join(sorted(self.function_units))}")
        
        if self.total_cycles > 0:
            print(f"Total simulation cycles: {self.total_cycles}")
        elif self.current_cycle > 0:
            print(f"Last processed cycle: {self.current_cycle + 1}")
        else:
            print("No cycle information detected")
        
        # Generate summary charts
        self.generate_summary_charts()

    def generate_summary_charts(self):
        """Generate parsing summary and instruction type analysis charts"""
        if not self.output_dir:
            return
            
        # Create figure with subplots
        fig, ((ax1, ax2), (ax3, ax4)) = plt.subplots(2, 2, figsize=(15, 10))
        fig.suptitle('Parsing Summary and Instruction Analysis', fontsize=16, fontweight='bold')
        
        # 1. Parsing Summary Bar Chart
        summary_labels = ['Total Instructions', 'Instruction Events', 'Unit Status Records', 'Detected Units']
        summary_values = [len(self.instructions), len(self.instruction_events), 
                         len(self.unit_statuses), len(self.function_units)]
        
        bars1 = ax1.bar(summary_labels, summary_values, color=['#2196F3', '#4CAF50', '#FF9800', '#9C27B0'])
        ax1.set_title('Parsing Summary', fontweight='bold')
        ax1.set_ylabel('Count')
        
        # Add value labels on bars
        for bar in bars1:
            height = bar.get_height()
            ax1.text(bar.get_x() + bar.get_width()/2., height,
                    f'{int(height)}', ha='center', va='bottom')
        
        # Rotate x-axis labels for better readability
        ax1.tick_params(axis='x', rotation=45)
        
        # 2. Instruction Type Distribution Pie Chart
        instruction_types = defaultdict(int)
        for inst in self.instructions:
            instruction_types[inst.name] += 1
        
        if instruction_types:
            labels = list(instruction_types.keys())
            sizes = list(instruction_types.values())
            # Generate distinct colors
            colors = ['#FF6B6B', '#4ECDC4', '#45B7D1', '#96CEB4', '#FFEAA7', '#DDA0DD', '#98D8C8', '#F7DC6F']
            colors = colors[:len(labels)]  # Use only as many colors as needed
            
            wedges, texts, autotexts = ax2.pie(sizes, labels=labels, autopct='%1.1f%%', 
                                              colors=colors, startangle=90)
            ax2.set_title('Instruction Type Distribution', fontweight='bold')
            
            # Make percentage text more readable
            for autotext in autotexts:
                autotext.set_color('white')
                autotext.set_fontweight('bold')
        
        # 3. Scalar vs Vector Instructions
        vector_instructions = sum(1 for inst in self.instructions 
                                 if inst.name.startswith('V') or inst.vrd is not None or inst.vrs1 is not None)
        scalar_instructions = len(self.instructions) - vector_instructions
        
        categories = ['Scalar', 'Vector']
        counts = [scalar_instructions, vector_instructions]
        colors = ['#FF6B6B', '#4ECDC4']
        
        bars3 = ax3.bar(categories, counts, color=colors)
        ax3.set_title('Instruction Classification', fontweight='bold')
        ax3.set_ylabel('Count')
        
        # Add value labels
        for bar in bars3:
            height = bar.get_height()
            ax3.text(bar.get_x() + bar.get_width()/2., height,
                    f'{int(height)}', ha='center', va='bottom')
        
        # 4. Cycle Information
        cycle_info = []
        cycle_labels = []
        
        if self.total_cycles > 0:
            cycle_info.append(self.total_cycles)
            cycle_labels.append('Total Cycles')
        
        if self.current_cycle > 0:
            cycle_info.append(self.current_cycle)
            cycle_labels.append('Last Processed Cycle')
        
        if cycle_info:
            bars4 = ax4.bar(cycle_labels, cycle_info, color="#A21ABA")
            ax4.set_title('Cycle Information', fontweight='bold')
            ax4.set_ylabel('Cycles')
            
            # Add value labels
            for bar in bars4:
                height = bar.get_height()
                ax4.text(bar.get_x() + bar.get_width()/2., height,
                        f'{int(height)}', ha='center', va='bottom')
        else:
            ax4.text(0.5, 0.5, 'No Cycle Information', ha='center', va='center',
                    transform=ax4.transAxes, fontsize=12)
            ax4.set_title('Cycle Information', fontweight='bold')
        
        plt.tight_layout()
        
        # Save the image
        output_path = self.output_dir / 'summary_and_instruction_analysis.png'
        plt.savefig(output_path, bbox_inches='tight', dpi=150, facecolor='white')
        plt.close()
        
        print(f"Summary and instruction analysis charts saved to: {output_path}")

def main():
    """Main function"""
    parser = argparse.ArgumentParser(description='RISC-V Vector Simulator Log Parser')
    parser.add_argument('log_file', help='Log file path')
    parser.add_argument('-v', '--verbose', action='store_true', help='Verbose output')
    parser.add_argument('--no-lifecycle', action='store_true', help='Skip instruction lifecycle analysis')
    parser.add_argument('--no-utilization', action='store_true', help='Skip functional unit utilization analysis')
    parser.add_argument('--no-images', action='store_true', help='Skip image generation')
    # parser.add_argument('--no-performance', action='store_true', help='Skip pipeline performance analysis')
    
    args = parser.parse_args()
    
    try:
        log_parser = LogParser()
        log_parser.verbose = args.verbose
        
        # If no-images is specified, mark to skip output directory setup
        log_parser.skip_images = args.no_images
        
        log_parser.parse_log_file(args.log_file)
        
        # Execute various analyses
        log_parser.print_config()     # Print configuration first
        log_parser.print_summary()
        log_parser.analyze_instruction_types()
        
        if not args.no_lifecycle:
            log_parser.analyze_instruction_lifecycle()
        
        if not args.no_utilization:
            log_parser.analyze_unit_utilization()
        
        # if not args.no_performance:
            # log_parser.analyze_pipeline_performance()
        
        if not args.no_images and log_parser.output_dir:
            print(f"\nAll images have been saved to: {log_parser.output_dir}")
        
    except FileNotFoundError:
        print(f"Error: Log file '{args.log_file}' not found")
    except Exception as e:
        print(f"Parsing error: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    main()
