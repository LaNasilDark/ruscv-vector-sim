本文档用于记录自从jiaqi处接手模拟器项目后所作的主要更改：

1. bug修复
   i.8月27日，修复了sd指令导致模拟器无限循环的bug，主要涉及文件：memory_unit.rs
   ii.10月8日，修复了memory instruction issue时机错误的bug，主要涉及文件：sim.rs和register.rs
   iii.10月30日，修复了vse指令无法参与chaining的bug，主要涉及文件：register.rs
2. 指令添加
   i.9月17日，添加vsetivli指令支持
   ii.10月16日，添加vadd.vv,vmul.vx,vmul.vv,vredsum.vs,vadd.vx,vsub.vv指令支持
3. 其他更改
   i.9月13日，删除有关github submodules有关的功能，将vendor文件夹下的内容作为常规文件夹管理
   ii.9月13日，编写python脚本的log分析工具log_parser.py
