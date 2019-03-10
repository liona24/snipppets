TEENS = {
    0: 'Zero',
    1: 'One',
    2: 'Two',
    3: 'Three',
    4: 'Four',
    5: 'Five',
    6: 'Six',
    7: 'Seven',
    8: 'Eight',
    9: 'Nine',
    10: 'Ten',
    11: 'Eleven',
    12: 'Twelve',
    13: 'Thirteen',
    14: 'Fourteen',
    15: 'Fifteen',
    16: 'Sixteen',
    17: 'Seventeen',
    18: 'Eighteen',
    19: 'Nineteen'
}
TYS = {
    20: 'Twenty',
    30: 'Thirty',
    40: 'Forty',
    50: 'Fifty',
    60: 'Sixty',
    70: 'Seventy',
    80: 'Eighty',
    90: 'Ninety'
}

HUNDRED = 'Hundred'

BIG_BOYS = {
    1000: 'Thousand',
    1000000: 'Million',
    1000000000: 'Billion',
    1000000000000: 'Trillion'
}

def hundreds(n, skip_zero=True):
    n = n % 1000
    if n == 0:
        if skip_zero:
            return []
        return [TEENS[0]]

    # from here no zero should be returned

    tmp = n % 100
    words = []
    if tmp != 0:
        if tmp < 20:
            words.append(TEENS[tmp])
        elif tmp in TYS:
            words.append(TYS[tmp])
        else:
            words.append(TEENS[tmp % 10])
            words.append(TYS[(tmp // 10) * 10])

    n = n // 100
    if n == 0:
        return words
    tmp = n % 10
    if tmp > 0:
        words.append(HUNDRED)
        words.append(TEENS[tmp])    
    
    return words


def _reverse_concat(words):
    return ' '.join(reversed(words))


def number_in_words(n):
    if n < 1000:
        return _reverse_concat(hundreds(n, skip_zero=False))

    words = hundreds(n)

    for exp in sorted(BIG_BOYS.keys()):
        tmp = n // exp
        if tmp == 0:
            return _reverse_concat(words)

        tmp = tmp % 1000
        if tmp > 0:
            words.append(BIG_BOYS[exp])
            words.extend(hundreds(tmp))
    
    return _reverse_concat(words)


