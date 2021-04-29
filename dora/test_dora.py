import json
from dora import explore

test_json = """
{
    "param1": {
        "a": [1, 2, 3],
        "b": [ "$a * 2$", "$a * 4$", 1024 ],
        "c": "$param2 * 3$"
    },
    "param2": "a",
    "param3": [{
        "hey": 1,
        "bye": 2
    }],
    "param4": [4, 5, 6]
}
"""

if __name__ == '__main__':
    blob = json.loads(test_json)
    for obj in explore(blob):
        print(obj)
