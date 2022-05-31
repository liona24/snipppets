import ctypes
import subprocess


class _IO_COUNTERS(ctypes.Structure):
    _fields_ = [
        ("ReadOperationCount", ctypes.c_uint64),
        ("WriteOperationCount", ctypes.c_uint64),
        ("OtherOperationCount", ctypes.c_uint64),
        ("ReadTransferCount", ctypes.c_uint64),
        ("WriteTransferCount", ctypes.c_uint64),
        ("OtherTransferCount", ctypes.c_uint64),
    ]


PROCESS_QUERY_LIMITED_INFORMATION = 0x1000


def run(read_limit, write_limit, *args, **kvargs):
    p = subprocess.Popen(*args, **kvargs)

    handle = ctypes.windll.kernel32.OpenProcess(
        PROCESS_QUERY_LIMITED_INFORMATION,
        False,
        p.pid
    )

    io_counters = _IO_COUNTERS()

    ret = None

    while ret is None:

        try:
            ret = p.wait(0.5)
        except subprocess.TimeoutExpired:
            success = ctypes.windll.kernel32.GetProcessIoCounters(handle, ctypes.byref(io_counters))
            if not success:
                if p.poll() is None:
                    raise RuntimeError("Could not access IoCounters!")
                else:
                    break
            else:
                kill = False
                if read_limit > 0 and io_counters.ReadTransferCount >= read_limit:
                    kill = True
                if write_limit > 0 and io_counters.WriteTransferCount >= write_limit:
                    kill = True

                if kill:
                    p.kill()
                    return p.wait()

    return p
