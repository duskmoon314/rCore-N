from collections import defaultdict
import matplotlib.pyplot as plt
import copy
from numpy import percentile
import sqlite3 as sl

from event_def import (
    event_map,
    pid,
    hartid,
    event_type,
    event_subtype,
    cause_intr,
    cause_excep,
    serial_call_name,
    syscall_name,
    trap_cause_name,
)

kernel_pid = {3, 4, 11, 12}

accept_pid = {3, 4, 5, 6, 7, 8, 11, 12, 13, 14, 15, 16}


def filter_outlier(data, factor):
    q25, q75 = percentile(data, 25), percentile(data, 75)
    iqr = q75 - q25
    cut_off = iqr * factor
    lower, upper = q25 - cut_off, q75 + cut_off
    # lower, upper = percentile(data, 0.1), percentile(data, 99.9)
    return [x for x in data if x >= lower and x <= upper]


def trap_rec_stat(rec_dict, enter_id, exit_id):
    trap_stat = defaultdict(list)
    for k1 in rec_dict.keys():
        for cause in rec_dict[k1].keys():
            print(
                "key1: {}, cause: {}, num: {}".format(
                    k1, trap_cause_name(cause), len(rec_dict[k1][cause])
                )
            )
            it1 = iter(rec_dict[k1][cause])
            for e1, c1 in it1:
                # s trap handler
                if event_subtype(e1) == enter_id:
                    it2 = copy.copy(it1)
                    for e2, c2 in it2:
                        if event_subtype(e2) == exit_id:
                            trap_stat[cause].append(c2 - c1)
                            break

    return {cause: filter_outlier(stat, 2) for cause, stat in trap_stat.items()}


def syscall_stat(rec_dict, enter_id, exit_id):
    syscall_stat = defaultdict(list)
    for k1 in rec_dict.keys():
        for sid in rec_dict[k1].keys():
            print(
                "key1: {}, syscall: {}, num: {}".format(
                    k1, syscall_name[sid], len(rec_dict[k1][sid])
                )
            )
            it1 = iter(rec_dict[k1][sid])
            for e1, c1 in it1:
                # s trap handler
                if event_subtype(e1) == enter_id:
                    it2 = copy.copy(it1)
                    for e2, c2 in it2:
                        if event_subtype(e2) == exit_id:
                            syscall_stat[sid].append(c2 - c1)
                            break

    return {sid: filter_outlier(stat, 2) for sid, stat in syscall_stat.items()}


def serial_stat(rec_dict, enter_id, exit_id):
    serial_stat = defaultdict(list)
    for k1 in rec_dict.keys():
        for sid in rec_dict[k1].keys():
            print(
                "key1: {}, serial call: {}, num: {}".format(
                    k1, serial_call_name[sid], len(rec_dict[k1][sid])
                )
            )
            it1 = iter(rec_dict[k1][sid])
            for e1, c1 in it1:
                # s trap handler
                if event_subtype(e1) == enter_id:
                    it2 = copy.copy(it1)
                    for e2, c2 in it2:
                        if event_subtype(e2) == exit_id:
                            serial_stat[sid].append(c2 - c1)
                            break

    return {sid: filter_outlier(stat, 2) for sid, stat in serial_stat.items()}


if __name__ == "__main__":
    trace_db = sl.connect("trace.sqlite")
    trace_db.execute("DROP TABLE IF EXISTS trace_filter_tx;")
    # no syscall
    trace_db.execute(
        """
        CREATE TABLE trace_filter_tx AS
            SELECT *
            FROM trace
            WHERE NOT event='syscall'
                AND NOT (event='S trap' AND (extra='8'))
                AND pid=24
    """
    )

    trace_db.execute("DROP TABLE IF EXISTS trace_filter_rx;")
    # no syscall
    trace_db.execute(
        """
        CREATE TABLE trace_filter_rx AS
            SELECT *
            FROM trace
            WHERE NOT event='syscall'
                AND NOT (event='S trap' AND (extra='8'))
                AND pid=23
    """
    )
