BinaryView2 :: Executing disassember / decompiler

== Overview ==
The basic idea of this project is to run code in a limited VM, and record the disassembled instructions for each address. The VM keeps track of register states ranging from constant known (e.g. just read from ROM, or loaded with immediate) to fully unknown (never written,. or loaded from unknown memory). The eventual plan is to include a large range of value ranges, allowing the system to catch (for example) jump table addresses.

== Structure ==


