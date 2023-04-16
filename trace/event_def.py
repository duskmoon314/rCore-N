event_map = {
    0x57AB: {
        "name": "S trap",
        "sub_event": {
            0x0: {"name": "stvec enter"},
            0x1: {"name": "stvec restore"},
            0x2: {"name": "trap handler"},
            0x3: {"name": "trap return"},
            0x4: {"name": "SEI enter"},
            0x5: {"name": "SEI exit"},
        },
    },
    0x5CED: {
        "name": "scheduler",
        "sub_event": {
            0x0: {"name": "schedule"},
            0x1: {"name": "run next"},
            0x2: {"name": "suspend current"},
        },
    },
    0xC7AB: {
        "name": "U trap",
        "sub_event": {
            0x0: {"name": "enable UEI enter"},
            0x1: {"name": "enable UEI exit"},
            0x2: {"name": "disable UEI enter"},
            0x3: {"name": "disable UEI exit"},
            0x4: {"name": "push trap record enter"},
            0x5: {"name": "push trap record exit"},
            0x6: {"name": "trap queue enter"},
            0x7: {"name": "trap queue exit"},
            0x8: {"name": "trap handler"},
            0x9: {"name": "trap return"},
            0xA: {"name": "UEI handler"},
            0xB: {"name": "USI handler"},
            0xC: {"name": "UTI handler"},
        },
    },
    0x575C: {
        "name": "syscall",
        "sub_event": {
            0x0: {"name": "U enter"},
            0x1: {"name": "U exit"},
            0x2: {"name": "S enter"},
            0x3: {"name": "S exit"},
            0x4: {"name": "write find fd"},
            0x5: {"name": "write res"},
            0x6: {"name": "read find fd"},
            0x7: {"name": "read res"},
        },
    },
    0x5B1C: {
        "name": "SBI call",
        "sub_event": {
            0x0: {"name": "send IPI enter"},
            0x1: {"name": "send IPI exit"},
        },
    },
    0x5E1A: {
        "name": "serial driver",
        "sub_event": {
            0x0: {"name": "intr enter"},
            0x1: {"name": "intr exit"},
            0x2: {"name": "call enter"},
            0x3: {"name": "call exit"},
            0x4: {"name": "test enter"},
            0x5: {"name": "test exit"},
        },
    },
    0x911C: {
        "name": "PLIC",
        "sub_event": {
            0x0: {"name": "claim"},
            0x1: {"name": "complete enter"},
            0x2: {"name": "complete exit"},
        },
    },
    0x315C: {
        "name": "misc",
        "sub_event": {
            0x0: {"name": "trace test"},
        },
    },
}

cause_intr = {
    0: "usi",
    1: "ssi",
    2: "hsi",
    3: "msi",
    4: "uti",
    5: "Supervisor Timer Interrupt",
    6: "hti",
    7: "mti",
    8: "User External Interrupt",
    9: "Supervisor External Interrupt",
    10: "hei",
    11: "mei",
}

cause_excep = {
    0: "Instruction Address Misaligned",
    8: "User Environment Call",
    9: "s-ecall",
    11: "m-ecall",
    13: "Load Page Fault",
    15: "Store Page Fault",
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

serial_call_name = {
    63: "User Serial Read (Poll)",
    64: "User Serial Write (Poll)",
    65: "User Serial Read (Intr)",
    66: "User Serial Write (Intr)",
}


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
