from fractions import gcd
from itertools import product

def gcd2(a, b):
    if a == 0:
        return b, 0, 1
    else:
        g, y, x = gcd2(b % a, a)
        return g, x - (b//a) * y, y


def mod_inv(a, n):
    g, y, _ = gcd2(a, n)

    if g == 1:
        return (n + y) % n
    
    raise ValueError("a, n not coprime!")


def legendre_symbol(a, p):
    return pow(a, (p - 1) // 2, p)


def is_quadratic_residue(a, p):
    return legendre_symbol(a, p) == 1


def tonelli_shanks(n, p):
    if not is_quadratic_residue(n, p):
        return ()
        
    q = p - 1
    s = 0

    while ~q & 1:
        q = q >> 1
        s += 1
    
    if s == 1:
        x = pow(n, (p + 1)//4, p)
        return x, p - x
    
    z = 0
    for k in range(1, p):
        if not is_quadratic_residue(k, p):
            z = k
            break
    
    c = pow(z, q, p)
    r = pow(n, (q + 1) // 2, p)
    t = pow(n, q, p)
    m = s

    while t != 1:
        i = 1
        x = (t * t) % p

        while x != 1:
            x = (x*x) % p
            i += 1
        
        b = pow(c, (1 << (m - i - 1)), p)

        r = (r * b) % p
        c = (b * b) % p
        t = (t * c) % p
        m = i
    
    return r, p - r


def chinese_remainder_theorem(pr):
    x = 0
    m = 1

    for (_, pi) in pr:
        m *= pi
    
    for (ai, pi) in pr:
        y0 = m // pi
        y1 = mod_inv(y0, pi)

        x += ((ai * y0 % m) * y1) % m
        if x >= m:
            x -= m
    
    return x


def sqrt_mod_n(x, n):
    """Solve for all possible y where y ^ 2 == x (mod n) with n being prime"""
    return tonelli_shanks(x, n)


def sqrt_mod_pq(x, p, q):
    """Solve for all possible y where y ^ 2 == x (mod p*q) with p, q primes"""
    n = p * q

    x = x % n

    solutions = []

    for i in [ p, q ]:
        tmp = sqrt_mod_n(x, i)
        solutions.append([ (j, i) for j in tmp ])

    return [ chinese_remainder_theorem(it) for it in product(*solutions) ]

