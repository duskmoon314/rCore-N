import struct
from collections import defaultdict
import matplotlib.pyplot as plt
import copy
from numpy import percentile

trace = {
    0x57AB: {"name": "S trap", "record": []},
    0x5CED: {"name": "scheduler", "record": []},
    0xC7AB: {"name": "U trap", "record": []},
    0x575C: {"name": "syscall", "record": []},
    0x5B1C: {"name": "SBI call", "record": []},
    0x315C: {"name": "misc", "record": []},
}

cause_intr = {
    0: "usi",
    1: "ssi",
    2: "hsi",
    3: "msi",
    4: "uti",
    5: "sti",
    6: "hti",
    7: "mti",
    8: "uei",
    9: "sei",
    10: "hei",
    11: "mei",
}

cause_excep = {
    0: "InstAddrMisaligned",
    8: "u-ecall",
    9: "s-ecall",
    11: "m-ecall",
    13: "LoadPageFault",
    15: "StorePageFault",
}

syscall_name = {
    57: "CLOSE",
    59: "PIPE",
    63: "READ",
    64: "WRITE",
    93: "EXIT",
    124: "YIELD",
    140: "SET_PRIORITY",
    169: "GET_TIME",
    172: "GETPID",
    215: "MUNMAP",
    220: "FORK",
    221: "EXEC",
    222: "MMAP",
    260: "WAITPID",
    400: "SPAWN",
    401: "MAILREAD",
    402: "MAILWRITE",
    555: "FLUSH_TRACE",
    600: "INIT_USER_TRAP",
    601: "SEND_MSG",
    602: "SET_TIMER",
    603: "CLAIM_EXT_INT",
    604: "SET_EXT_INT_ENABLE",
}


def filter_outlier_iqr(data, factor):
    q25, q75 = percentile(data, 25), percentile(data, 75)
    iqr = q75 - q25
    cut_off = iqr * factor
    lower, upper = q25 - cut_off, q75 + cut_off
    return [x for x in data if x >= lower and x <= upper]


def event_type(eid):
    return (eid >> 16) & 0xFFFF


def event_subtype(eid):
    return (eid >> 12) & 0xF


def hartid(eid):
    return (eid >> 32) & 0xF


def pid(eid):
    return (eid >> 36) & 0xFF


def extra(eid):
    return eid & 0xFFFFF00000000FFF


def trap_cause_name(cause):
    return cause_excep[cause] if cause < 64 else cause_intr[cause & 0xF]


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
            for (e1, c1) in it1:
                # s trap handler
                if event_subtype(e1) == enter_id:
                    it2 = copy.copy(it1)
                    for (e2, c2) in it2:
                        if event_subtype(e2) == exit_id:
                            trap_stat[cause].append(c2 - c1)
                            break

    return {cause: filter_outlier_iqr(stat, 3) for cause, stat in trap_stat.items()}


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
            for (e1, c1) in it1:
                # s trap handler
                if event_subtype(e1) == enter_id:
                    it2 = copy.copy(it1)
                    for (e2, c2) in it2:
                        if event_subtype(e2) == exit_id:
                            syscall_stat[sid].append(c2 - c1)
                            break

    return {sid: filter_outlier_iqr(stat, 3) for sid, stat in syscall_stat.items()}


if __name__ == "__main__":
    s_trap = {}
    u_trap = {}
    syscall = {}
    for hart in range(4):
        s_trap[hart] = defaultdict(list)

    with open("trace-bench.bin", "rb", buffering=0x10000) as f:
        while True:
            record_bytes = f.read(16)
            if len(record_bytes) < 16:
                break
            (e, c) = struct.unpack("<QQ", record_bytes)

            # sys flush trace
            if event_type(e) == 0x575C and extra(e) == 555:
                break

            # S trap enter and return
            if (
                event_type(e) == 0x57AB
                and (event_subtype(e) == 2 or event_subtype(e) == 3)
                and extra(e) != 8
            ):
                s_trap[hartid(e)][extra(e)].append((e, c))

            # U trap enter and return
            if event_type(e) == 0xC7AB and (
                event_subtype(e) == 8 or event_subtype(e) == 9
            ):
                p = pid(e)
                if p not in u_trap:
                    u_trap[p] = defaultdict(list)
                u_trap[p][extra(e)].append((e, c))

            # syscall
            if event_type(e) == 0x575C:
                p = pid(e)
                if p not in syscall:
                    syscall[p] = defaultdict(list)
                syscall[p][extra(e)].append((e, c))

        s_trap_stat = trap_rec_stat(s_trap, 2, 3)
        u_trap_stat = trap_rec_stat(u_trap, 8, 9)
        syscall_stat = syscall_stat(syscall, 0, 0)

        for cause in s_trap_stat.keys():
            stat = s_trap_stat[cause]
            plt.hist(stat, 1000)
            plt.title(trap_cause_name(cause))
            plt.show()

        for cause in u_trap_stat.keys():
            stat = u_trap_stat[cause]
            plt.hist(stat, 1000)
            plt.title(trap_cause_name(cause))
            plt.show()

        for sid in syscall_stat.keys():
            stat = syscall_stat[sid]
            plt.hist(stat, 1000)
            plt.title(syscall_name[sid])
            plt.show()
