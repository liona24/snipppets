"""A simple TCP-Proxy.

This is a snippet to create a simple TCP proxy server which can accept
multiple connections and forward them to a single remote address.
"""
import socket
import time
import select
import sys


BUFF_SIZE = 4096
DELAY = 0.0001

BIND_PORT = 9999


def pp_peername(peername):
    """Prettily format a peername tuple (host, port)"""
    return f"{peername[0]}:{peername[1]}"


def process_data(data, peer_from, peer_to):
    print(f"[D] {pp_peername(peer_from)} --> {pp_peername(peer_to)}")
    # TODO put custom data processing here if desired
    return data


def accept(server):
    conn, addr = server.accept()
    print(f"[S] Client connected from {pp_peername(addr)}")

    try:
        forward = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        forward.connect((remote_host, remote_port))
    except Exception as e:
        print("Error while trying to connect to remote target:", e)
        conn.close()
        return None, None

    return conn, forward


if __name__ == '__main__':
    try:
        remote_host, remote_port = sys.argv[1:3]
        remote_port = int(remote_port)
    except:
        print(f"Usage: {sys.argv[0]} <REMOTE_HOST> <REMOTE_PORT>")
        sys.exit(1)

    server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    server.bind(('127.0.0.1', BIND_PORT))
    server.listen()
    print(f"[S] Proxy listening on 127.0.0.1:{BIND_PORT} ..")
    print(f"[S] Forwarding messages to {remote_host}:{remote_port} ..")

    sockets = [server]
    # we use this forward table to map bi-directional traffic
    forward_table = {}

    try:

        while True:
            time.sleep(DELAY)

            ready, _, _ = select.select(sockets, [], [])

            for s in ready:
                if s == server:
                    client, forward = accept(server)
                    if client is not None:
                        forward_table[client] = forward
                        forward_table[forward] = client
                        sockets.append(client)
                        sockets.append(forward)
                else:
                    data = s.recv(BUFF_SIZE)
                    if len(data) == 0:
                        print(f"[S] {pp_peername(s.getpeername())} disconnected.")
                        f = forward_table.pop(s)
                        forward_table.pop(f)
                        sockets.remove(f)
                        sockets.remove(s)
                        f.close()
                        s.close()
                        break
                    else:
                        f = forward_table[s]
                        data = process_data(data, s.getpeername(), f.getpeername())
                        f.send(data)
    except KeyboardInterrupt:
        print("[S] Shutting down ..")
        for s in forward_table:
            s.close()

