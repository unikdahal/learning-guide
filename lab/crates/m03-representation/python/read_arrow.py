"""Article: "Apache Arrow: the columnar in-memory standard".

Reads the files written by `cargo run --release -p m03-representation --example arrow_ipc`
from Python with pyarrow: the Arrow IPC file by memory-mapping it, and the CSV by parsing it.
Checks that the memory-mapped column points into the file mapping: zero copies.

    pip install pyarrow
    python3 crates/m03-representation/python/read_arrow.py
"""

import os
import time

import pyarrow as pa
import pyarrow.compute as pc
import pyarrow.csv as pacsv

here = os.path.dirname(os.path.abspath(__file__))
data = os.path.normpath(os.path.join(here, "..", "..", "..", "data"))
print(f"pyarrow {pa.__version__}\n")

t = time.perf_counter()
source = pa.memory_map(os.path.join(data, "m03-demo.arrow"), "r")
table = pa.ipc.open_file(source).read_all()
ipc_s = time.perf_counter() - t
print(f"Arrow IPC, memory-mapped: {table.num_rows:,} rows in {ipc_s * 1e3:.2f} ms")
print(f"  schema: {table.schema.names} {[str(t) for t in table.schema.types]}")
buf = table.column("value").chunk(0).buffers()[1]
source.seek(0)
whole = source.read_buffer(source.size())  # a view of the mapping itself, not a copy
inside = whole.address <= buf.address < whole.address + whole.size
print(f"  file mapped at {whole.address:#x}, {whole.size:,} bytes")
print(f"  value column's buffer at {buf.address:#x}, {buf.size:,} bytes: inside the mapping = {inside}")
print(f"  pyarrow total_allocated_bytes after reading: {pa.total_allocated_bytes():,}")

t = time.perf_counter()
s = pc.sum(table.column("value")).as_py()
print(f"  sum(value) = {s:,.1f} in {(time.perf_counter() - t) * 1e3:.1f} ms\n")

t = time.perf_counter()
csv_table = pacsv.read_csv(os.path.join(data, "m03-demo.csv"))
csv_s = time.perf_counter() - t
print(f"CSV, parsed by pyarrow:   {csv_table.num_rows:,} rows in {csv_s * 1e3:.0f} ms")
print(f"  schema: {csv_table.schema.names} {[str(t) for t in csv_table.schema.types]}")
print(f"  sum(value) = {pc.sum(csv_table.column('value')).as_py():,.1f}")
print(f"  pyarrow total_allocated_bytes after parsing: {pa.total_allocated_bytes():,}")
print(f"\nThe IPC read was {csv_s / ipc_s:,.0f} times faster.")
