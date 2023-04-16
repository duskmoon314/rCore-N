import struct

trace = {
    0x57AB: {"name": "S trap", "record": []},
    0x5CED: {"name": "scheduler", "record": []},
    0xC7AB: {"name": "U trap", "record": []},
    0x575C: {"name": "syscall", "record": []},
    0x5B1C: {"name": "SBI call", "record": []},
    0x315C: {"name": "misc", "record": []},
}

if __name__ == "__main__":
    with open("trace.bin", "rb", buffering=0x100000) as f:
        while True:
            record_bytes = f.read(16)
            if len(record_bytes) < 16:
                break
            (event, cycle) = struct.unpack("<QQ", record_bytes)
            cat = (event >> 16) & 0xFFFF
            trace[cat]["record"] += [cycle]
        for t in trace.values():
            print("name: {}, record num: {}".format(t["name"], len(t["record"])))
