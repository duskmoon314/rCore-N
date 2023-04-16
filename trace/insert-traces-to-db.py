import struct
import sqlite3 as sl
from event_def import event_map, pid, hartid, event_type, event_subtype, extra

trace_db = sl.connect("trace.sqlite")

if __name__ == "__main__":
    with trace_db:
        trace_db.execute("DROP TABLE IF EXISTS trace;")
        trace_db.execute(
            """
            CREATE TABLE trace (
                id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
                cycle INTEGER,
                hartid INTEGER,
                pid INTEGER,
                event TEXT,
                sub_event TEXT,
                extra TEXT
            );
        """
        )
        cmd = "INSERT INTO trace (id, cycle, hartid, pid, event, sub_event, extra) values (?, ?, ?, ?, ?, ?, ?)"
        with open("trace.bin", "rb", buffering=0x10000) as f:
            record_id = 0
            while True:
                record_bytes = f.read(16)
                if len(record_bytes) < 16:
                    break
                (e, c) = struct.unpack("<QQ", record_bytes)

                # sys flush trace
                if event_type(e) == 0x575C and extra(e) == 555:
                    break

                event = event_map[event_type(e)]
                sub_event = event["sub_event"][event_subtype(e)]
                try:
                    record = [
                        (
                            record_id,
                            c,
                            hartid(e),
                            pid(e),
                            event["name"],
                            sub_event["name"],
                            str(extra(e)),
                        )
                    ]
                except Exception as exc:
                    print(
                        "error record: 0x{:x}, cycle: {}, event: {}".format(e, c, event)
                    )
                    raise exc

                try:
                    trace_db.executemany(cmd, record)
                except Exception as exc:
                    print(
                        "error insert: 0x{:x}, cycle: {}, event: {}".format(e, c, event)
                    )
                    raise exc
                record_id += 1
            print("{} records inserted into db".format(record_id))
