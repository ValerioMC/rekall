"""Compare two recordings of calls.py or reads.py (the Java server's and the Rust server's).

Reports every call whose answer differs in content, and separately counts the ones that differ
only in the order of JSON object keys: the Java server built some objects with `Map.of`, whose
iteration order changes from one JVM start to the next, so those orders are not a contract.
"""
import json
import re
import sys

java, rust = (json.load(open(path)) for path in sys.argv[1:3])
content, order_only = 0, 0
for a, b in zip(java, rust):
    ta = re.sub(r'"timestamp": "[^"]*"', '"timestamp": "<ts>"', json.dumps(a, ensure_ascii=False))
    tb = re.sub(r'"timestamp": "[^"]*"', '"timestamp": "<ts>"', json.dumps(b, ensure_ascii=False))
    if ta == tb:
        continue
    if json.loads(ta) == json.loads(tb):
        order_only += 1
        continue
    content += 1
    print("###", a["label"], "|", a.get("call", ""))
    for key in sorted(set(a) | set(b)):
        if a.get(key) != b.get(key):
            print("  JAVA", key, json.dumps(a.get(key), ensure_ascii=False)[:1200])
            print("  RUST", key, json.dumps(b.get(key), ensure_ascii=False)[:1200])
print(f"{len(java)} calls: {content} differ in content, {order_only} only in key order")
